extern crate alloc;
use crate::consts::{PPI_ID_MAX, SGI_ID_MAX, SPI_ID_MAX};
use crate::interrupt::VgicInt;

pub struct Vgicd {
    pub ctrlr: u32,
    pub typer: u32,
    pub iidr: u32,
    interrupt: [VgicInt; SPI_ID_MAX],
}

impl Vgicd {
    pub fn new() -> Self {
        let mut gic_int = [VgicInt::new(0, 0); SPI_ID_MAX];
        for (idx, item) in gic_int.iter_mut().enumerate() {
            *item = VgicInt::new(idx as u32, 0);
        }
        Self {
            ctrlr: 0,
            typer: 0,
            iidr: 0,
            interrupt: gic_int,
        }
    }

    pub fn vgicd_ctrlr_write(&mut self, ctrlr: usize) {
        self.ctrlr = ctrlr as u32;
    }

    pub fn vgicd_isenabler_read(&self, idx: u32) -> usize {
        let mut isenabler = 0;
        for i in 0..32 {
            if self.interrupt[(idx * 32 + i) as usize].get_enable() {
                isenabler |= 1 << i;
            }
        }
        isenabler
    }

    pub fn vgicd_isenabler_write(&mut self, idx: u32, isenabler: usize) {
        for i in 0..32 {
            if isenabler & (1 << i) != 0 {
                self.interrupt[(idx * 32 + i) as usize].set_enable(true);
            }
        }
    }

    pub fn inject_irq(&self, irq: u32) {
        self.interrupt[irq as usize].inject_irq();
    }
}
