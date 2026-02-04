use log::debug;
pub use vdev_if::VirtPlatformOp;
pub use vdev_if::{GuestPhysAddr, VirtDeviceOp};

pub use gicd::Gicd;

mod gicd;
pub mod icc;

pub struct VGic {}

impl VGic {
    pub fn new() -> Self {
        VGic {}
    }

    pub fn new_gicd(&mut self, plat: impl VirtPlatformOp, mmio: GuestPhysAddr) -> Gicd {
        let mmio = plat
            .alloc_mmio_region(Some(mmio), 0x10000)
            .expect("Failed to allocate MMIO for GICD");
        Gicd::new(mmio)
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

impl VirtDeviceOp for Gicd {
    fn name(&self) -> &str {
        "GICv3 distributor"
    }

    fn invoke(&mut self) {
        debug!("GICD run invoked");
    }
}
