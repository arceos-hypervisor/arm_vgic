//! Definitions for ITS commands.
//!
//! Commands introduced in GICv4 or later are not supported in this module.

/// The device ID used in ITS commands.
///
/// The actual supported bit width of this type depends on the GIC version and implementation, we
/// use a typically large enough value to accommodate most cases.
pub type DeviceID = u32;
/// The event ID used in ITS commands.
///
/// The actual supported bit width of this type depends on the GIC version and implementation, we
/// use a typically large enough value to accommodate most cases.
pub type EventID = u32;
/// The interrupt collection ID used in ITS commands.
///
/// The actual supported bit width of this type depends on the GIC version and implementation, we
/// use a typically large enough value to accommodate most cases.
pub type ICID = u16;
/// The physical interrupt ID used in ITS commands.
///
/// The actual supported bit width of this type depends on the GIC version and implementation, we
/// use a typically large enough value to accommodate most cases.
pub type PINTID = u32;

/// An its command (raw format).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ItsCommandRaw(pub [u64; 4]);

macro_rules! its_cmd_raw_getter_setter {
    (
        $getter_name:ident / $setter_name:ident / $desc:literal : $type:ty,
        at u64 $u64idx:literal from $base:literal len $length:literal
        $(shifted $shift:literal)?
    ) => {
        #[doc = concat!("Get the value of the ", $desc, " field.")]
        #[inline(always)]
        pub const fn $getter_name(&self) -> $type {
            const MASK: u64 = (1 << $length) - 1;
            let value = (self.0[$u64idx] >> $base) & MASK;
            $(
                let value = value << $shift;
            )?
            value as $type
        }

        #[doc = concat!("Set the value of the ", $desc, " field.")]
        #[inline(always)]
        pub const fn $setter_name(&mut self, value: $type) {
            const MASK: u64 = (1 << $length) - 1;
            let value = value as u64;
            $(
                let value = value >> $shift;
            )?
            self.0[$u64idx] = (self.0[$u64idx] & !(MASK << $base)) | ((value << $base) & (MASK << $base));
        }
    };
}

impl ItsCommandRaw {
    /// Creates a new `ItsCommandRaw` with all fields set to zero.
    pub fn nil() -> Self {
        Default::default()
    }

    its_cmd_raw_getter_setter!(get_id/set_id/"command identifier": u8,
        at u64 0 from 0 len 8);
    its_cmd_raw_getter_setter!(get_device_id/set_device_id/"device ID": DeviceID,
        at u64 0 from 32 len 32);
    its_cmd_raw_getter_setter!(get_event_id/set_event_id/"event ID": EventID,
        at u64 1 from 0 len 32);
    its_cmd_raw_getter_setter!(get_size/set_size/"size of ITT": u8,
        at u64 1 from 0 len 5);
    its_cmd_raw_getter_setter!(get_p_int_id/set_p_int_id/"physical interrupt ID": PINTID,
        at u64 1 from 32 len 32);
    its_cmd_raw_getter_setter!(get_icid/set_icid/"interrupt collection ID": ICID,
        at u64 2 from 0 len 16);
    its_cmd_raw_getter_setter!(get_rd_base1/set_rd_base1/"Redistributor base address": u64,
        at u64 2 from 16 len 36 shifted 16);
    its_cmd_raw_getter_setter!(get_itt_addr/set_itt_addr/"ITT base address": u64,
        at u64 2 from 8 len 44 shifted 8);
    its_cmd_raw_getter_setter!(get_v/set_v/"valid bit": u8,
        at u64 2 from 63 len 1);
    its_cmd_raw_getter_setter!(get_rd_base2/set_rd_base2/"Redistributor base address 2": u64,
        at u64 3 from 16 len 36 shifted 16);
}

/// An its command (decoded format).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItsCommand {
    /// This command translates the event defined by EventID and DeviceID into an ICID and pINTID,
    /// and instructs the appropriate Redistributor to remove the pending state.
    ///
    /// Format:
    /// - ID(0x04) @ byte 0
    /// - DeviceID @ byte 4
    /// - EventID @ byte 8
    /// - Res0 for others
    CLEAR {
        device_id: DeviceID,
        event_id: EventID,
    },
    /// Same as [`CLEAR`](GitsCmd::CLEAR). This command also removes the mapping of the DeviceID and
    /// EventID from the ITT, and ensures that incoming requests with a particular EventID are
    /// silently discarded. It also ensures that any caching in the Redistributors associated with a
    /// specific EventID is consistent with the configuration held in memory.
    ///
    /// Format:
    /// - ID (0x0F) @ byte 0
    /// - Others are same as [`CLEAR`](GitsCmd::CLEAR)
    DISCARD {
        /// The device ID.
        device_id: DeviceID,
        /// The event ID.
        event_id: EventID,
    },
    /// This command translates the event defined by EventID and DeviceID into either an ICID and
    /// pINTID, and instructs the appropriate Redistributor to set the interrupt pending.
    ///
    /// Format:
    /// - ID (0x03) @ byte 0
    /// - Others are same as [`CLEAR`](GitsCmd::CLEAR)
    INT {
        /// The device ID.
        device_id: DeviceID,
        /// The event ID.
        event_id: EventID,
    },
    /// This command specifies that the ITS must ensure that any caching in the Redistributors
    /// associated with the specified EventID is consistent with the LPI Configuration tables held
    /// in memory.
    ///
    /// Format:
    /// - ID (0x0C) @ byte 0
    /// - Others are same as [`CLEAR`](GitsCmd::CLEAR)
    INV {
        /// The device ID.
        device_id: DeviceID,
        /// The event ID.
        event_id: EventID,
    },
    /// This command specifies that the ITS must ensure any caching associated with the interrupt
    /// collection defined by ICID is consistent with the LPI Configuration tables held in memory
    /// for all Redistributors.
    ///
    /// Format:
    /// - ID (0x0D) @ byte 0
    /// - ICID @ byte 16
    /// - Res0 for others
    INVALL {
        /// The interrupt collection ID.
        icid: ICID,
    },
    /// This command maps the Collection table entry defined by ICID to the target Redistributor,
    /// defined by RDbase.
    ///
    /// Format:
    /// - ID (0x09) @ byte 0
    /// - ICID @ byte 16
    /// - RDbase @ byte 18 (shifted by 16 bits)
    /// - V @ bit 7, byte 23
    /// - Res0 for others
    MAPC {
        /// The interrupt collection ID.
        icid: ICID,
        /// The base address of the Redistributor.
        rd_base: u64,
        /// Valid bit, indicating whether the mapping is valid. When set to false, the mapping on
        /// this ICID is removed.
        valid: bool,
    },
    /// This command maps the Device table entry associated with DeviceID to its associated ITT,
    /// defined by ITT_addr and Size.
    ///
    /// Format:
    /// - ID (0x08) @ byte 0
    /// - DeviceID @ byte 4
    /// - Size @ byte 8
    /// - ITT_addr @ byte 17 (shifted by 8 bits)
    /// - V @ bit 7, byte 23
    /// - Res0 for others
    MAPD {
        /// The device ID.
        device_id: DeviceID,
        /// The number of bits of supported EventID entries in the ITT, minus 1.
        ///
        /// Only the lower 5 bits are used, so the maximum value is 31 (representing 2^(31 + 1)
        /// entries).
        size: u8,
        /// The base address of the ITT.
        itt_addr: u64,
        /// Valid bit, indicating whether the mapping is valid. When set to false, the mapping on
        /// this DeviceID is removed.
        valid: bool,
    },
    /// Same as [`MAPTI`](GitsCmd::MAPTI), but it uses the EventID as the pINTID.
    ///
    /// In other words, this command is equivalent to `MAPTI DeviceID, EventID, EventID, ICID`.
    ///
    /// Format:
    /// - ID (0x0B) @ byte 0
    /// - DeviceID @ byte 4
    /// - EventID @ byte 8
    /// - ICID @ byte 16
    /// - Res0 for others
    MAPI {
        /// The device ID.
        device_id: DeviceID,
        /// The event ID.
        event_id: EventID,
        /// The interrupt collection ID.
        icid: ICID,
    },
    /// This command maps the event defined by EventID and DeviceID to its associated ITE, defined
    /// by ICID and pINTID in the ITT associated with DeviceID.
    ///
    /// Format:
    /// - ID (0x0A) @ byte 0
    /// - DeviceID @ byte 4
    /// - EventID @ byte 8
    /// - pINTID @ byte 12
    /// - ICID @ byte 16
    /// - Res0 for others
    MAPTI {
        /// The device ID.
        device_id: DeviceID,
        /// The event ID.
        event_id: EventID,
        /// The physical interrupt ID.
        p_int_id: PINTID,
        /// The interrupt collection ID.
        icid: ICID,
    },
    /// This command instructs the Redistributor specified by RDbase1 to move all of its pending
    /// interrupts to the Redistributor specified by RDbase2.
    ///
    /// Format:
    /// - ID (0x0E) @ byte 0
    /// - RDbase1 @ byte 18 (shifted by 16 bits)
    /// - RDbase2 @ byte 26 (shifted by 16 bits)
    /// - Res0 for others
    MOVALL {
        /// The base address of the Redistributor 1.
        rd_base1: u64,
        /// The base address of the Redistributor 2.
        rd_base2: u64,
    },
    /// This command updates the ICID field in the ITT entry for the event defined by DeviceID and
    /// EventID. It also translates the event defined by EventID and DeviceID into an ICID and
    /// pINTID, and instructs the appropriate Redistributor to move the pending state, if it is set,
    /// of the interrupt to the Redistributor defined by the new ICID, and to update the ITE
    /// associated with the event to use the new ICID.
    ///
    /// Format:
    /// - ID (0x01) @ byte 0
    /// - DeviceID @ byte 4
    /// - EventID @ byte 8
    /// - ICID @ byte 16
    /// - Res0 for others
    MOVI {
        /// The device ID.
        device_id: DeviceID,
        /// The event ID.
        event_id: EventID,
        /// The interrupt collection ID.
        icid: ICID,
    },
    /// This command ensures all outstanding ITS operations associated with physical interrupts for
    /// the Redistributor specified by RDbase are globally observed before any further ITS commands
    /// are executed. Following the execution of a SYNC, the effects of all previous commands must
    /// apply to subsequent writes to GITS_TRANSLATER.
    ///
    /// Format:
    /// - ID (0x05) @ byte 0
    /// - RDbase @ byte 18 (shifted by 16 bits)
    /// - Res0 for others
    SYNC {
        /// The base address of the Redistributor.
        rd_base: u64,
    },
}

/// The type identifier for the ITS command [`ItsCommand::CLEAR`].
pub const ITS_CMD_TYPE_CLEAR: u8 = 0x04;
/// The type identifier for ITS command [`ItsCommand::DISCARD`].
pub const ITS_CMD_TYPE_DISCARD: u8 = 0x0F;
/// The type identifier for ITS command [`ItsCommand::INT`].
pub const ITS_CMD_TYPE_INT: u8 = 0x03;
/// The type identifier for ITS command [`ItsCommand::INV`].
pub const ITS_CMD_TYPE_INV: u8 = 0x0C;
/// The type identifier for ITS command [`ItsCommand::INVALL`].
pub const ITS_CMD_TYPE_INVALL: u8 = 0x0D;
/// The type identifier for ITS command [`ItsCommand::MAPC`].
pub const ITS_CMD_TYPE_MAPC: u8 = 0x09;
/// The type identifier for ITS command [`ItsCommand::MAPD`].
pub const ITS_CMD_TYPE_MAPD: u8 = 0x08;
/// The type identifier for ITS command [`ItsCommand::MAPI`].
pub const ITS_CMD_TYPE_MAPI: u8 = 0x0B;
/// The type identifier for ITS command [`ItsCommand::MAPTI`].
pub const ITS_CMD_TYPE_MAPTI: u8 = 0x0A;
/// The type identifier for ITS command [`ItsCommand::MOVALL`].
pub const ITS_CMD_TYPE_MOVALL: u8 = 0x0E;
/// The type identifier for ITS command [`ItsCommand::MOVI`].
pub const ITS_CMD_TYPE_MOVI: u8 = 0x01;
/// The type identifier for ITS command [`ItsCommand::SYNC`].
pub const ITS_CMD_TYPE_SYNC: u8 = 0x05;

impl ItsCommand {
    /// Converts the `ItsCommand` into its raw format, represented by [`ItsCommandRaw`].
    pub fn into_raw(&self) -> ItsCommandRaw {
        let mut result = ItsCommandRaw::nil();
        match self {
            ItsCommand::CLEAR {
                device_id,
                event_id,
            } => {
                result.set_id(ITS_CMD_TYPE_CLEAR);
                result.set_device_id(*device_id);
                result.set_event_id(*event_id);
            }
            ItsCommand::DISCARD {
                device_id,
                event_id,
            } => {
                result.set_id(ITS_CMD_TYPE_DISCARD);
                result.set_device_id(*device_id);
                result.set_event_id(*event_id);
            }
            ItsCommand::INT {
                device_id,
                event_id,
            } => {
                result.set_id(ITS_CMD_TYPE_INT);
                result.set_device_id(*device_id);
                result.set_event_id(*event_id);
            }
            ItsCommand::INV {
                device_id,
                event_id,
            } => {
                result.set_id(ITS_CMD_TYPE_INV);
                result.set_device_id(*device_id);
                result.set_event_id(*event_id);
            }
            ItsCommand::INVALL { icid } => {
                result.set_id(ITS_CMD_TYPE_INVALL);
                result.set_icid(*icid);
            }
            ItsCommand::MAPC {
                icid,
                rd_base,
                valid,
            } => {
                result.set_id(ITS_CMD_TYPE_MAPC);
                result.set_icid(*icid);
                result.set_rd_base1(*rd_base);
                result.set_v(*valid as u8);
            }
            ItsCommand::MAPD {
                device_id,
                size,
                itt_addr,
                valid,
            } => {
                result.set_id(ITS_CMD_TYPE_MAPD);
                result.set_device_id(*device_id);
                result.set_size(*size);
                result.set_itt_addr(*itt_addr);
                result.set_v(*valid as u8);
            }
            ItsCommand::MAPI {
                device_id,
                event_id,
                icid,
            } => {
                result.set_id(ITS_CMD_TYPE_MAPI);
                result.set_device_id(*device_id);
                result.set_event_id(*event_id);
                result.set_icid(*icid);
            }
            ItsCommand::MAPTI {
                device_id,
                event_id,
                p_int_id,
                icid,
            } => {
                result.set_id(ITS_CMD_TYPE_MAPTI);
                result.set_device_id(*device_id);
                result.set_event_id(*event_id);
                result.set_p_int_id(*p_int_id);
                result.set_icid(*icid);
            }
            ItsCommand::MOVALL { rd_base1, rd_base2 } => {
                result.set_id(ITS_CMD_TYPE_MOVALL);
                result.set_rd_base1(*rd_base1);
                result.set_rd_base2(*rd_base2);
            }
            ItsCommand::MOVI {
                device_id,
                event_id,
                icid,
            } => {
                result.set_id(ITS_CMD_TYPE_MOVI);
                result.set_device_id(*device_id);
                result.set_event_id(*event_id);
                result.set_icid(*icid);
            }
            ItsCommand::SYNC { rd_base } => {
                result.set_id(ITS_CMD_TYPE_SYNC);
                result.set_rd_base1(*rd_base);
            }
        }

        result
    }

    /// Creates an `ItsCommand` from its raw format, represented by [`ItsCommandRaw`].
    pub fn from_raw(raw: ItsCommandRaw) -> Result<Self, ItsCommandRaw> {
        match raw.get_id() {
            ITS_CMD_TYPE_CLEAR => Ok(ItsCommand::CLEAR {
                device_id: raw.get_device_id(),
                event_id: raw.get_event_id(),
            }),
            ITS_CMD_TYPE_DISCARD => Ok(ItsCommand::DISCARD {
                device_id: raw.get_device_id(),
                event_id: raw.get_event_id(),
            }),
            ITS_CMD_TYPE_INT => Ok(ItsCommand::INT {
                device_id: raw.get_device_id(),
                event_id: raw.get_event_id(),
            }),
            ITS_CMD_TYPE_INV => Ok(ItsCommand::INV {
                device_id: raw.get_device_id(),
                event_id: raw.get_event_id(),
            }),
            ITS_CMD_TYPE_INVALL => Ok(ItsCommand::INVALL {
                icid: raw.get_icid(),
            }),
            ITS_CMD_TYPE_MAPC => Ok(ItsCommand::MAPC {
                icid: raw.get_icid(),
                rd_base: raw.get_rd_base1(),
                valid: raw.get_v() != 0,
            }),
            ITS_CMD_TYPE_MAPD => Ok(ItsCommand::MAPD {
                device_id: raw.get_device_id(),
                size: raw.get_size(),
                itt_addr: raw.get_itt_addr(),
                valid: raw.get_v() != 0,
            }),
            ITS_CMD_TYPE_MAPI => Ok(ItsCommand::MAPI {
                device_id: raw.get_device_id(),
                event_id: raw.get_event_id(),
                icid: raw.get_icid(),
            }),
            ITS_CMD_TYPE_MAPTI => Ok(ItsCommand::MAPTI {
                device_id: raw.get_device_id(),
                event_id: raw.get_event_id(),
                p_int_id: raw.get_p_int_id(),
                icid: raw.get_icid(),
            }),
            ITS_CMD_TYPE_MOVALL => Ok(ItsCommand::MOVALL {
                rd_base1: raw.get_rd_base1(),
                rd_base2: raw.get_rd_base2(),
            }),
            ITS_CMD_TYPE_MOVI => Ok(ItsCommand::MOVI {
                device_id: raw.get_device_id(),
                event_id: raw.get_event_id(),
                icid: raw.get_icid(),
            }),
            ITS_CMD_TYPE_SYNC => Ok(ItsCommand::SYNC {
                rd_base: raw.get_rd_base1(),
            }),
            _ => Err(raw),
        }
    }
}

impl From<ItsCommand> for ItsCommandRaw {
    fn from(cmd: ItsCommand) -> Self {
        cmd.into_raw()
    }
}

impl TryFrom<ItsCommandRaw> for ItsCommand {
    type Error = ItsCommandRaw;

    fn try_from(raw: ItsCommandRaw) -> Result<Self, Self::Error> {
        Self::from_raw(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_raw_builder_mapd() {
        let mut cmd = ItsCommandRaw::nil();
        cmd.set_id(0x08); // Set command type to MAPD
        cmd.set_device_id(0x12345678); // Set device ID
        cmd.set_size(0x1F); // Set size of ITT to 31 (2^(31 + 1) entries)
        cmd.set_itt_addr(0x1000_0000); // Set ITT base address
        cmd.set_v(1); // Set valid bit

        assert_eq!(cmd.get_id(), 0x08);
        assert_eq!(cmd.get_device_id(), 0x12345678);
        assert_eq!(cmd.get_size(), 0x1F);
        assert_eq!(cmd.get_itt_addr(), 0x1000_0000);
        assert_eq!(cmd.get_v(), 1);

        assert_eq!(cmd.0[0], 0x1234_5678_0000_0008);
        assert_eq!(cmd.0[1], 0x0000_0000_0000_001F);
        assert_eq!(cmd.0[2], 0x8000_0000_1000_0000);
        assert_eq!(cmd.0[3], 0x0000_0000_0000_0000);
    }

    #[test]
    fn test_command_raw_builder_mapc() {
        let mut cmd = ItsCommandRaw::nil();
        cmd.set_id(0x09); // Set command type to MAPC
        cmd.set_icid(0x1234); // Set ICID
        cmd.set_rd_base1(0x2000_0000); // Set Redistributor base address 1
        cmd.set_v(1); // Set valid bit

        assert_eq!(cmd.get_id(), 0x09);
        assert_eq!(cmd.get_icid(), 0x1234);
        assert_eq!(cmd.get_rd_base1(), 0x2000_0000);
        assert_eq!(cmd.get_v(), 1);

        assert_eq!(cmd.0[0], 0x0000_0000_0000_0009);
        assert_eq!(cmd.0[1], 0x0000_0000_0000_0000);
        assert_eq!(cmd.0[2], 0x8000_0000_2000_1234);
        assert_eq!(cmd.0[3], 0x0000_0000_0000_0000);
    }

    #[test]
    fn test_command_raw_builder_mapti() {
        let mut cmd = ItsCommandRaw::nil();
        cmd.set_id(0x0A); // Set command type to MAPTI
        cmd.set_device_id(0x8765_4321); // Set device ID
        cmd.set_event_id(0x1234_5678); // Set event ID
        cmd.set_p_int_id(0x9ABC_DEF0); // Set physical interrupt ID
        cmd.set_icid(0x5678); // Set ICID

        assert_eq!(cmd.get_id(), 0x0A);
        assert_eq!(cmd.get_device_id(), 0x8765_4321);
        assert_eq!(cmd.get_event_id(), 0x1234_5678);
        assert_eq!(cmd.get_p_int_id(), 0x9ABC_DEF0);
        assert_eq!(cmd.get_icid(), 0x5678);

        assert_eq!(cmd.0[0], 0x8765_4321_0000_000A);
        assert_eq!(cmd.0[1], 0x9ABC_DEF0_1234_5678);
        assert_eq!(cmd.0[2], 0x0000_0000_0000_5678);
        assert_eq!(cmd.0[3], 0x0000_0000_0000_0000);
    }

    #[test]
    fn test_command_raw_builder_movall() {
        let mut cmd = ItsCommandRaw::nil();
        cmd.set_id(0x0E); // Set command type to MOVALL
        cmd.set_rd_base1(0x10_3000_0000); // Set Redistributor base address 1
        cmd.set_rd_base2(0x10_4000_0000); // Set Redistributor base address 2

        assert_eq!(cmd.get_id(), 0x0E);
        assert_eq!(cmd.get_rd_base1(), 0x10_3000_0000);
        assert_eq!(cmd.get_rd_base2(), 0x10_4000_0000);

        assert_eq!(cmd.0[0], 0x0000_0000_0000_000E);
        assert_eq!(cmd.0[1], 0x0000_0000_0000_0000);
        assert_eq!(cmd.0[2], 0x0000_0010_3000_0000);
        assert_eq!(cmd.0[3], 0x0000_0010_4000_0000);
    }

    #[test]
    fn test_conversion() {
        /// Helper function to test conversion from `ItsCommand` to `ItsCommandRaw` and back.
        fn test_conversion_to_and_back(cmd: ItsCommand, expected_raw: [u64; 4]) {
            let raw: ItsCommandRaw = cmd.into();
            assert_eq!(raw.0, expected_raw);

            let cmd_converted: ItsCommand = raw.try_into().expect("Failed to convert back");
            assert_eq!(cmd_converted, cmd);
        }

        // test CLEAR command
        test_conversion_to_and_back(
            ItsCommand::CLEAR {
                device_id: 0x1234_5678,
                event_id: 0x9ABC_DEF0,
            },
            [
                0x1234_5678_0000_0004,
                0x0000_0000_9ABC_DEF0,
                0x0000_0000_0000_0000,
                0x0000_0000_0000_0000,
            ],
        );

        // test DISCARD command
        test_conversion_to_and_back(
            ItsCommand::DISCARD {
                device_id: 0x1234_5678,
                event_id: 0x9ABC_DEF0,
            },
            [
                0x1234_5678_0000_000F,
                0x0000_0000_9ABC_DEF0,
                0x0000_0000_0000_0000,
                0x0000_0000_0000_0000,
            ],
        );

        // test INT command
        test_conversion_to_and_back(
            ItsCommand::INT {
                device_id: 0x1234_5678,
                event_id: 0x9ABC_DEF0,
            },
            [
                0x1234_5678_0000_0003,
                0x0000_0000_9ABC_DEF0,
                0x0000_0000_0000_0000,
                0x0000_0000_0000_0000,
            ],
        );

        // test INV command
        test_conversion_to_and_back(
            ItsCommand::INV {
                device_id: 0x1234_5678,
                event_id: 0x9ABC_DEF0,
            },
            [
                0x1234_5678_0000_000C,
                0x0000_0000_9ABC_DEF0,
                0x0000_0000_0000_0000,
                0x0000_0000_0000_0000,
            ],
        );

        // test INVALL command
        test_conversion_to_and_back(
            ItsCommand::INVALL { icid: 0x1234 },
            [
                0x0000_0000_0000_000D,
                0x0000_0000_0000_0000,
                0x0000_0000_0000_1234,
                0x0000_0000_0000_0000,
            ],
        );

        // test MAPC command
        test_conversion_to_and_back(
            ItsCommand::MAPC {
                icid: 0x1234,
                rd_base: 0x9988_7766_0000,
                valid: true,
            },
            [
                0x0000_0000_0000_0009,
                0x0000_0000_0000_0000,
                0x8000_9988_7766_1234,
                0x0000_0000_0000_0000,
            ],
        );

        // test MAPD command
        test_conversion_to_and_back(
            ItsCommand::MAPD {
                device_id: 0x1234_5678,
                size: 0x1F,
                itt_addr: 0x5544_3322_0000,
                valid: true,
            },
            [
                0x1234_5678_0000_0008,
                0x0000_0000_0000_001F,
                0x8000_5544_3322_0000,
                0x0000_0000_0000_0000,
            ],
        );

        // test MAPI command
        test_conversion_to_and_back(
            ItsCommand::MAPI {
                device_id: 0x1234_5678,
                event_id: 0x9ABC_DEF0,
                icid: 0x1234,
            },
            [
                0x1234_5678_0000_000B,
                0x0000_0000_9ABC_DEF0,
                0x0000_0000_0000_1234,
                0x0000_0000_0000_0000,
            ],
        );

        // test MAPTI command
        test_conversion_to_and_back(
            ItsCommand::MAPTI {
                device_id: 0x1234_5678,
                event_id: 0x9ABC_DEF0,
                p_int_id: 0x1122_3344,
                icid: 0x1234,
            },
            [
                0x1234_5678_0000_000A,
                0x1122_3344_9ABC_DEF0,
                0x0000_0000_0000_1234,
                0x0000_0000_0000_0000,
            ],
        );

        // test MOVALL command
        test_conversion_to_and_back(
            ItsCommand::MOVALL {
                rd_base1: 0x9988_7766_0000,
                rd_base2: 0xDDCC_BBAA_0000,
            },
            [
                0x0000_0000_0000_000E,
                0x0000_0000_0000_0000,
                0x0000_9988_7766_0000,
                0x0000_DDCC_BBAA_0000,
            ],
        );

        // test MOVI command
        test_conversion_to_and_back(
            ItsCommand::MOVI {
                device_id: 0x1234_5678,
                event_id: 0x9ABC_DEF0,
                icid: 0x1234,
            },
            [
                0x1234_5678_0000_0001,
                0x0000_0000_9ABC_DEF0,
                0x0000_0000_0000_1234,
                0x0000_0000_0000_0000,
            ],
        );

        // test SYNC command
        test_conversion_to_and_back(
            ItsCommand::SYNC {
                rd_base: 0x9988_7766_0000,
            },
            [
                0x0000_0000_0000_0005,
                0x0000_0000_0000_0000,
                0x0000_9988_7766_0000,
                0x0000_0000_0000_0000,
            ],
        );
    }
}
