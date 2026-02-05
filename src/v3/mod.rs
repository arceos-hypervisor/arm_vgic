use alloc::vec::Vec;

use vdev_if::IrqNum;
pub use vdev_if::{GuestPhysAddr, VirtDeviceOp, VirtPlatformOp};

pub use gicd::Gicd;

use crate::VGicConfig;

mod gicd;
mod gicr;

pub mod icc;

pub struct VGic {
    config: VGicConfig,
    virt_irqs: Vec<IrqNum>,
}

impl VGic {
    pub fn new(config: VGicConfig) -> Self {
        mmio_api::init(config.mmio);
        VGic {
            config,
            virt_irqs: vec![],
        }
    }

    pub fn new_virt_irq(&mut self, irq: IrqNum) {
        self.virt_irqs.push(irq);
    }

    pub fn build_gicd(&mut self, plat: impl VirtPlatformOp, mmio: GuestPhysAddr) -> Gicd {
        let size = 0x10000;
        let mmio_base: usize = mmio.into();
        let base = unsafe { mmio_api::ioremap(mmio_base.into(), size) }.unwrap();
        let mmio = plat
            .alloc_mmio_region(Some(mmio), size)
            .expect("Failed to allocate MMIO for GICD");

        Gicd::new(mmio, &self.config, &self.virt_irqs)
    }
}

impl VirtDeviceOp for VGic {
    fn name(&self) -> &str {
        "GICv3 distributor"
    }

    fn invoke(&mut self) {
        debug!("GICv3 run invoked");
    }
}
