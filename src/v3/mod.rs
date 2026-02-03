use vdev_if::MmioRegion;
pub use vdev_if::VirtPlatformOp;
pub use vdev_if::{GuestPhysAddr, VirtDeviceOp};

pub mod icc;

pub struct VGic<P: VirtPlatformOp> {
    gicd: MmioRegion,
    plat: P,
}

impl<P: VirtPlatformOp> VGic<P> {
    pub fn new(gicd: GuestPhysAddr, gicr: GuestPhysAddr, plat: P) -> Self {
        let gicd = plat.alloc_mmio_region(Some(gicd), 0x10000, false).unwrap();

        Self { plat, gicd }
    }
}

impl<P: VirtPlatformOp> VirtDeviceOp for VGic<P> {
    fn name(&self) -> &str {
        "GICv3 distributor"
    }
}
