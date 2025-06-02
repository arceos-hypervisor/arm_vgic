use axaddrspace::{GuestPhysAddr, GuestPhysAddrRange, HostPhysAddr};
use axdevice_base::BaseDeviceOps;
use log::trace;

use crate::{registers_v3::{GICR_CLRLPIR, GICR_CTLR, GICR_ICACTIVER, GICR_ICENABLER, GICR_ICFGR, GICR_ICFGR_RANGE, GICR_ICPENDR, GICR_IIDR, GICR_IMPL_DEF_IDENT_REGS_END, GICR_IMPL_DEF_IDENT_REGS_START, GICR_INVALLR, GICR_INVLPIR, GICR_IPRIORITYR, GICR_IPRIORITYR_RANGE, GICR_ISACTIVER, GICR_ISENABLER, GICR_ISPENDR, GICR_PENDBASER, GICR_PROPBASER, GICR_SETLPIR, GICR_SGI_BASE, GICR_STATUSR, GICR_SYNCR, GICR_TYPER, GICR_TYPER_LAST, GICR_WAKER, MAINTENACE_INTERRUPT}, utils::{enable_one_lpi, perform_mmio_read, perform_mmio_write}};

pub struct VGicR {
    /// The address of the VGicR in the guest physical address space.
    pub addr: GuestPhysAddr,
    /// The size of the VGicR in bytes.
    pub size: usize,

    pub cpu_id: usize,
    pub host_gicr_base: HostPhysAddr,
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

                Ok(read_prop_baser())
            }
            GICR_SYNCR => {
                // always return 0 for synchronization register
                Ok(0)
            }
            GICR_SETLPIR | GICR_CLRLPIR | GICR_INVALLR => {
                perform_mmio_read(gicr_base + reg, width)
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
                // mmio_perform_access(gicr_base, mmio);
                set_prop_baser(value);
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

pub fn enable_ipi() {
    let base = host_gicr_base(this_cpu_id()) + GICR_SGI_BASE;

    unsafe {
        let gicr_waker = (base + GICR_WAKER) as *mut u32;
        gicr_waker.write_volatile(gicr_waker.read_volatile() & !0x02);
        while gicr_waker.read_volatile() & 0x04 != 0 {}

        let gicr_igroupr0 = (base + GICR_IGROUPR) as *mut u32;
        gicr_igroupr0.write_volatile(gicr_igroupr0.read_volatile() | (1 << SGI_IPI_ID));

        let gicr_isenabler0 = (base + GICR_ISENABLER) as *mut u32;
        let gicr_ipriorityr0 = (base + GICR_IPRIORITYR) as *mut u32;
        for irq_id in [SGI_IPI_ID, MAINTENACE_INTERRUPT] {
            let reg = irq_id / 4;
            let offset = irq_id % 4 * 8;
            let mask = ((1 << 8) - 1) << offset;
            let p = gicr_ipriorityr0.add(reg as _);
            let prio = p.read_volatile();

            p.write_volatile((prio & !mask) | (0x01 << offset));

            gicr_isenabler0.write_volatile(1 << irq_id);
        }
    }
}

pub struct LpiPropTable {
    phy_addr: usize,
    frame: Frame,
    baser_list: [usize; MAX_ZONE_NUM],
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
            baser_list: [0; MAX_ZONE_NUM],
        }
    }

    fn set_prop_baser(&mut self, zone_id: usize, value: usize) {
        assert!(zone_id < MAX_ZONE_NUM, "Invalid zone id!");
        self.baser_list[zone_id] = value;
    }

    fn read_prop_baser(&self, zone_id: usize) -> usize {
        assert!(zone_id < MAX_ZONE_NUM, "Invalid zone id!");
        self.baser_list[zone_id]
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

pub fn set_prop_baser(value: usize) {
    let mut lpt = LPT.get().unwrap().lock();
    lpt.set_prop_baser(this_zone_id(), value);
}

pub fn read_prop_baser() -> usize {
    let lpt = LPT.get().unwrap().lock();
    lpt.read_prop_baser(this_zone_id())
}

pub fn enable_one_lpi(lpi: usize) {
    let lpt = LPT.get().unwrap().lock();
    lpt.enable_one_lpi(lpi);
}
