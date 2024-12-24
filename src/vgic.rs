use log::error;

use crate::registers::GicRegister;
use crate::vgicd::Vgicd;
use axdevice_base::VCpuIf;
use axerrno::AxResult;
use spin::Mutex;

// 实现 Vgic
pub struct Vgic {
    vgicd: Mutex<Vgicd>,
}

impl Vgic {
    pub fn new() -> Vgic {
        Vgic {
            vgicd: Mutex::new(Vgicd::new()),
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

    pub fn handle_read32(&self, addr: usize, _vcpu: &dyn VCpuIf) -> AxResult<usize> {
        match GicRegister::from_addr(addr as u32) {
            Some(reg) => match reg {
                GicRegister::GicdCtlr => Ok(self.vgicd.lock().ctrlr as usize),
                // GicRegister::GicdTyper => Ok(self.vgicd.lock().typer as usize),
                // GicRegister::GicdIidr => Ok(self.vgicd.lock().iidr as usize),
                // // GicRegister::GicdStatusr => self.read_statusr(),
                // // GicRegister::GicdIgroupr(idx) => self.read_igroupr(idx),
                // GicRegister::GicdIsenabler(idx) => Ok(self.vgicd.lock().vgicd_isenabler_read(idx)),
                // GicRegister::GicdIcenabler(idx) => self.read_icenabler(idx),
                // GicRegister::GicdIspendr(idx) => self.read_ispendr(idx),
                _ => Ok(0),
            },
            None => {
                error!("Invalid read register address: {addr:#x}");
                Ok(0)
            }
        }
    }

    pub fn handle_write8(&self, addr: usize, value: usize, vcpu: &dyn VCpuIf) {
        self.handle_write32(addr, value, vcpu);
    }

    pub fn handle_write16(&self, addr: usize, value: usize, vcpu: &dyn VCpuIf) {
        self.handle_write32(addr, value, vcpu);
    }

    pub fn handle_write32(&self, addr: usize, value: usize, _vcpu: &dyn VCpuIf) {
        match GicRegister::from_addr(addr as u32) {
            Some(reg) => {
                match reg {
                    GicRegister::GicdCtlr => self.vgicd.lock().vgicd_ctrlr_write(value),
                    // GicRegister::GicdIsenabler(idx) => self.write_isenabler(idx, value),
                    GicRegister::GicdIsenabler(idx) => {
                        self.vgicd.lock().vgicd_isenabler_write(idx, value)
                    }
                    _ => self.nothing(0),
                }
            }
            None => error!("Invalid write register address: {addr:#x}"),
        }
    }

    pub fn inject_irq(&self, irq: u32) {
        self.vgicd.lock().inject_irq(irq);
    }

    pub fn nothing(&self, _value: u32) {}
}
