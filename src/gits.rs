//! WARNING: Identical mapping only!!

use core::{cell::UnsafeCell, ptr};

use axaddrspace::{GuestPhysAddr, GuestPhysAddrRange, HostPhysAddr};
use axdevice_base::BaseDeviceOps;
use axvisor_api::memory::{phys_to_virt, PhysFrame};
use log::trace;
use memory_addr::PhysAddr;
use spin::{Mutex, Once};

use crate::{
    registers_v3::{
        GITS_BASER, GITS_CBASER, GITS_COLLECTION_BASER, GITS_CREADR, GITS_CTRL, GITS_CT_BASER,
        GITS_CWRITER, GITS_DT_BASER, GITS_TYPER,
    },
    utils_v3::{enable_one_lpi, perform_mmio_read, perform_mmio_write},
};

#[derive(Default)]
pub struct VirtualGitsRegs {
    pub ct_baser: usize,
    pub dt_baser: usize,

    pub cbaser: usize,
    pub creadr: usize,
    pub cwriter: usize,
}

pub struct Gits {
    pub addr: GuestPhysAddr,
    pub size: usize,

    pub host_gits_base: HostPhysAddr,
    pub is_root_vm: bool,

    pub regs: UnsafeCell<VirtualGitsRegs>,
}

impl Gits {
    fn regs(&self) -> &VirtualGitsRegs {
        unsafe { &*self.regs.get() }
    }

    fn regs_mut(&self) -> &mut VirtualGitsRegs {
        unsafe { &mut *self.regs.get() }
    }
}

impl BaseDeviceOps<GuestPhysAddrRange> for Gits {
    fn emu_type(&self) -> axdevice_base::EmuDeviceType {
        // todo: determine the correct type
        axdevice_base::EmuDeviceType::EmuDeviceTGPPT
    }

    fn address_range(&self) -> GuestPhysAddrRange {
        GuestPhysAddrRange::from_start_size(self.addr, self.size)
    }

    fn handle_read(
        &self,
        addr: <GuestPhysAddrRange as axaddrspace::device::DeviceAddrRange>::Addr,
        width: axaddrspace::device::AccessWidth,
    ) -> axerrno::AxResult<usize> {
        let gits_base = self.host_gits_base;
        let reg = addr - self.addr;
        // let reg = mmio.address;

        // mmio_perform_access(gits_base, mmio);
        match reg {
            GITS_CTRL => perform_mmio_read(gits_base + reg, width),
            GITS_CBASER => Ok(self.regs().cbaser),
            GITS_DT_BASER => {
                if self.is_root_vm {
                    perform_mmio_read(gits_base + reg, width)
                } else {
                    Ok(
                        (self.regs().dt_baser)
                            & (1usize.unbounded_shl(width.size() as u32 * 8) - 1),
                    )
                }
            }
            GITS_CT_BASER => {
                if self.is_root_vm {
                    perform_mmio_read(gits_base + reg, width)
                } else {
                    Ok(
                        (self.regs().ct_baser)
                            & (1usize.unbounded_shl(width.size() as u32 * 8) - 1),
                    )
                }
            }
            GITS_CWRITER => Ok(self.regs().cwriter),
            GITS_CREADR => Ok(self.regs().creadr),
            GITS_TYPER => perform_mmio_read(gits_base + reg, width),
            _ => perform_mmio_read(gits_base + reg, width),
        }
    }

    fn handle_write(
        &self,
        addr: <GuestPhysAddrRange as axaddrspace::device::DeviceAddrRange>::Addr,
        width: axaddrspace::device::AccessWidth,
        val: usize,
    ) -> axerrno::AxResult {
        let gits_base = self.host_gits_base;
        let reg = addr - self.addr;
        // let reg = mmio.address;

        // mmio_perform_access(gits_base, mmio);
        match reg {
            GITS_CTRL => perform_mmio_write(gits_base + reg, width, val),
            GITS_CBASER => {
                if self.is_root_vm {
                    perform_mmio_write(gits_base + reg, width, val)?;
                }

                self.regs_mut().cbaser = val;
                Ok(())
            }
            GITS_DT_BASER => {
                if self.is_root_vm {
                    perform_mmio_write(gits_base + reg, width, val)
                } else {
                    self.regs_mut().dt_baser = val;
                    Ok(())
                }
            }
            GITS_CT_BASER => {
                if self.is_root_vm {
                    perform_mmio_write(gits_base + reg, width, val)
                } else {
                    self.regs_mut().ct_baser = val;
                    Ok(())
                }
            }
            GITS_CWRITER => {
                self.regs_mut().cwriter = val;

                if val != 0 {
                    let regs = self.regs();
                    let cbaser = regs.cbaser;
                    let creadr = regs.creadr;

                    let mut cmdq = get_cmdq(self.host_gits_base).lock();
                    self.regs_mut().creadr = cmdq.insert_cmd(cbaser, creadr, val);
                }

                Ok(())
            }
            GITS_CREADR => {
                panic!("GITS_CREADR should not be written by guest!");
            }
            GITS_TYPER => perform_mmio_write(gits_base + reg, width, val),
            _ => perform_mmio_write(gits_base + reg, width, val),
        }
    }
}

pub struct Cmdq {
    phy_addr: PhysAddr,
    readr: usize,
    writer: usize,

    host_gits_base: HostPhysAddr,
}

impl Drop for Cmdq {
    fn drop(&mut self) {
        trace!("Cmdq dealloc 16 frames: {:?}", self.phy_addr);
        axvisor_api::memory::dealloc_contiguous_frames(self.phy_addr, 16)
    }
}

pub const BYTES_PER_CMD: usize = 0x20;
pub const QWORD_PER_CMD: usize = BYTES_PER_CMD >> 3; // 8 bytes per qword

impl Cmdq {
    fn new(host_gits_base: HostPhysAddr) -> Self {
        let phy_addr = axvisor_api::memory::alloc_contiguous_frames(16, 0).unwrap();
        trace!("Cmdq alloc 16 frames: {:?}", phy_addr);
        let r = Self {
            phy_addr,
            readr: 0,
            writer: 0,
            host_gits_base,
        };
        r.init_real_cbaser();
        r
    }

    fn init_real_cbaser(&self) {
        let reg = self.host_gits_base + GITS_CBASER;
        let writer = self.host_gits_base + GITS_CWRITER;
        let val = 0xb80000000000040f | self.phy_addr.as_usize();
        let ctrl = self.host_gits_base + GITS_CTRL;

        let reg_ptr = phys_to_virt(reg).as_mut_ptr_of::<u64>();
        let writer_ptr = phys_to_virt(writer).as_mut_ptr_of::<u64>();
        let ctrl_ptr = phys_to_virt(ctrl).as_mut_ptr_of::<u64>();

        unsafe {
            let origin_ctrl = ptr::read_volatile(ctrl_ptr);
            ptr::write_volatile(ctrl_ptr, origin_ctrl | 0xfffffffffffffffeu64); // turn off, vm will turn on this ctrl
            ptr::write_volatile(reg_ptr, val as u64);
            ptr::write_volatile(writer_ptr, 0 as u64); // init cwriter
        }
    }

    // it's ok to add qemu-args: -trace gicv3_gits_cmd_*, remember to remain `enable one lpi`
    fn analyze_cmd(&self, value: [u64; 4]) {
        let code = (value[0] & 0xff) as usize;
        match code {
            0x0b => {
                let id = value[0] & 0xffffffff00000000;
                let event = value[1] & 0xffffffff;
                let icid = value[2] & 0xffff;
                enable_one_lpi((event - 8192) as _);
                trace!(
                    "MAPI cmd, for device {:#x}, event = intid = {:#x} -> icid {:#x}",
                    id >> 32,
                    event,
                    icid
                );
            }
            0x08 => {
                let id = value[0] & 0xffffffff00000000;
                let itt_base = (value[2] & 0x000fffffffffffff) >> 8;
                trace!(
                    "MAPD cmd, set ITT: {:#x} to device {:#x}",
                    itt_base,
                    id >> 32
                );
            }
            0x0a => {
                let id = value[0] & 0xffffffff00000000;
                let event = value[1] & 0xffffffff;
                let intid = value[1] >> 32;
                let icid = value[2] & 0xffff;
                enable_one_lpi((intid - 8192) as _);
                trace!(
                    "MAPTI cmd, for device {:#x}, event {:#x} -> icid {:#x} + intid {:#x}",
                    id >> 32,
                    event,
                    icid,
                    intid
                );
            }
            0x09 => {
                let icid = value[2] & 0xffff;
                let rd_base = (value[2] >> 16) & 0x7ffffffff;
                trace!("MAPC cmd, icid {:#x} -> redist {:#x}", icid, rd_base);
            }
            0x05 => {
                trace!("SYNC cmd");
            }
            0x04 => {
                trace!("CLEAR cmd");
            }
            0x0f => {
                trace!("DISCARD cmd");
            }
            0x03 => {
                trace!("INT cmd");
            }
            0x0c => {
                trace!("INV cmd");
            }
            0x0d => {
                trace!("INVALL cmd");
            }
            _ => {
                trace!("other cmd, code: 0x{:x}", code);
            }
        }
    }

    fn insert_cmd(&mut self, vm_cbaser: usize, vm_creadr: usize, vm_writer: usize) -> usize {
        let vm_addr = vm_cbaser & 0xffffffffff000;

        let origin_vm_readr = vm_creadr;

        // todo: handle wrap around
        let cmd_size = vm_writer - origin_vm_readr;
        let cmd_num = cmd_size / BYTES_PER_CMD;

        trace!("cmd size: {:#x}, cmd num: {:#x}", cmd_size, cmd_num);

        let mut vm_cmdq_addr = PhysAddr::from_usize(vm_addr + origin_vm_readr);
        let mut real_cmdq_addr = self.phy_addr + self.readr;

        for _cmd_id in 0..cmd_num {
            let vm_cmdq_ptr = phys_to_virt(vm_cmdq_addr).as_mut_ptr_of::<[u64; QWORD_PER_CMD]>();
            let mut real_cmdq_ptr = phys_to_virt(real_cmdq_addr).as_mut_ptr_of::<u64>();

            unsafe {
                let v = ptr::read_volatile(vm_cmdq_ptr);
                self.analyze_cmd(v.clone());

                for i in 0..QWORD_PER_CMD {
                    ptr::write_volatile(real_cmdq_ptr, v[i] as u64);
                    real_cmdq_addr += 8;
                    real_cmdq_ptr = real_cmdq_ptr.add(1);
                }
            }
            vm_cmdq_addr += BYTES_PER_CMD;
            vm_cmdq_addr = (ring_ptr_update(vm_cmdq_addr.as_usize() - vm_addr) + vm_addr).into();
            real_cmdq_addr =
                (ring_ptr_update(real_cmdq_addr - self.phy_addr) + self.phy_addr.as_usize()).into();
        }

        self.writer += cmd_size;
        self.writer = ring_ptr_update(self.writer); // ring buffer ptr
        let cwriter_addr = self.host_gits_base + GITS_CWRITER;
        let creadr_addr = self.host_gits_base + GITS_CREADR;

        let cwriter_ptr = phys_to_virt(cwriter_addr).as_mut_ptr_of::<u64>();
        let creadr_ptr = phys_to_virt(creadr_addr).as_mut_ptr_of::<u64>();
        unsafe {
            ptr::write_volatile(cwriter_ptr, self.writer as _);
            loop {
                self.readr = (ptr::read_volatile(creadr_ptr)) as usize; // hw readr
                if self.readr == self.writer {
                    trace!(
                        "readr={:#x}, writer={:#x}, its cmd end",
                        self.readr,
                        self.writer
                    );
                    break;
                }
            }
        }

        return vm_writer;
    }
}

static CMDQ: Once<Mutex<Cmdq>> = Once::new();

fn get_cmdq(host_gits_base: HostPhysAddr) -> &'static Mutex<Cmdq> {
    if !CMDQ.is_completed() {
        CMDQ.call_once(|| Mutex::new(Cmdq::new(host_gits_base)));
    }

    CMDQ.get().unwrap()
}

fn ring_ptr_update(val: usize) -> usize {
    if val >= 0x10000 {
        val - 0x10000
    } else {
        val
    }
}
