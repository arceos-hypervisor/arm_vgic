use aarch64_cpu::registers::{Readable, CNTPCT_EL0};
use aarch64_sysreg::SystemRegType;
use axaddrspace::device::{AccessWidth, DeviceAddrRange, SysRegAddr, SysRegAddrRange};
use axdevice_base::{BaseDeviceOps, EmuDeviceType};
use axerrno::AxResult;
use log::info;

impl BaseDeviceOps<SysRegAddrRange> for SysCntpctEl0 {
    fn emu_type(&self) -> EmuDeviceType {
        EmuDeviceType::Console
    }

    fn address_range(&self) -> SysRegAddrRange {
        SysRegAddrRange {
            start: SysRegAddr::new(SystemRegType::CNTPCT_EL0 as usize),
            end: SysRegAddr::new(SystemRegType::CNTPCT_EL0 as usize),
        }
    }

    fn handle_read(
        &self,
        _addr: <SysRegAddrRange as DeviceAddrRange>::Addr,
        _width: AccessWidth,
    ) -> AxResult<usize> {
        Ok(CNTPCT_EL0.get() as usize)
    }

    fn handle_write(
        &self,
        addr: <SysRegAddrRange as DeviceAddrRange>::Addr,
        _width: AccessWidth,
        val: usize,
    ) -> AxResult {
        info!("Write to emulator register: {addr:?}, value: {val}");
        Ok(())
    }
}

#[derive(Default)]
pub struct SysCntpctEl0 {
    // Fields
}

impl SysCntpctEl0 {
    pub fn new() -> Self {
        Self {
            // Initialize fields
        }
    }
}
