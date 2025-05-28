use axaddrspace::{device::AccessWidth, HostPhysAddr};
use axerrno::AxResult;

pub(crate) fn perform_mmio_read(addr: HostPhysAddr, width: AccessWidth) -> AxResult<usize> {
    let addr = axvisor_api::memory::phys_to_virt(addr).as_ptr();

    return match width {
        AccessWidth::Byte => Ok(unsafe { *(addr as *const u8) as _ }),
        AccessWidth::Word => Ok(unsafe { *(addr as *const u16) as _ }),
        AccessWidth::Dword => Ok(unsafe { *(addr as *const u32) as _ }),
        AccessWidth::Qword => Ok(unsafe { *(addr as *const u64) as _ }),
    };
}

pub(crate) fn perform_mmio_write(
    addr: HostPhysAddr,
    width: AccessWidth,
    val: usize,
) -> AxResult<()> {
    let addr = axvisor_api::memory::phys_to_virt(addr).as_mut_ptr();

    match width {
        AccessWidth::Byte => unsafe { *(addr as *mut u8) = val as _ },
        AccessWidth::Word => unsafe { *(addr as *mut u16) = val as _ },
        AccessWidth::Dword => unsafe { *(addr as *mut u32) = val as _ },
        AccessWidth::Qword => unsafe { *(addr as *mut u64) = val as _ },
    }

    Ok(())
}
