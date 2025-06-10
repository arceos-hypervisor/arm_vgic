use axaddrspace::{device::AccessWidth, GuestPhysAddr, GuestPhysAddrRange, HostPhysAddr};
use axdevice_base::{BaseDeviceOps, EmuDeviceType};
use axerrno::AxResult;
use bitmaps::Bitmap;
use log::debug;

use super::{
    registers::{
        GICDV3_CIDR0_RANGE, GICDV3_PIDR0_RANGE, GICDV3_PIDR4_RANGE, GICD_CTLR,
        GICD_ICACTIVER_RANGE, GICD_ICENABLER_RANGE, GICD_ICFGR_RANGE, GICD_ICPENDR_RANGE,
        GICD_IGROUPR_RANGE, GICD_IGRPMODR, GICD_IGRPMODR_RANGE, GICD_IIDR, GICD_IPRIORITYR_RANGE,
        GICD_IROUTER, GICD_IROUTER_RANGE, GICD_ISACTIVER_RANGE, GICD_ISENABLER_RANGE,
        GICD_ISPENDR_RANGE, GICD_ITARGETSR, GICD_ITARGETSR_RANGE, GICD_TYPER, GICD_TYPER2,
        MAX_IRQ_V3,
    },
    utils::{perform_mmio_read, perform_mmio_write},
};

pub const DEFAULT_GICD_SIZE: usize = 0x10000; // 64K

/// Virtual Generic Interrupt Controller (VGIC) Distributor (D) implementation.
///
/// For GIC version 3.
pub struct VGicD {
    /// The address of the VGicD in the guest physical address space.
    pub addr: GuestPhysAddr,
    /// The size of the VGicD in bytes.
    pub size: usize,

    /// IRQs assigned to this VGicD.
    pub assigned_irqs: Bitmap<{ MAX_IRQ_V3 }>,

    /// The host physical address of the VGicD.
    ///
    /// TODO: move host gicd access to a separate crate, maybe arm_gic_driver.
    pub host_gicd_addr: HostPhysAddr,
}

impl VGicD {
    pub fn new(addr: GuestPhysAddr, size: Option<usize>) -> Self {
        let size = size.unwrap_or(DEFAULT_GICD_SIZE);

        Self {
            addr,
            size,
            assigned_irqs: Bitmap::new(),
            host_gicd_addr: axvisor_api::arch::get_host_gicd_base(),
        }
    }

    pub fn assign_irq(&mut self, irq: u32, cpu_phys_id: usize, target_cpu_affinity: (u8, u8, u8, u8)) {
        debug!("Physically assigning IRQ {} to CPU {} with affinity {:?}",
            irq, cpu_phys_id, target_cpu_affinity);

        if irq >= MAX_IRQ_V3 as u32 {
            panic!("IRQ {} is out of range for VGicD", irq);
        }
        self.assigned_irqs.set(irq as usize, true);

        // TODO: update host GICD_ITARGETSR and GICD_IROUTER registers
        let gicd_itargetsr_paddr = self.host_gicd_addr + GICD_ITARGETSR + irq as usize;
        let gicd_itargetsr_vaddr = phys_to_virt(gicd_itargetsr_paddr);
        unsafe { core::ptr::write_volatile(gicd_itargetsr_vaddr.as_mut_ptr_of::<u8>(), 1u8 << (cpu_phys_id)); }

        let gicd_irouter_paddr = self.host_gicd_addr + GICD_IROUTER + (irq as usize) * 8;
        let gicd_irouter_vaddr = phys_to_virt(gicd_irouter_paddr);
        unsafe {
            core::ptr::write_volatile(
                gicd_irouter_vaddr.as_mut_ptr_of::<u64>(),
                (target_cpu_affinity.0 as u64) << 32
                    | 1 << 31 // set the routing mode bit
                    | (target_cpu_affinity.1 as u64) << 16
                    | (target_cpu_affinity.2 as u64) << 8
                    | target_cpu_affinity.3 as u64,
            );
        }
    }
}

impl BaseDeviceOps<GuestPhysAddrRange> for VGicD {
    fn emu_type(&self) -> axdevice_base::EmuDeviceType {
        EmuDeviceType::EmuDeviceTInterruptController
    }

    fn address_range(&self) -> GuestPhysAddrRange {
        GuestPhysAddrRange::from_start_size(self.addr, self.size)
    }

    fn handle_read(
        &self,
        addr: <GuestPhysAddrRange as axaddrspace::device::DeviceAddrRange>::Addr,
        width: axaddrspace::device::AccessWidth,
    ) -> axerrno::AxResult<usize> {
        let gicd_base = self.host_gicd_addr;
        let reg = addr - self.addr;

        debug!("vGICD read reg {:#x} width {:?}", reg, width);

        match reg {
            reg if GICD_IROUTER_RANGE.contains(&reg) => {
                let irq = (reg - GICD_IROUTER) as u32 / 8;

                if self.is_irq_assigned(irq) && self.is_irq_spi(irq) {
                    perform_mmio_read(gicd_base + reg, width)
                } else {
                    // If the IRQ is not assigned, return 0
                    Ok(0)
                }
            }
            reg if GICD_ITARGETSR_RANGE.contains(&reg) => {
                let irq = (reg - GICD_ITARGETSR) as u32 / 4;

                if self.is_irq_assigned(irq) && self.is_irq_spi(irq) {
                    perform_mmio_read(gicd_base + reg, width)
                } else {
                    // If the IRQ is not assigned, return 0
                    Ok(0)
                }
            }
            reg if GICD_ICENABLER_RANGE.contains(&reg)
                || GICD_ISENABLER_RANGE.contains(&reg)
                || GICD_ICPENDR_RANGE.contains(&reg)
                || GICD_ISPENDR_RANGE.contains(&reg)
                || GICD_ICACTIVER_RANGE.contains(&reg)
                || GICD_ISACTIVER_RANGE.contains(&reg) =>
            {
                self.irq_masked_read(reg, reg & 0x7f, 0, width, true)
            }
            reg if GICD_IGROUPR_RANGE.contains(&reg) => {
                self.irq_masked_read(reg, reg & 0x7f, 0, width, false)
            }
            reg if GICD_IGRPMODR_RANGE.contains(&reg) => {
                self.irq_masked_read(reg, reg & 0x7f, 0, width, false)
            }
            reg if GICD_ICFGR_RANGE.contains(&reg) => {
                self.irq_masked_read(reg, reg & 0xff, 1, width, false)
            }
            reg if GICD_IPRIORITYR_RANGE.contains(&reg) => {
                self.irq_masked_read(reg, reg & 0x3ff, 3, width, false)
            }
            reg if GICDV3_PIDR0_RANGE.contains(&reg)
                || GICDV3_PIDR4_RANGE.contains(&reg)
                || GICDV3_CIDR0_RANGE.contains(&reg)
                || reg == GICD_CTLR
                || reg == GICD_TYPER
                || reg == GICD_IIDR
                || reg == GICD_TYPER2 =>
            {
                // read-only
                // ignore write
                perform_mmio_read(gicd_base + reg, width)
            }
            _ => {
                todo!("vgicdv3 read unimplemented for reg {:#x}", reg);
            }
        }
    }

    fn handle_write(
        &self,
        addr: <GuestPhysAddrRange as axaddrspace::device::DeviceAddrRange>::Addr,
        width: axaddrspace::device::AccessWidth,
        val: usize,
    ) -> axerrno::AxResult {
        let gicd_base = self.host_gicd_addr;
        let reg = addr - self.addr;

        debug!("vGICD write reg {:#x} width {:?} val {:#x}", reg, width, val);

        match reg {
            reg if GICD_IROUTER_RANGE.contains(&reg) => {
                let irq = (reg - GICD_IROUTER) as u32 / 8;

                if self.is_irq_assigned(irq) && self.is_irq_spi(irq) {
                    perform_mmio_write(gicd_base + reg, width, val)
                } else {
                    // If the IRQ is not assigned, ignore the write
                    Ok(())
                }
            }
            reg if GICD_ITARGETSR_RANGE.contains(&reg) => {
                let irq = (reg - GICD_ITARGETSR) as u32 / 4;

                if self.is_irq_assigned(irq) && self.is_irq_spi(irq) {
                    perform_mmio_write(gicd_base + reg, width, val)
                } else {
                    // If the IRQ is not assigned, ignore the write
                    Ok(())
                }
            }
            reg if GICD_ICENABLER_RANGE.contains(&reg)
                || GICD_ISENABLER_RANGE.contains(&reg)
                || GICD_ICPENDR_RANGE.contains(&reg)
                || GICD_ISPENDR_RANGE.contains(&reg)
                || GICD_ICACTIVER_RANGE.contains(&reg)
                || GICD_ISACTIVER_RANGE.contains(&reg) =>
            {
                self.irq_masked_write(reg, reg & 0x7f, 0, width, true, val)
            }
            reg if GICD_IGROUPR_RANGE.contains(&reg) => {
                self.irq_masked_write(reg, reg & 0x7f, 0, width, false, val)
            }
            reg if GICD_IGRPMODR_RANGE.contains(&reg) => {
                self.irq_masked_write(reg, reg & 0x7f, 0, width, false, val)
            }
            reg if GICD_ICFGR_RANGE.contains(&reg) => {
                self.irq_masked_write(reg, reg & 0xff, 1, width, false, val)
            }
            reg if GICD_IPRIORITYR_RANGE.contains(&reg) => {
                self.irq_masked_write(reg, reg & 0x3ff, 3, width, false, val)
            }
            reg if GICDV3_PIDR0_RANGE.contains(&reg)
                || GICDV3_PIDR4_RANGE.contains(&reg)
                || GICDV3_CIDR0_RANGE.contains(&reg)
                || reg == GICD_CTLR
                || reg == GICD_TYPER
                || reg == GICD_IIDR
                || reg == GICD_TYPER2 =>
            {
                // read-only
                // ignore write
                Ok(())
            }
            _ => {
                todo!("vgicdv3 write unimplemented for reg {:#x}", reg);
            }
        }
    }
}

impl VGicD {
    pub fn is_irq_assigned(&self, irq: u32) -> bool {
        self.assigned_irqs.get(irq as usize)
    }

    pub fn is_irq_sgi(&self, irq: u32) -> bool {
        // Check if the IRQ is a Software Generated Interrupt (SGI)
        irq < 16
    }

    pub fn is_irq_spi(&self, irq: u32) -> bool {
        // Check if the IRQ is a Shared Peripheral Interrupt (SPI)
        irq >= 16 && irq < 1020
    }

    /// Returns the mask of bits for the irqs assigned to this VGicD, in a bit-field reg.
    pub fn irq_access_mask(
        &self,
        reg_offset: usize,
        bits_per_irq_shift: usize,
        width: AccessWidth,
    ) -> usize {
        if bits_per_irq_shift > 3 {
            panic!(
                "bits_per_irq_shift must be <= 3, got {}",
                bits_per_irq_shift
            );
        }

        // How many IRQs there are in the mmio region the access width covers?
        let irqs_in_access_width = width.size() << (3 - bits_per_irq_shift);
        // The first IRQ at the given register offset.
        let first_irq = reg_offset << (3 - bits_per_irq_shift);
        // The mask of a single IRQ in the bit-field register.
        let single_irq_mask = (1 << (bits_per_irq_shift + 1)) - 1;

        let mut mask = 0;
        for irq in 0..irqs_in_access_width {
            if self.is_irq_assigned((first_irq + irq) as _) {
                // If the IRQ is assigned, set the corresponding bits in the mask.
                mask |= single_irq_mask << (irq << bits_per_irq_shift);
            }
        }

        mask
    }

    pub fn irq_masked_read(
        &self,
        offset: usize,
        reg_offset: usize,
        bits_per_irq_shift: usize,
        width: AccessWidth,
        _is_poke: bool,
    ) -> AxResult<usize> {
        let mask = self.irq_access_mask(reg_offset, bits_per_irq_shift, width);

        Ok(perform_mmio_read(self.host_gicd_addr + offset, width)? & mask)
    }

    pub fn irq_masked_write(
        &self,
        offset: usize,
        reg_offset: usize,
        bits_per_irq_shift: usize,
        width: AccessWidth,
        is_poke: bool,
        val: usize,
    ) -> AxResult<()> {
        let mask = self.irq_access_mask(reg_offset, bits_per_irq_shift, width);

        if is_poke {
            perform_mmio_write(self.host_gicd_addr + offset, width, val & mask)
        } else {
            let _lock = GICD_LOCK.lock();

            let current_value = perform_mmio_read(self.host_gicd_addr + offset, width)?;
            let new_value = (current_value & !mask) | (val & mask);
            perform_mmio_write(self.host_gicd_addr + offset, width, new_value)
        }
    }
}

// Todo: move this lock to arceos or axvisor
static GICD_LOCK: spin::Mutex<()> = spin::Mutex::new(());
