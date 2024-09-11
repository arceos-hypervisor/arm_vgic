extern crate alloc;
use alloc::vec::Vec;
use axerrno::AxResult;

pub use arm_gicv2::GicInterface;

use log::*;
use spin::Mutex;

use crate::consts::*;
use crate::vgicc::Vgicc;

struct VgicInner {
    used_irq: [u32; SPI_ID_MAX / 32],
    ptov: [u32; SPI_ID_MAX],
    vtop: [u32; SPI_ID_MAX],
    gicc: Vec<Vgicc>,

    ctrlr: u32,
    typer: u32,
    iidr: u32,

    gicd_igroupr: [u32; SPI_ID_MAX / 32],
    gicd_isenabler: [u32; SPI_ID_MAX / 32],
    gicd_ipriorityr: [u8; SPI_ID_MAX],
    gicd_itargetsr: [u8; SPI_ID_MAX],
    gicd_icfgr: [u32; SPI_ID_MAX / 16],
}

pub struct Vgic {
    inner: Mutex<VgicInner>,
}

impl Vgic {
    pub fn new() -> Vgic {
        Vgic {
            inner: Mutex::new(VgicInner {
                gicc: Vec::new(),
                ctrlr: 0,
                typer: 0,
                iidr: 0,
                used_irq: [0; SPI_ID_MAX / 32],
                ptov: [0; SPI_ID_MAX],
                vtop: [0; SPI_ID_MAX],
                gicd_igroupr: [0; SPI_ID_MAX / 32],
                gicd_isenabler: [0; SPI_ID_MAX / 32],
                gicd_ipriorityr: [0; SPI_ID_MAX],
                gicd_itargetsr: [0; SPI_ID_MAX],
                gicd_icfgr: [0; SPI_ID_MAX / 16],
            }),
        }
    }

    pub(crate) fn handle_read8(&self, _addr: usize) -> AxResult<usize> {
        Ok(0)
    }

    pub(crate) fn handle_read16(&self, _addr: usize) -> AxResult<usize> {
        Ok(0)
    }

    pub(crate) fn handle_read32(&self, _addr: usize) -> AxResult<usize> {
        Ok(0)
    }

    pub(crate) fn handle_write8(&self, addr: usize, val: usize) {
        match addr {
            VGICD_CTLR => {
                // 这里只关心写入的最后两位，也就是 grp0 grp1
                let mut vgic_inner = self.inner.lock();
                vgic_inner.ctrlr = (val & 0b11) as u32;

                if vgic_inner.ctrlr > 0 {
                    for i in SGI_ID_MAX..SPI_ID_MAX {
                        if vgic_inner.used_irq[i / 32] & (1 << (i % 32)) != 0 {
                            GicInterface::set_enable(i, true);
                            // 设置优先级为0
                            GicInterface::set_priority(i, 0);
                        }
                    }
                } else {
                    for i in SGI_ID_MAX..SPI_ID_MAX {
                        if vgic_inner.used_irq[i / 32] & (1 << (i % 32)) != 0 {
                            GicInterface::set_enable(i, false);
                        }
                    }
                }
                // TODO: 告知其它PE开启或关闭相应中断
            }
            _ => {
                error!("Unkonwn addr: {:#x}", addr);
            }
        }
    }

    pub(crate) fn handle_write16(&self, addr: usize, val: usize) {
        match addr {
            VGICD_CTLR => self.handle_write8(addr, val),
            _ => {
                error!("Unkonwn addr: {:#x}", addr);
            }
        }
    }

    pub(crate) fn handle_write32(&self, addr: usize, val: usize) {
        match addr {
            VGICD_CTLR => self.handle_write8(addr, val),
            _ => {
                error!("Unkonwn addr: {:#x}", addr);
            }
        }
    }
}
