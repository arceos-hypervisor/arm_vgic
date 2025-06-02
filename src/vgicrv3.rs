use core::{cell::UnsafeCell, ptr};

use axaddrspace::{GuestPhysAddr, GuestPhysAddrRange, HostPhysAddr};
use axdevice_base::BaseDeviceOps;
use log::trace;
use spin::{Mutex, Once};

use crate::{
    registers_v3::{
        GICR_CLRLPIR, GICR_CTLR, GICR_ICACTIVER, GICR_ICENABLER, GICR_ICFGR, GICR_ICFGR_RANGE,
        GICR_ICPENDR, GICR_IIDR, GICR_IMPL_DEF_IDENT_REGS_END, GICR_IMPL_DEF_IDENT_REGS_START,
        GICR_INVALLR, GICR_INVLPIR, GICR_IPRIORITYR, GICR_IPRIORITYR_RANGE, GICR_ISACTIVER,
        GICR_ISENABLER, GICR_ISPENDR, GICR_PENDBASER, GICR_PROPBASER, GICR_SETLPIR, GICR_SGI_BASE,
        GICR_STATUSR, GICR_SYNCR, GICR_TYPER, GICR_TYPER_LAST, GICR_WAKER, MAINTENACE_INTERRUPT,
    },
    utils::{enable_one_lpi, perform_mmio_read, perform_mmio_write},
};

pub struct VGicRRegs {
    pub propbaser: usize,
}

pub struct VGicR {
    /// The address of the VGicR in the guest physical address space.
    pub addr: GuestPhysAddr,
    /// The size of the VGicR in bytes.
    pub size: usize,

    pub cpu_id: usize,
    pub host_gicr_base: HostPhysAddr,
    pub regs: UnsafeCell<VGicRRegs>,
}

impl VGicR {
    pub fn regs(&self) -> &VGicRRegs {
        unsafe { &*self.regs.get() }
    }

    pub fn regs_mut(&self) -> &mut VGicRRegs {
        unsafe { &mut *self.regs.get() }
    }
}

impl BaseDeviceOps<GuestPhysAddrRange> for VGicR {
    fn emu_type(&self) -> axdevice_base::EmuDeviceType {
        axdevice_base::EmuDeviceType::EmuDeviceTInterruptController
    }

    fn address_range(&self) -> GuestPhysAddrRange {
        GuestPhysAddrRange::from_start_size(self.addr, self.size)
    }

    fn handle_read(
        &self,
        addr: <GuestPhysAddrRange as axaddrspace::device::DeviceAddrRange>::Addr,
        width: axaddrspace::device::AccessWidth,
    ) -> axerrno::AxResult<usize> {
        let gicr_base = self.host_gicr_base;
        let reg = addr - self.addr;
        match reg {
            GICR_CTLR => {
                // TODO: is cross vcpu access allowed?
                perform_mmio_read(gicr_base + reg, width)
            }
            GICR_TYPER => {
                let value = perform_mmio_read(gicr_base + reg, width)?;

                // TODO: implement this
                // if self.is_last_gicr {
                //     value |= GICR_TYPER_LAST;
                // }

                Ok(value)
            }
            GICR_IIDR | GICR_IMPL_DEF_IDENT_REGS_START..=GICR_IMPL_DEF_IDENT_REGS_END => {
                // Make these read-only registers accessible.
                perform_mmio_read(gicr_base + reg, width)
            }
            GICR_PENDBASER => {
                // every redist have its own pending tbl
                perform_mmio_read(gicr_base + reg, width)
            }
            GICR_PROPBASER => {
                // all the redist share one prop tbl
                // mmio_perform_access(gicr_base, mmio);

                Ok(self.regs().propbaser)
            }
            GICR_SYNCR => {
                // always return 0 for synchronization register
                Ok(0)
            }
            GICR_SETLPIR | GICR_CLRLPIR | GICR_INVALLR => perform_mmio_read(gicr_base + reg, width),
            reg if reg == GICR_STATUSR
                || reg == GICR_WAKER
                || reg == GICR_ISENABLER
                || reg == GICR_ICENABLER
                || reg == GICR_ISPENDR
                || reg == GICR_ICPENDR
                || reg == GICR_ISACTIVER
                || reg == GICR_ICACTIVER
                || GICR_IPRIORITYR_RANGE.contains(&reg)
                || GICR_ICFGR_RANGE.contains(&reg) =>
            {
                perform_mmio_read(gicr_base + reg, width)
            }
            _ => {
                todo!("vgicr read unimplemented for reg {:#x}", reg);
            }
        }
    }

    fn handle_write(
        &self,
        addr: <GuestPhysAddrRange as axaddrspace::device::DeviceAddrRange>::Addr,
        width: axaddrspace::device::AccessWidth,
        value: usize,
    ) -> axerrno::AxResult<()> {
        let gicr_base = self.host_gicr_base;
        let reg = addr - self.addr;
        match reg {
            GICR_CTLR => {
                // TODO: is cross zone access allowed?
                perform_mmio_write(gicr_base + reg, width, value)
            }
            GICR_PENDBASER => {
                // every redist have its own pending tbl
                perform_mmio_write(gicr_base + reg, width, value)
            }
            GICR_PROPBASER => {
                // all the redist share one prop tbl
                self.regs_mut().propbaser = value;
                Ok(())
            }
            GICR_SETLPIR | GICR_CLRLPIR | GICR_INVALLR => {
                perform_mmio_write(gicr_base + reg, width, value)
            }
            GICR_INVLPIR => {
                // Presume that this write is to enable an LPI.
                // Or we need to check all the proptbl created by vm.
                enable_one_lpi((value & 0xffffffff) - 8192); // ⬅️Why?
                Ok(())
            }
            reg if reg == GICR_STATUSR
                || reg == GICR_WAKER
                || reg == GICR_ISENABLER
                || reg == GICR_ICENABLER
                || reg == GICR_ISPENDR
                || reg == GICR_ICPENDR
                || reg == GICR_ISACTIVER
                || reg == GICR_ICACTIVER
                || GICR_IPRIORITYR_RANGE.contains(&reg)
                || GICR_ICFGR_RANGE.contains(&reg) =>
            {
                let mut value = value;
                // avoid linux disable maintenance interrupt
                if reg == GICR_ICENABLER {
                    value &= !(1 << MAINTENACE_INTERRUPT);
                    // value &= !(1 << SGI_IPI_ID);
                }
                perform_mmio_write(gicr_base + reg, width, value)
            }
            _ => {
                todo!("vgicr write unimplemented for reg {:#x}", reg);
            }
        }
    }
}

// Following to be refactored into this repo

pub struct LpiPropTable {
    phy_addr: usize,
    frame: Frame,
}

impl LpiPropTable {
    fn new() -> Self {
        let gicd_typer =
            unsafe { ptr::read_volatile((host_gicd_base() + GICD_TYPER) as *const u32) };
        let id_bits = (gicd_typer >> 19) & 0x1f;
        let page_num: usize = ((1 << (id_bits + 1)) - 8192) / PAGE_SIZE;
        let f = Frame::new_contiguous(page_num, 0).unwrap();
        let propreg = f.start_paddr() | 0x78f;
        for id in 0..unsafe { consts::NCPU } {
            let propbaser = host_gicr_base(id) + GICR_PROPBASER;
            unsafe {
                ptr::write_volatile(propbaser as *mut u64, propreg as _);
            }
        }
        Self {
            phy_addr: f.start_paddr(),
            frame: f,
        }
    }

    fn enable_one_lpi(&self, lpi: usize) {
        let addr = self.phy_addr + lpi;
        let val: u8 = 0b1;
        // no priority
        unsafe {
            ptr::write_volatile(addr as *mut u8, val as _);
        }
    }
}

pub static LPT: Once<Mutex<LpiPropTable>> = Once::new();

pub fn init_lpi_prop() {
    LPT.call_once(|| Mutex::new(LpiPropTable::new()));
}

pub fn enable_one_lpi(lpi: usize) {
    let lpt = LPT.get().unwrap().lock();
    lpt.enable_one_lpi(lpi);
}
