use alloc::vec::Vec;

use vdev_if::IrqNum;
pub use vdev_if::{GuestPhysAddr, VirtDeviceOp, VirtPlatformOp};

pub use gicd::Gicd;
pub use gicr::Gicr;

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

    pub fn build_gicr(
        &mut self,
        plat: impl VirtPlatformOp,
        vcpu_id: usize,
        mmio_base: GuestPhysAddr,
        mmio_sgi: GuestPhysAddr,
    ) -> Gicr {
        // 每个 GICR 区域大小为 8KB (GICR_BASE 4KB + GICR_SGI 4KB)
        let size = 0x2000;

        // 为 GICR_BASE 分配 MMIO 区域
        let mmio_base_region = plat
            .alloc_mmio_region(Some(mmio_base), size)
            .expect("Failed to allocate MMIO for GICR_BASE");

        // 为 GICR_SGI 分配 MMIO 区域
        let mmio_sgi_region = plat
            .alloc_mmio_region(Some(mmio_sgi), size)
            .expect("Failed to allocate MMIO for GICR_SGI");

        gicr::Gicr::new(
            mmio_base_region,
            mmio_sgi_region,
            vcpu_id,
            &self.config,
            &self.virt_irqs,
        )
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
