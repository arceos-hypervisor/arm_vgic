#![cfg_attr(not(test), no_std)]
#![recursion_limit = "512"]

#[macro_use]
extern crate log;

#[macro_use]
extern crate alloc;

use alloc::sync::Arc;

pub use mmio_api::MmioOp;
pub use vdev_if::*;

pub mod v3;

pub struct VGicConfig {
    pub cpu_num: usize,
    pub mmio: &'static dyn MmioOp,
    irq_chip: IrqChip,
}

impl VGicConfig {
    pub fn new(cpu_num: usize, mmio: &'static dyn MmioOp, irq_chip: impl IrqChipOp) -> Self {
        VGicConfig {
            cpu_num,
            mmio,
            irq_chip: Arc::new(irq_chip),
        }
    }
}

pub trait IrqChipOp: Send + Sync + 'static {
    fn get_cfg(&self, irq: IrqNum) -> Trigger;
    fn set_cfg(&self, irq: IrqNum, cfg: Trigger);
}

#[derive(Debug, Clone, Copy)]
pub enum Trigger {
    Level,
    Edge,
}

type IrqChip = Arc<dyn IrqChipOp>;
