#![allow(dead_code)] // allow unused constants

use tock_registers::{
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

/// The offset of the `GITS_CTRL` register in the GITS register range.
pub const GITS_CTRL: usize = 0x0000;
/// The offset of the `GITS_IIDR` register in the GITS register range.
pub const GITS_IIDR: usize = 0x0004;
/// The offset of the `GITS_TYPER` register in the GITS register range.
pub const GITS_TYPER: usize = 0x0008;
/// The end of the `GITS_TYPER` register.
pub const GITS_TYPER_END: usize = 0x0010;
/// The offset of the `GITS_MPAMIDR` register in the GITS register range.
pub const GITS_MPAMIDR: usize = 0x0010; // read-only
/// The offset of the `GITS_PARTIDR` register in the GITS register range.
pub const GITS_PARTIDR: usize = 0x0014;
/// The offset of the `GITS_MPIDR` register in the GITS register range.
pub const GITS_MPIDR: usize = 0x0018;
/// The offset of the `GITS_STATUSR` register in the GITS register range.
pub const GITS_STATUSR: usize = 0x0040;
/// The offset of the `GITS_UMSIR` register in the GITS register range.
pub const GITS_UMSIR: usize = 0x0048;
/// The offset of the `GITS_CBASER` register in the GITS register range.
pub const GITS_CBASER: usize = 0x0080;
/// The offset of the `GITS_CWRITER` register in the GITS register range.
pub const GITS_CWRITER: usize = 0x0088;
/// The offset of the `GITS_CREADR` register in the GITS register range.
pub const GITS_CREADR: usize = 0x0090;
/// The end of the `GITS_CREADR` register.
pub const GITS_CREADR_END: usize = 0x0098;
/// The offset of the `GITS_BASER` registers in the GITS register range.
pub const GITS_BASER: usize = 0x0100;
/// The offset of the `GITS_BASER` register for the device table.
///
/// By convention, the 0th entry in the `GITS_BASER` array is used for the device table.
pub const GITS_DT_BASER: usize = GITS_BASER + GITS_DT_INDEX * GITS_BASER_SIZE;
/// The offset of the `GITS_BASER` register for the interrupt collection table.
///
/// By convention, the 1st entry in the `GITS_BASER` array is used for the interrupt collection table.
pub const GITS_CT_BASER: usize = GITS_BASER + GITS_CT_INDEX * GITS_BASER_SIZE;
/// The end of the `GITS_BASER` register range.
pub const GITS_BASER_END: usize = 0x0140;
/// The end of the first frame (the control frame) in the GITS register range.
pub const GITS_CONTROL_FRAME_END: usize = 0x10000;

/// The index of the device table in the `GITS_BASER` array.
///
/// By convention, the 0th entry in the `GITS_BASER` array is used for the device table.
pub const GITS_DT_INDEX: usize = 0;
/// The index of the interrupt collection table in the `GITS_BASER` array.
///
/// By convention, the 1st entry in the `GITS_BASER` array is used for the interrupt collection table.
pub const GITS_CT_INDEX: usize = 1;
/// The size of each entry in the `GITS_BASER` array, in bytes.
pub const GITS_BASER_SIZE: usize = 0x8;

register_structs! {
    /// The ITS registers.
    pub GitsRegs {
        /// ITS control register.
        (GITS_CTRL => pub ctrl: ReadWrite<u32, CTLR::Register>),
        /// ITS implementation identification register.
        (GITS_IIDR => pub iidr: ReadOnly<u32>),
        /// ITS type register.
        (GITS_TYPER => pub typer: ReadOnly<u64, TYPER::Register>),
        (GITS_TYPER_END => _res0), // We ignore mpamidr, partidr, ...
        /// ITS Command Queue descriptor.
        (GITS_CBASER => pub cbaser: ReadWrite<u64, CBASER::Register>),
        /// ITS Command Queue write pointer.
        (GITS_CWRITER => pub cwriter: ReadWrite<u64>),
        /// ITS Command Queue read pointer.
        (GITS_CREADR => pub creadr: ReadWrite<u64>),
        (GITS_CREADR_END => _res1),
        /// ITS Table descriptors.
        (GITS_BASER => pub baser: [ReadWrite<u64, BASER::Register>; 8]),
        (GITS_BASER_END => _res2),
        (GITS_CONTROL_FRAME_END => @END),
    }
}

register_bitfields! {
    u32,
    /// Fields for `GITS_CTRL`
    pub CTLR [
        /// Whether the GITS is in quiescent state and can be powered down or not.
        QUIESCENT OFFSET(31) NUMBITS(1) [
            NotQuiescent = 0,
            Quiescent = 1
        ],
        /// Whether the GITS is enabled or not.
        ENABLED OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ]
    ]
}

register_bitfields! {
    u64,

    /// Fields for `GITS_TYPER`
    pub TYPER [
        /// The number of bytes per entry in the ITT, minus 1.
        ITT_ENTRY_SIZE OFFSET(4) NUMBITS(4) [],
    ],

    /// Fields for `GITS_BASER`
    pub BASER [
        /// Whether the table is valid or not.
        VALID OFFSET(63) NUMBITS(1) [
            Valid = 1,
            Invalid = 0
        ],
        /// Whether the table is an indirect table or not.
        INDIRECT OFFSET(62) NUMBITS(1) [
            Direct = 0,
            Indirect = 1
        ],
        /// Inner cacheability attributes.
        INNER_CACHE OFFSET(59) NUMBITS(3) [
            DevicenGnRnE = 0b000,
            InnerNonCachable = 0b001,
            InnerReadAllocateWriteThrough = 0b010,
            InnerReadAllocateWriteBack = 0b011,
            InnerWriteAllocateWriteThrough = 0b100,
            InnerWriteAllocateWriteBack = 0b101,
            InnerWriteThrough = 0b110,
            InnerWriteBack = 0b111
        ],
        /// The expected table type of this entry.
        TYPE OFFSET(56) NUMBITS(3) [
            Unimplemented = 0b000,
            Devices = 0b001,
            VirtualPEs = 0b010,
            Collections = 0b100,
        ],
        /// Outer cacheability attributes.
        OUTER_CACHE OFFSET(53) NUMBITS(3) [
            SameAsInner = 0b000,
            OuterNonCachable = 0b001,
            OuterReadAllocateWriteThrough = 0b010,
            OuterReadAllocateWriteBack = 0b011,
            OuterWriteAllocateWriteThrough = 0b100,
            OuterWriteAllocateWriteBack = 0b101,
            OuterWriteThrough = 0b110,
            OuterWriteBack = 0b111
        ],
        /// The number of bytes per table entry, minus 1.
        ENTRY_SIZE OFFSET(48) NUMBITS(5) [],
        /// The physical address of the table, shifted right by 12 bits.
        PHYSICAL_ADDRESS OFFSET(12) NUMBITS(36) [],
        /// Shareability attributes.
        SHAREABILITY OFFSET(10) NUMBITS(2) [
            NonShareable = 0b00,
            InnerShareable = 0b01,
            OuterShareable = 0b10,
        ],
        /// The size of the pages allocated for this table.
        PAGE_SIZE OFFSET(8) NUMBITS(2) [
            Page4KiB = 0b00,
            Page16KiB = 0b01,
            Page64KiB = 0b10,
        ],
        /// The number of pages allocated for this table, minus 1.
        SIZE OFFSET(0) NUMBITS(8) []
    ],

    /// Fields for `GITS_CBASER`
    pub CBASER [
        /// Whether the command queue is valid or not.
        VALID OFFSET(63) NUMBITS(1) [
            Valid = 1,
            Invalid = 0
        ],
        /// Inner cacheability attributes.
        INNER_CACHE OFFSET(59) NUMBITS(3) [
            DevicenGnRnE = 0b000,
            InnerNonCachable = 0b001,
            InnerReadAllocateWriteThrough = 0b010,
            InnerReadAllocateWriteBack = 0b011,
            InnerWriteAllocateWriteThrough = 0b100,
            InnerWriteAllocateWriteBack = 0b101,
            InnerWriteThrough = 0b110,
            InnerWriteBack = 0b111
        ],
        /// Outer cacheability attributes.
        OUTER_CACHE OFFSET(53) NUMBITS(3) [
        SameAsInner = 0b000,
            OuterNonCachable = 0b001,
            OuterReadAllocateWriteThrough = 0b010,
            OuterReadAllocateWriteBack = 0b011,
            OuterWriteAllocateWriteThrough = 0b100,
            OuterWriteAllocateWriteBack = 0b101,
            OuterWriteThrough = 0b110,
            OuterWriteBack = 0b111
        ],
        /// The physical address of the command queue, shifted right by 12 bits. The lowest 4 bits of this field are
        /// reserved and must be 0.
        PHYSICAL_ADDRESS OFFSET(12) NUMBITS(40) [],
        /// Shareability attributes.
        SHAREABILITY OFFSET(10) NUMBITS(2) [
            NonShareable = 0b00,
            InnerShareable = 0b01,
            OuterShareable = 0b10,
        ],
        /// The number of 4-KiB frames allocated for this table, minus 1.
        SIZE OFFSET(0) NUMBITS(8) [],
    ],
}
