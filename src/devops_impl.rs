use axdevice_base::BaseDeviceOps;
use axdevice_base::EmuDeviceType;

use axaddrspace::GuestPhysAddr;
use axdevice_base::VCpuIf;
use axerrno::AxResult;
use memory_addr::AddrRange;

use crate::vgic::Vgic;

impl BaseDeviceOps for Vgic {
    /// Gets the emulator type of the current device.
    ///
    /// This function returns the emulator device type of the current instance. Specifically, it always returns `EmuDeviceType::EmuDeviceTGicdV2`,
    /// indicating that the emulator device is of type `EmuDeviceTGicdV2`.
    ///
    /// # Returns
    /// - Returns an instance of the `EmuDeviceType` enum, representing the specific type of the emulator device.
    fn emu_type(&self) -> EmuDeviceType {
        EmuDeviceType::EmuDeviceTGicdV2
    }

    /// Returns the address range for the device.
    ///
    /// This function defines the address range accessible to the device, starting from `0x800_0000`,
    /// with a length of `0x10000` (64KB). It is used to specify where the device can read or write in memory.
    ///
    /// # Returns
    /// An `AddrRange` instance representing the address range from `0x800_0000` to `0x800_FFFF`.
    fn address_range(&self) -> AddrRange<GuestPhysAddr> {
        AddrRange::new(0x800_0000.into(), (0x800_0000 + 0x10000).into())
    }

    /// Handles memory read operations.
    ///
    /// Based on the given physical address and read width, performs the corresponding read operation.
    /// Supports reading 1 byte, 2 bytes, and 4 bytes. This function dereferences the provided physical
    /// address and calls the specific read function based on the width parameter.
    ///
    /// Parameters:
    /// - `addr`: The physical address to read from.
    /// - `width`: The width of the data to be read, determining the size of the read operation.
    ///
    /// Returns:
    /// - `AxResult<usize>`: The result of the read operation, including any errors and the size of the data read.
    fn handle_read(&self, addr: GuestPhysAddr, width: usize, vcpu: &dyn VCpuIf) -> AxResult<usize> {
        // Perform bitwise operation to ensure the address is aligned to byte boundaries
        let addr = addr.as_usize() & 0xfff;

        // Match different read operations based on the width parameter
        match width {
            1 => {
                // Handle 1-byte read
                return self.handle_read8(addr, vcpu);
            }
            2 => {
                // Handle 2-byte read
                return self.handle_read16(addr, vcpu);
            }
            4 => {
                // Handle 4-byte read
                return self.handle_read32(addr, vcpu);
            }
            // Return success for unsupported widths without performing any operation
            _ => Ok(0),
        }
    }
    /// Handles write operations of different widths.
    ///
    /// This function performs a write operation based on the given physical address, width, and value.
    /// It first converts the physical address to a `usize` and applies a mask to ensure proper alignment.
    /// Then, depending on the width parameter, it calls the corresponding write handling function.
    ///
    /// Parameters:
    /// - `addr`: The physical address to write to.
    /// - `width`: The byte width of the data to be written (1, 2, 4 for 8-bit, 16-bit, and 32-bit data respectively).
    /// - `val`: The value to be written.
    fn handle_write(&self, addr: GuestPhysAddr, width: usize, val: usize, vcpu: &dyn VCpuIf) {
        // Convert the physical address to a `usize` and apply a mask to ensure proper alignment
        let addr = addr.as_usize() & 0xfff;

        // Depending on the width parameter, perform the corresponding write operation
        match width {
            1 => {
                // Handle 8-bit write operation
                self.handle_write8(addr, val, vcpu);
            }
            2 => {
                // Handle 16-bit write operation
                self.handle_write16(addr, val, vcpu);
            }
            4 => {
                // Handle 32-bit write operation
                self.handle_write32(addr, val, vcpu);
            }
            // For other width values, do nothing
            _ => {}
        }
    }
}
