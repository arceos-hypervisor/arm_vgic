extern crate alloc;
use core::usize;

use alloc::vec::Vec;
use axerrno::AxResult;

// pub use arm_gicv2::GicInterface;
use axdevice_base::VCpuIf;
use axhal::irq::MyVgic;

use log::*;
use spin::Mutex;

use crate::consts::*;
// use crate::vgicc::Vgicc;
// use crate_interface::call_interface;
// pub use vcpu_if::*;

struct CoreState {
    id: u32,
    vmcr: u32,
    pending_lr: [u32; SPI_ID_MAX],
    saved_lr: [u32; GICH_LR_NUM],
    saved_elsr0: u32,
    saved_apr: u32,
    saved_hcr: u32,
    irq_no_mask: [u32; SPI_ID_MAX / 32],

    ppi_isenabler: u32,
    ppi_ipriorityr: [u8; PPI_ID_MAX],
}

struct VgicInner {
    used_irq: [u32; SPI_ID_MAX / 32],
    ptov: [i32; SPI_ID_MAX],
    vtop: [i32; SPI_ID_MAX],
    // gicc: Vec<Vgicc>,
    ctrlr: u32,
    typer: u32,
    iidr: u32,
    real_pri: u8,

    core_state: Vec<CoreState>,

    gicd_igroupr: [u32; SPI_ID_MAX / 32],
    gicd_isenabler: [u32; SPI_ID_MAX / 32],
    gicd_ipriorityr: [u8; SPI_ID_MAX],
    gicd_itargetsr: [u8; SPI_ID_MAX],
    gicd_icfgr: [u32; SPI_ID_MAX / 16],
}

pub struct Vgic {
    inner: Mutex<VgicInner>,
    vcpu_num: usize,
}

impl Vgic {
    pub fn new() -> Vgic {
        let mut this = Self {
            inner: Mutex::new(VgicInner {
                used_irq: [0; SPI_ID_MAX / 32],
                ptov: [0; SPI_ID_MAX],
                vtop: [0; SPI_ID_MAX],
                // gicc: Vec::new(),
                ctrlr: 0,
                typer: 0,
                iidr: 0,
                real_pri: 0,

                core_state: Vec::new(),

                gicd_igroupr: [0; SPI_ID_MAX / 32],
                gicd_isenabler: [0; SPI_ID_MAX / 32],
                gicd_ipriorityr: [0; SPI_ID_MAX],
                gicd_itargetsr: [0; SPI_ID_MAX],
                gicd_icfgr: [0; SPI_ID_MAX / 16],
            }),
            vcpu_num: 1,
        };
        Self::init(&mut this);
        this
    }

    fn init(this: &mut Self) {
        let gicd = MyVgic::get_gicd();
        let mut vgic = this.inner.lock();
        for i in 0..SGI_ID_MAX {
            vgic.ptov[i] = i as i32;
            vgic.vtop[i] = i as i32;
        }

        for i in SGI_ID_MAX..SPI_ID_MAX {
            vgic.ptov[i] = -1;
            vgic.vtop[i] = -1;
        }

        for i in 0..SPI_ID_MAX / 32 {
            vgic.used_irq[i] = 0;
        }

        for i in 0..this.vcpu_num {
            vgic.core_state.push(CoreState {
                id: i as u32,
                vmcr: 0,
                pending_lr: [0; SPI_ID_MAX],
                saved_lr: [0; GICH_LR_NUM],
                saved_elsr0: 0,
                saved_apr: 0,
                saved_hcr: 0x5,
                irq_no_mask: [0; SPI_ID_MAX / 32],

                ppi_isenabler: 0xffff,
                ppi_ipriorityr: [0; PPI_ID_MAX],
            });
        }

        vgic.typer = gicd.lock().get_typer();
        // vgic.typer = GicInterface::get_typer();
        vgic.typer &= !(1 << 10);
        vgic.typer &= !0xE0;

        // vgic.iidr = GicInterface::get_iidr();
        vgic.iidr = gicd.lock().get_iidr();
        for i in 0..SPI_ID_MAX / 32 {
            vgic.gicd_igroupr[i] = 0;
        }

        vgic.gicd_icfgr[0] = 0xaaaa_aaaa;
        vgic.gicd_icfgr[1] = 0x5554_0000;
        for i in PPI_ID_MAX / 2..SPI_ID_MAX / 16 {
            vgic.gicd_icfgr[i] = 0x5555_5555;
        }

        vgic.real_pri = 0;
    }

    // 注入中断
    pub fn inject(&self, vcpu: &dyn VCpuIf, int_id: usize) {
        let id = vcpu.vcpu_id();
        let mut inner = self.inner.lock();

        if int_id < SGI_ID_MAX {
            // 处理SGI（软件生成中断）
            inner.core_state[id].pending_lr[int_id] = 1;
        } else if int_id < PPI_ID_MAX {
            // 处理PPI（私有外围中断）
            let ppi_id = int_id - SGI_ID_MAX;
            inner.core_state[id].ppi_isenabler |= 1 << ppi_id;
        } else if int_id < SPI_ID_MAX {
            // 处理SPI（共享外围中断）
            let spi_id = int_id - PPI_ID_MAX;
            inner.used_irq[spi_id / 32] |= 1 << (spi_id % 32);
        } else {
            // 无效的中断ID
            error!("Invalid interrupt ID: {}", int_id);
        }

        // 触发中断处理
        if inner.ctrlr != 0 {
            Self::gic_enable_int(int_id, inner.real_pri);
        }
    }

    pub(crate) fn handle_read8(&self, addr: usize, vcpu: &dyn VCpuIf) -> AxResult<usize> {
        let value = self.handle_read32(addr, vcpu)?;
        return Ok((value >> (8 * (addr & 0x3))) & 0xff);
    }

    pub(crate) fn handle_read16(&self, addr: usize, vcpu: &dyn VCpuIf) -> AxResult<usize> {
        let value = self.handle_read32(addr, vcpu)?;
        return Ok((value >> (8 * (addr & 0x3))) & 0xffff);
    }

    pub(crate) fn handle_read32(&self, addr: usize, vcpu: &dyn VCpuIf) -> AxResult<usize> {
        match addr {
            VGICD_CTRL => Ok(self.inner.lock().ctrlr as usize),
            VGICD_TYPER => Ok(self.inner.lock().typer as usize),
            VGICD_IIDR => Ok(self.inner.lock().iidr as usize),
            VGICD_ISENABLER_X | VGICD_ICENABLER_X => {
                let id = vcpu.vcpu_id();
                Ok(self.inner.lock().core_state[id].ppi_isenabler as usize)
            }
            idr if idr >= VGICD_ISENABLER_X + 0x04 && idr < VGICD_ICENABLER_X => {
                Ok(self.inner.lock().gicd_isenabler[(idr - VGICD_ISENABLER_X) / 4] as usize)
            }
            idr if idr >= VGICD_ICENABLER_X + 0x04 && idr < VGICD_ISPENDER_X => {
                Ok(self.inner.lock().gicd_isenabler[(idr - VGICD_ICENABLER_X) / 4] as usize)
            }
            VGICD_ISPENDER_X => {
                let id = vcpu.vcpu_id();
                let mut value = self.inner.lock().core_state[id].irq_no_mask[0];
                for i in 0..GICH_LR_NUM {
                    let lr = VGICH_LR_X + i * 4;
                    if (lr & 1 << 28) != 0 && (lr & 0x1ff) / 32 == 0 {
                        value |= 1 << ((lr & 0x1ff) % 32)
                    }
                }
                return Ok(value as usize);
            }
            idr if idr >= VGICD_ISPENDER_X + 0x04
                && idr < VGICD_ISPENDER_X + (SPI_ID_MAX / 32) * 0x04 =>
            {
                let mut value = 0;
                let idx = idr - VGICD_ISPENDER_X / 4;
                for i in 0..self.vcpu_num {
                    value |= self.inner.lock().core_state[i].irq_no_mask[idx];
                    for _j in 0..GICH_LR_NUM {
                        let lr = VGICH_LR_X + i * 4;
                        if (lr & 1 << 28) != 0 && (lr & 0x1ff) / 32 == idx {
                            todo!("Get LR for read");
                        }
                    }
                }
                return Ok(value as usize);
            }
            idr if idr >= VGICD_ICPENDER_X + 0x04
                && idr < VGICD_ICPENDER_X + (SPI_ID_MAX / 32) * 0x04 =>
            {
                let mut value = 0;
                let idx = (idr - VGICD_ICPENDER_X) / 4;
                for i in 0..self.vcpu_num {
                    value |= self.inner.lock().core_state[i].irq_no_mask[idx];
                    for _j in 0..GICH_LR_NUM {
                        let lr = VGICH_LR_X + i * 4;
                        if (lr & 1 << 28) != 0 && (lr & 0x1ff) / 32 == idx {
                            todo!("Get LR for read");
                        }
                    }
                }
                return Ok(value as usize);
            }
            idr if idr >= VGICD_IPRIORITYR_X + (PPI_ID_MAX / 4) * 0x04
                && idr <= VGICD_IPRIORITYR_X + (SPI_ID_MAX / 4) * 0x04 =>
            {
                let id = vcpu.vcpu_id();
                return Ok(self.inner.lock().core_state[id].ppi_ipriorityr
                    [idr - VGICD_IPRIORITYR_X / 4] as usize);
            }
            idr if idr >= VGICD_ITARGETSR_X
                && idr <= VGICD_ITARGETSR_X + (PPI_ID_MAX / 4) * 0x04 =>
            {
                let id = vcpu.vcpu_id();
                let value = 1 << id;
                return Ok(value << 24 | value << 16 | value << 8 as usize);
            }
            idr if idr >= VGICD_ITARGETSR_X + (PPI_ID_MAX / 4) * 0x04
                && idr < VGICD_ITARGETSR_X + (SPI_ID_MAX / 4) * 0x04 =>
            {
                return Ok(self.inner.lock().gicd_itargetsr[(idr - VGICD_ITARGETSR_X) / 4] as usize)
            }
            idr if idr >= VGICD_ICFGR_X + PPI_ID_MAX / 16 * 0x04
                && idr < VGICD_ICFGR_X + (SPI_ID_MAX / 16) * 0x04 =>
            {
                return Ok(self.inner.lock().gicd_icfgr[(idr - VGICD_ICFGR_X) / 4] as usize);
            }
            _ => {
                error!("Unkonwn read addr: {:#x}", addr);
                Ok(0)
            }
        }
    }

    pub(crate) fn handle_write8(&self, addr: usize, val: usize, vcpu: &dyn VCpuIf) {
        match addr {
            VGICD_CTRL => {
                self.inner.lock().ctrlr = val as u32;
                if val > 0 {
                    for i in SGI_ID_MAX..SPI_ID_MAX {
                        if self.inner.lock().used_irq[i / 32] & (1 << (i % 32)) != 0 {
                            todo!("gic enable int");
                        }
                    }
                } else {
                    for i in SGI_ID_MAX..SPI_ID_MAX {
                        if self.inner.lock().used_irq[i / 32] & (1 << (i % 32)) != 0 {
                            todo!("gic enable int");
                        }
                    }
                }
            }
            idr if idr >= VGICD_ISENABLER_X && idr < VGICD_ISPENDER_X => {
                Self::handle_write32(&self, idr & !(0x3), val << (8 * (addr & 0x3)), vcpu);
            }
            idr if idr >= VGICD_IPRIORITYR_X && idr < VGICD_IPRIORITYR_X + PPI_ID_MAX => {
                let id = vcpu.vcpu_id();
                self.inner.lock().core_state[id].ppi_ipriorityr[idr - VGICD_IPRIORITYR_X] =
                    val as u8;
            }
            idr if idr >= VGICD_IPRIORITYR_X + PPI_ID_MAX && idr < VGICD_IPRIORITYR_X + 0x400 => {
                self.inner.lock().gicd_ipriorityr[idr - VGICD_IPRIORITYR_X] = val as u8;
            }
            idr if idr >= VGICD_SPENDSGIR_X && idr < VGICD_SPENDSGIR_X + SGI_ID_MAX => {
                todo!("virtual_gic_send_software_int_inner")
            }
            idr if idr >= VGICD_IGROUPR_X && idr < VGICD_ISENABLER_X => {
                error!("use group");
            }
            idr if idr >= VGICD_ICFGR_X + PPI_ID_MAX / 4
                && idr < VGICD_ICFGR_X + SPI_ID_MAX / 4 =>
            {
                let i = (idr - VGICD_ICFGR_X) / 4;
                self.inner.lock().gicd_icfgr[i] = val as u32 & 0x55;
                for j in 0..4 {
                    let id = self.inner.lock().vtop[i * 4 + j];
                    if id > 0 {
                        //let val = INW(GICD_ICFGR(id/16));
                        // UW val = INW(GICD_ICFGR(id/16));
                        // if(reg_value&(1 << (j*2+1))){
                        //     val |= 1 << (id%16*2 + 1);
                        // } else {
                        //     val &= ~(1 << (id%16*2 + 1));
                        // }
                        // OUTW(GICD_ICFGR(id/16),val);
                        trace!("ICFGR Read and Write");
                    }
                }
            }
            idr if idr >= VGICD_ITARGETSR_X + PPI_ID_MAX
                && idr < VGICD_ITARGETSR_X + SPI_ID_MAX =>
            {
                self.inner.lock().gicd_itargetsr[idr - VGICD_ITARGETSR_X] = val as u8;
            }
            _ => {
                error!("Unkonwn write addr: {:#x}", addr);
            }
        }
    }

    pub(crate) fn handle_write16(&self, addr: usize, val: usize, vcpu: &dyn VCpuIf) {
        match addr {
            VGICD_CTRL => {
                Self::handle_write8(&self, addr, val, vcpu);
            }
            idr if idr >= VGICD_IPRIORITYR_X && idr < VGICD_IPRIORITYR_X + PPI_ID_MAX => {
                Self::handle_write32(&self, idr & (!0x3), val << (8 * (addr & 0x3)), vcpu);
            }
            idr if idr >= VGICD_IPRIORITYR_X && idr < VGICD_IPRIORITYR_X + PPI_ID_MAX => {
                let id = vcpu.vcpu_id();
                self.inner.lock().core_state[id].ppi_ipriorityr[(idr - VGICD_IPRIORITYR_X) / 2] =
                    val as u8;
            }
            idr if idr >= VGICD_IPRIORITYR_X + PPI_ID_MAX && idr < VGICD_IPRIORITYR_X + 0x400 => {
                self.inner.lock().gicd_ipriorityr[(idr - VGICD_IPRIORITYR_X) / 2] = val as u8;
            }
            idr if idr >= VGICD_IGROUPR_X && idr < VGICD_ISENABLER_X => {
                error!("use group");
            }
            idr if idr >= VGICD_ICFGR_X + PPI_ID_MAX / 4
                && idr < VGICD_ICFGR_X + SPI_ID_MAX / 4 =>
            {
                for i in 0..2 {
                    Self::handle_write8(&self, idr + i, (val >> (8 * i)) & 0xff, vcpu);
                }
            }
            idr if idr >= VGICD_ITARGETSR_X + PPI_ID_MAX
                && idr < VGICD_ITARGETSR_X + SPI_ID_MAX =>
            {
                self.inner.lock().gicd_itargetsr[(idr - VGICD_ITARGETSR_X) / 2] = val as u8;
            }
            _ => {
                error!("Unkonwn write addr: {:#x}", addr);
            }
        }
    }

    pub(crate) fn handle_write32(&self, addr: usize, val: usize, vcpu: &dyn VCpuIf) {
        match addr {
            VGICD_CTRL => self.handle_write8(addr, val, vcpu),
            VGICD_ISENABLER_X => {
                let id = vcpu.vcpu_id();
                self.inner.lock().core_state[id].ppi_isenabler |= val as u32;

                for j in 0..32 {
                    if val & 1 << j != 0 {
                        let p = self.inner.lock().vtop[j];
                        debug!("write32 addr: {:#x}, val: {:#x}", p, val);
                        if p >= 0 {
                            self.inner.lock().used_irq[(p / 32i32) as usize] |= 1 << (p % 32);
                            debug!(
                                "write32 addr: {:#x}, val: {:#x}",
                                p,
                                self.inner.lock().ctrlr
                            );
                            if self.inner.lock().ctrlr != 0 {
                                // Self::gic_enable_int(
                                //     p as usize,
                                //     self.inner.lock().core_state[id].ppi_ipriorityr[j]
                                //         + self.inner.lock().real_pri,
                                // );
                            }
                        }
                    }
                }
            }
            idr if idr >= VGICD_ISENABLER_X + 0x04 && idr < VGICD_ICENABLER_X => {
                let i = (idr - VGICD_ISENABLER_X) / 4;
                self.inner.lock().gicd_isenabler[i] |= val as u32;

                for j in 0..32 {
                    if val & (1 << j) != 0 {
                        let p = self.inner.lock().vtop[i * 32 + j];
                        if p >= 0 {
                            self.inner.lock().used_irq[(p / 32i32) as usize] |= 1 << (p % 32);
                            todo!("gic enable int");
                        }
                    }
                }
            }
            VGICD_ICENABLER_X => {
                let id = vcpu.vcpu_id();
                self.inner.lock().core_state[id].ppi_isenabler &= !(val as u32);

                for j in 0..32 {
                    if val & 1 << j != 0 {
                        let p = self.inner.lock().vtop[j];
                        if p >= 0 {
                            self.inner.lock().used_irq[(p / 32i32) as usize] |= 1 << (p % 32);
                            if self.inner.lock().ctrlr & 0x03 != 0 {
                                // Gic Disable p
                                debug!("Gic Diable {}", p);
                            }
                        }
                    }
                }
            }
            idr if idr >= VGICD_ICENABLER_X + 0x04 && idr < VGICD_ISPENDER_X + SPI_ID_MAX / 8 => {
                let i = (idr - VGICD_ICENABLER_X) / 4;
                self.inner.lock().gicd_isenabler[i] &= !(val as u32);
                for j in 0..32 {
                    if val & (1 << j) != 0 {
                        let p = self.inner.lock().vtop[i * 32 + j];
                        if p >= 0 {
                            self.inner.lock().used_irq[(p / 32i32) as usize] &= !(1 << (p % 32));
                            todo!("gic disable int");
                        }
                    }
                }
            }
            idr if idr >= VGICD_IPRIORITYR_X && idr < VGICD_IPRIORITYR_X + PPI_ID_MAX => {
                let id = vcpu.vcpu_id();
                self.inner.lock().core_state[id].ppi_ipriorityr[(idr - VGICD_IPRIORITYR_X) / 4] =
                    val as u8;
            }
            idr if idr >= VGICD_IPRIORITYR_X + PPI_ID_MAX
                && idr < VGICD_IPRIORITYR_X + SPI_ID_MAX =>
            {
                self.inner.lock().gicd_ipriorityr[(idr - VGICD_IPRIORITYR_X) / 4] = val as u8;
            }
            idr if idr >= VGICD_ICFGR_X + PPI_ID_MAX / 4
                && idr < VGICD_ICFGR_X + SPI_ID_MAX / 4 =>
            {
                for i in 0..4 {
                    Self::handle_write8(&self, idr + i, (val >> (8 * i)) & 0xff, vcpu);
                }
            }
            idr if idr >= VGICD_ITARGETSR_X + PPI_ID_MAX
                && idr < VGICD_ITARGETSR_X + SPI_ID_MAX =>
            {
                self.inner.lock().gicd_itargetsr[(idr - VGICD_ITARGETSR_X) / 4] = val as u8;
            }
            idr if idr >= VGICD_ICPENDER_X + 0x04
                && idr < VGICD_ICPENDER_X + (SPI_ID_MAX / 32) * 0x04 =>
            {
                // let mut _value = 0;
                let idx = (idr - VGICD_ICPENDER_X) / 4;
                for i in 0..self.vcpu_num {
                    self.inner.lock().core_state[i].irq_no_mask[idx] &= val as u32;
                    // for j in 0..GICH_LR_NUM {
                    //     let lr = VGICH_LR_X + i * 4;
                    //     if (lr & 1 << 28) != 0 && (lr & 0x1ff) / 32 == idx {
                    //         todo!("Get LR for read");
                    //     }
                    // }
                }
                // return Ok(value as usize);
            }
            // VGICD_ICENABLER_SGI_PPI => {}
            // VGICD_ISENABLER_SGI_PPI..=VGICD_ISENABLER_SPI => {
            //     error!("enabler emu");
            // }
            _ => {
                error!("Unkonwn write addr: {:#x}", addr);
            }
        }
    }

    fn gic_enable_int(intvec: usize, pri: u8) {
        todo!("gic enable int{} , {}", intvec, pri);
    }
}
