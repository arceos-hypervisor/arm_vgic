//! WARNING: Identical mapping only!!

use core::{
    cell::UnsafeCell,
    ptr::{self, NonNull},
};

use axaddrspace::{
    device::{AccessWidth, DeviceAddrRange},
    GuestPhysAddr, GuestPhysAddrRange, HostPhysAddr,
};
use axdevice_base::BaseDeviceOps;
use axerrno::ax_err;
use axvisor_api::memory::{alloc_contiguous_frames, dealloc_contiguous_frames, phys_to_virt};
use log::{debug, error, trace};
use memory_addr::{PhysAddr, PAGE_SIZE_4K};
use spin::{Mutex, Once};
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::ReadWrite,
};

use crate::v3::gits::registers::GITS_TYPER;

use super::utils::{perform_mmio_read, perform_mmio_write};

#[cfg(target_arch = "aarch64")]
use super::{utils::enable_one_lpi, vgicr::get_lpt};

#[cfg(not(target_arch = "aarch64"))]
pub fn enable_one_lpi(_lpi_id: u32) {
    panic!("enable_one_lpi is not supported on this architecture");
}

mod cmd;
mod registers;

pub use cmd::{DeviceID, EventID, ItsCommand, ItsCommandRaw, ICID, PINTID};

use registers::{
    GitsRegs, BASER, CBASER, CTLR, GITS_CBASER, GITS_CREADR, GITS_CTRL, GITS_CT_BASER,
    GITS_CT_INDEX, GITS_CWRITER, GITS_DT_BASER, GITS_DT_INDEX,
};

/// The number of frames used for the command queue.
pub const GITS_DRIVER_CMDQ_FRAMES: usize = 16; // 16 frames, 64 KiB
/// The alignment of the command queue in power-of-two frames.
pub const GITS_DRIVER_CMDQ_ALIGN2_POW2: usize = 0; // (2 ** 4) * 4 KiB = 64 KiB
/// The number of bytes used for the command queue.
pub const GITS_DRIVER_CMDQ_BYTES: usize = GITS_DRIVER_CMDQ_FRAMES * PAGE_SIZE_4K;
/// The number of frames used for the device table.
pub const GITS_DRIVER_DT_FRAMES: usize = 16; // 16 frames, 64 KiB
/// The alignment of the device table in power-of-two frames.
pub const GITS_DRIVER_DT_ALIGN2_POW2: usize = 4; // (2 ** 4) * 4 KiB = 64 KiB
/// The number of frames used for the command table.
pub const GITS_DRIVER_CT_FRAMES: usize = 16; // 16 frames, 64 KiB
/// The alignment of the command table in power-of-two frames.
pub const GITS_DRIVER_CT_ALIGN2_POW2: usize = 4; // (2 ** 4) * 4 KiB = 64 KiB

/// An driver
pub struct ItsDriver {
    /// A local copy of the `GITS_CREADR` register.
    creadr: usize,
    /// A local copy of the `GITS_CWRITER` register.
    cwriter: usize,

    gits_base: HostPhysAddr,
    gits: NonNull<GitsRegs>,

    cmdq_addr: HostPhysAddr, // command queue addr
    dt_addr: HostPhysAddr,   // device table addr
    ct_addr: HostPhysAddr,   // command table addr

    pub original_dt_baser: u64, // original baser0
    pub original_ct_baser: u64, // original baser1
}

unsafe impl Send for ItsDriver {}
unsafe impl Sync for ItsDriver {}

impl Drop for ItsDriver {
    fn drop(&mut self) {
        trace!("Cmdq dealloc 16 frames: {:?}", self.cmdq_addr);
        dealloc_contiguous_frames(self.cmdq_addr, GITS_DRIVER_CMDQ_FRAMES)
    }
}

pub const BYTES_PER_CMD: usize = 0x20;
pub const QWORD_PER_CMD: usize = BYTES_PER_CMD >> 3; // 8 bytes per qword

impl ItsDriver {
    /// Creates a new `GitsDriver` instance and initialize the physical GITS.
    pub fn new(gits_base: HostPhysAddr) -> Self {
        let mut gits = NonNull::new(phys_to_virt(gits_base).as_mut_ptr_of()).unwrap();

        let regs: &mut GitsRegs = unsafe { gits.as_mut() };

        // disable GITS, vm will enable it later
        regs.ctrl.modify(CTLR::ENABLED::CLEAR);

        // alloc 64 KiB (16 * 4-KiB frames) for command queue
        let cmdq_addr =
            alloc_contiguous_frames(GITS_DRIVER_CMDQ_FRAMES, GITS_DRIVER_CMDQ_ALIGN2_POW2).unwrap();
        // set the command queue base address
        let cbaser = CBASER::VALID::Valid
            + CBASER::INNER_CACHE::InnerWriteBack
            + CBASER::OUTER_CACHE::SameAsInner
            + CBASER::PHYSICAL_ADDRESS
                .val(cmdq_addr.as_usize() as u64 >> CBASER::PHYSICAL_ADDRESS.shift)
            + CBASER::SHAREABILITY::InnerShareable
            + CBASER::SIZE.val(const { GITS_DRIVER_CMDQ_FRAMES - 1 } as u64);
        regs.cbaser.write(cbaser);
        // init cwriter
        regs.cwriter.set(0);

        // read the original baser0 and baser1
        let original_dt_baser = regs.baser[GITS_DT_INDEX].get();
        let original_ct_baser = regs.baser[GITS_CT_INDEX].get();

        // alloc 64 KiB (16 * 4-KiB frames) each for dt and ct
        let dt_addr =
            alloc_contiguous_frames(GITS_DRIVER_DT_FRAMES, GITS_DRIVER_DT_ALIGN2_POW2).unwrap();
        let ct_addr =
            alloc_contiguous_frames(GITS_DRIVER_CT_FRAMES, GITS_DRIVER_CT_ALIGN2_POW2).unwrap();

        // set up the device table
        regs.baser[GITS_DT_INDEX].modify(
            BASER::VALID::Valid
                + BASER::INDIRECT::Direct
                + BASER::INNER_CACHE::InnerWriteBack
                + BASER::OUTER_CACHE::SameAsInner
                + BASER::PHYSICAL_ADDRESS
                    .val(dt_addr.as_usize() as u64 >> BASER::PHYSICAL_ADDRESS.shift)
                + BASER::SHAREABILITY::InnerShareable
                + BASER::PAGE_SIZE::Page4KiB
                + BASER::SIZE.val(const { GITS_DRIVER_DT_FRAMES - 1 } as u64),
        );

        // set up the command table
        regs.baser[GITS_CT_INDEX].modify(
            BASER::VALID::Valid
                + BASER::INDIRECT::Direct
                + BASER::INNER_CACHE::InnerWriteBack
                + BASER::OUTER_CACHE::SameAsInner
                + BASER::PHYSICAL_ADDRESS
                    .val(ct_addr.as_usize() as u64 >> BASER::PHYSICAL_ADDRESS.shift)
                + BASER::SHAREABILITY::InnerShareable
                + BASER::PAGE_SIZE::Page4KiB
                + BASER::SIZE.val(const { GITS_DRIVER_CT_FRAMES - 1 } as u64),
        );

        debug!(
            "GITS driver initialized: cmdq_addr = {:#x}, dt_addr = {:#x}, ct_addr = {:#x}",
            cmdq_addr, dt_addr, ct_addr
        );

        error!(
            "original GITS_DT_BASER: {:#x}, GITS_CT_BASER: {:#x}",
            original_dt_baser, original_ct_baser
        );

        error!(
            "current GITS_DT_BASER: {:#x}, GITS_CT_BASER: {:#x}",
            regs.baser[GITS_DT_INDEX].get(),
            regs.baser[GITS_CT_INDEX].get()
        );

        Self {
            creadr: 0,
            cwriter: 0,
            gits_base,
            gits,
            cmdq_addr,
            dt_addr,
            ct_addr,
            original_dt_baser,
            original_ct_baser,
        }
    }

    /// Returns a immutable reference to the GITS registers.
    pub fn regs(&self) -> &GitsRegs {
        unsafe { self.gits.as_ref() }
    }

    /// Returns a mutable reference to the GITS registers.
    pub fn regs_mut(&mut self) -> &mut GitsRegs {
        unsafe { self.gits.as_mut() }
    }
}

// command queue operations
impl ItsDriver {
    /// Push a command to the command queue.
    ///
    /// This function **DOES NOT** commit the pushed command to the hardware. Use
    /// [`flush_cmd_queue`](ItsDriver::flush_cmd_queue) to commit the commands.
    fn push_cmd_queue(&mut self, cmd: ItsCommandRaw) {
        let cmdq_wr_addr = phys_to_virt(self.cmdq_addr + self.cwriter);

        unsafe {
            ptr::write_volatile(cmdq_wr_addr.as_mut_ptr_of(), cmd);
        }

        self.cwriter = (self.cwriter + BYTES_PER_CMD) % GITS_DRIVER_CMDQ_BYTES;
    }

    /// Flush the command queue. Wait until the committed commands are processed by the hardware.
    fn flush_cmd_queue(&mut self) -> usize {
        debug!(
            "Flushing command queue..., local creadr: {:#x}, local cwriter: {:#x}",
            self.creadr, self.cwriter
        );

        self.regs().cwriter.set(self.cwriter as u64);

        loop {
            let phys_creadr = self.regs().creadr.get() as usize;

            if phys_creadr & 1 == 1 {
                panic!("GITS stalled!");
            }

            if phys_creadr == self.cwriter {
                let bytes_flushed =
                    (self.cwriter - self.creadr + GITS_DRIVER_CMDQ_BYTES) % GITS_DRIVER_CMDQ_BYTES;
                debug!(
                    "GITS command queue flushed, creadr: {:#x}, cwriter: {:#x}, bytes_flushed: {:#x}({} commands)",
                    phys_creadr, self.cwriter,
                    bytes_flushed, bytes_flushed / BYTES_PER_CMD
                );
                self.creadr = phys_creadr;

                return bytes_flushed;
            }
        }
    }

    // it's ok to add qemu-args: -trace gicv3_gits_cmd_*, remember to remain `enable one lpi`
    fn analyze_cmd(&self, value: [u64; 4]) {
        let raw = ItsCommandRaw(value);
        let cmd = ItsCommand::from_raw(raw);

        match cmd {
            Ok(
                ItsCommand::MAPI {
                    event_id: p_int_id, ..
                }
                | ItsCommand::MAPTI { p_int_id, .. },
            ) => {
                enable_one_lpi(p_int_id as _);
            }
            _ => {}
        }

        debug!("GITS command: {:?}, raw: {:?}", cmd, raw);
    }

    pub fn send_cmds(&mut self, cmds: impl IntoIterator<Item = ItsCommand>) {
        self.send_cmds_raw(cmds.into_iter().map(|cmd| cmd.into_raw()));
    }

    pub fn send_cmds_raw(&mut self, cmds: impl IntoIterator<Item = ItsCommandRaw>) {
        for cmd in cmds {
            self.push_cmd_queue(cmd);
        }

        // commit the commands to the hardware
        self.flush_cmd_queue();
    }

    /// WARNING: this function supports only GPA-HPA identical mapping!
    fn insert_cmd(&mut self, vm_cbaser: usize, vm_creadr: usize, vm_writer: usize) -> usize {
        let vm_addr = vm_cbaser & 0xf_ffff_ffff_f000;

        let origin_vm_readr = vm_creadr;

        // todo: handle wrap around
        let cmd_size = vm_writer - origin_vm_readr;
        let cmd_num = cmd_size / BYTES_PER_CMD;

        trace!(
            "vm_cbaser: {:#x}, vm_creadr: {:#x}, vm_writer: {:#x}, vm_addr: {:#x}",
            vm_cbaser,
            vm_creadr,
            vm_writer,
            vm_addr
        );
        debug!("cmd size: {:#x}, cmd num: {:#x}", cmd_size, cmd_num);

        let mut vm_cmdq_addr = PhysAddr::from_usize(vm_addr + origin_vm_readr);
        let mut real_cmdq_addr = self.cmdq_addr + self.creadr;

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
            real_cmdq_addr = (ring_ptr_update(real_cmdq_addr - self.cmdq_addr)
                + self.cmdq_addr.as_usize())
            .into();
        }

        self.cwriter += cmd_size;
        self.cwriter = ring_ptr_update(self.cwriter); // ring buffer ptr
        let cwriter_addr = self.gits_base + GITS_CWRITER;
        let creadr_addr = self.gits_base + GITS_CREADR;

        let cwriter_ptr = phys_to_virt(cwriter_addr).as_mut_ptr_of::<u64>();
        let creadr_ptr = phys_to_virt(creadr_addr).as_mut_ptr_of::<u64>();
        // let ctlr_ptr = phys_to_virt(self.host_gits_base + GITS_CTRL).as_mut_ptr_of::<u64>();
        unsafe {
            ptr::write_volatile(cwriter_ptr, self.cwriter as _);
            loop {
                self.creadr = (ptr::read_volatile(creadr_ptr)) as usize; // hw readr
                if self.creadr == self.cwriter {
                    debug!(
                        "readr={:#x}, writer={:#x}, its cmd end",
                        self.creadr, self.cwriter
                    );
                    break;
                }
            }
        }

        return vm_writer;
    }
}

static CMDQ: Once<Mutex<ItsDriver>> = Once::new();

fn get_cmdq(host_gits_base: HostPhysAddr) -> &'static Mutex<ItsDriver> {
    if !CMDQ.is_completed() {
        CMDQ.call_once(|| Mutex::new(ItsDriver::new(host_gits_base)));
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

#[derive(Default)]
pub struct VirtualGitsStates {
    pub ct_baser: usize,
    pub dt_baser: usize,

    pub cbaser: usize,
    pub creadr: usize,
    pub cwriter: usize,

    pub cmdq_size: usize,
}

pub const DEFAULT_GITS_SIZE: usize = 0x20_000; // 128Ki: two 64-Ki frames

/// A virtual GIC ITS interface.
pub struct Gits {
    /// The address of the GITS in the guest physical address space.
    pub addr: GuestPhysAddr,
    /// The size of the GITS in bytes.
    pub size: usize,
    /// The base address of the physical GITS in the host physical address space.
    pub host_gits_base: HostPhysAddr,
    // This flag can be used to allow a VM to access the GITS directly. We just disable this now.
    // /// Whether this GITS is for the root VM.
    // pub is_root_vm: bool,
    /// The internal states of the virtual GITS.
    pub regs: UnsafeCell<VirtualGitsStates>,
}

impl Gits {
    fn regs(&self) -> &VirtualGitsStates {
        unsafe { &*self.regs.get() }
    }

    fn regs_mut(&self) -> &mut VirtualGitsStates {
        unsafe { &mut *self.regs.get() }
    }

    pub fn new(
        addr: GuestPhysAddr,
        size: Option<usize>,
        host_gits_base: HostPhysAddr,
        is_root_vm: bool,
    ) -> Self {
        let size = size.unwrap_or(DEFAULT_GITS_SIZE); // 4K
        let regs = UnsafeCell::new(VirtualGitsStates::default());

        // ensure cmdq and lpi prop table is initialized before VMs are up
        #[cfg(target_arch = "aarch64")]
        {
            let cmdq = get_cmdq(host_gits_base);
            let _ = get_lpt(
                axvisor_api::arch::read_vgicd_typer(),
                axvisor_api::arch::get_host_gicr_base(),
                None, // Use default size
            );

            unsafe {
                let regs = &mut *regs.get();
                regs.dt_baser = cmdq.lock().original_dt_baser as _;
                regs.ct_baser = cmdq.lock().original_ct_baser as _;
            }
        }

        let _is_root_vm = is_root_vm;

        error!(
            "vITS created at {:#x}, size {:#x}, host_gits_base {:#x}",
            addr.as_usize(),
            size,
            host_gits_base.as_usize()
        );

        Self {
            addr,
            size,
            host_gits_base,
            regs,
        }
    }
}

impl BaseDeviceOps<GuestPhysAddrRange> for Gits {
    fn emu_type(&self) -> axdevice_base::EmuDeviceType {
        // todo: determine the correct type
        axdevice_base::EmuDeviceType::GPPTITS
    }

    fn address_range(&self) -> GuestPhysAddrRange {
        GuestPhysAddrRange::from_start_size(self.addr, self.size)
    }

    fn handle_read(
        &self,
        addr: <GuestPhysAddrRange as DeviceAddrRange>::Addr,
        width: AccessWidth,
    ) -> axerrno::AxResult<usize> {
        let gits_base = self.host_gits_base;
        let reg = addr - self.addr;
        // let reg = mmio.address;

        error!(
            "vITS({:#x}) read reg {:#x} width {:?}",
            self.addr.as_usize(),
            reg,
            width
        );

        // emulate cmdq and its table accesses, passthrough other accesses
        let result = match reg {
            GITS_CBASER => Ok(self.regs().cbaser),
            GITS_DT_BASER => Ok(self.regs().dt_baser),
            GITS_CT_BASER => Ok(self.regs().ct_baser),
            GITS_CWRITER => Ok(self.regs().cwriter),
            GITS_CREADR => Ok(self.regs().creadr),
            // includes GITS_CTRL | GITS_TYPER
            _ => perform_mmio_read(gits_base + reg, width),
        };

        error!(
            "vITS({:#x}) read reg {:#x} width {:?} value {:#x}",
            self.addr.as_usize(),
            reg,
            width,
            result.unwrap_or(0)
        );

        result
    }

    fn handle_write(
        &self,
        addr: <GuestPhysAddrRange as DeviceAddrRange>::Addr,
        width: AccessWidth,
        val: usize,
    ) -> axerrno::AxResult {
        let gits_base = self.host_gits_base;
        let reg = addr - self.addr;
        // let reg = mmio.address;

        error!(
            "vITS({:#x}) write reg {:#x} width {:?} value {:#x}",
            self.addr.as_usize(),
            reg,
            width,
            val
        );

        // mmio_perform_access(gits_base, mmio);
        match reg {
            GITS_CBASER => {
                let cmdq_size = (CBASER::SIZE.read(val as _) + 1) as usize;
                let cmdq_gpa = (CBASER::PHYSICAL_ADDRESS.read(val as _)
                    << CBASER::PHYSICAL_ADDRESS.shift) as usize;

                self.regs_mut().cbaser = val;
                self.regs_mut().cmdq_size = cmdq_size * PAGE_SIZE_4K;

                debug!(
                    "Guest allocated command queue: GPA {:#x}, size {:#x} bytes ({} frames).",
                    cmdq_gpa,
                    cmdq_size * PAGE_SIZE_4K,
                    cmdq_size
                );

                Ok(())
            }
            GITS_DT_BASER => {
                // the guest-allocated device table is not used by hardware
                self.regs_mut().dt_baser = val;
                Ok(())
            }
            GITS_CT_BASER => {
                // the guest-allocated interrupt collection table is not used by hardware
                self.regs_mut().ct_baser = val;
                Ok(())
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
            GITS_CREADR | GITS_TYPER => {
                error!(
                    "Guest trying to write read-only GITS register {:#x} at {:#x}, value {:#x}",
                    reg, addr, val
                );

                ax_err!(Unsupported)
            }
            // includes GITS_CTRL
            _ => perform_mmio_write(gits_base + reg, width, val),
        }
    }
}
