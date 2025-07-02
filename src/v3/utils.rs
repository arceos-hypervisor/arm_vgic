use axaddrspace::{device::AccessWidth, HostPhysAddr};
use axerrno::AxResult;
use axvisor_api::memory::phys_to_virt;

/// Perform a memory-mapped I/O (MMIO) read operation on a given host physical address.
///
/// If the width is shorter than the size of `usize`, the value will be zero-extended to fit into `usize`.
pub(crate) fn perform_mmio_read(addr: HostPhysAddr, width: AccessWidth) -> AxResult<usize> {
    let addr = phys_to_virt(addr).as_ptr();

    return match width {
        AccessWidth::Byte => Ok(unsafe { (addr as *const u8).read_volatile() as _ }),
        AccessWidth::Word => Ok(unsafe { (addr as *const u16).read_volatile() as _ }),
        AccessWidth::Dword => Ok(unsafe { (addr as *const u32).read_volatile() as _ }),
        AccessWidth::Qword => Ok(unsafe { (addr as *const u64).read_volatile() as _ }),
    };
}

/// Perform a memory-mapped I/O (MMIO) write operation on a given host physical address.
pub(crate) fn perform_mmio_write(
    addr: HostPhysAddr,
    width: AccessWidth,
    val: usize,
) -> AxResult<()> {
    let addr = phys_to_virt(addr).as_mut_ptr();

    match width {
        AccessWidth::Byte => unsafe {
            (addr as *mut u8).write_volatile(val as _);
        },
        AccessWidth::Word => unsafe {
            (addr as *mut u16).write_volatile(val as _);
        },
        AccessWidth::Dword => unsafe {
            (addr as *mut u32).write_volatile(val as _);
        },
        AccessWidth::Qword => unsafe {
            (addr as *mut u64).write_volatile(val as _);
        },
    }

    Ok(())
}

#[cfg(target_arch = "aarch64")]
pub use super::vgicr::enable_one_lpi;
