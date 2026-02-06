//! GICv3 Redistributor (GICR) Register Definitions
//!
//! This module defines all registers for the ARM GICv3 Redistributor component
//! using the tock-registers library. All fields are public as required.
//!
//! 内存布局（2 个不连续的 4KB 区域）:
//! - GICR_BASE (0x0000-0x1FFF): 控制寄存器 + TYPER + WAKER + ID 寄存器
//! - GICR_SGI  (0x2000-0x3FFF): SGI/PPI 配置寄存器（中断 0-31）

use tock_registers::LocalRegisterCopy;
use tock_registers::register_bitfields;
use tock_registers::register_structs;
use tock_registers::registers::ReadWrite;

// ============================================================================
// Bit Field Definitions
// ============================================================================

register_bitfields! {
    u32,
    pub GICR_CTLR [
        /// Enable LPIs
        /// Controls whether the Redistributor supports LPIs
        EnableLPI OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        /// UWP - Wait for completion
        UWP OFFSET(3) NUMBITS(1) [],
        /// DPG1S - Disable Group 1 Secure (GICv3.1)
        DPG1S OFFSET(24) NUMBITS(1) [],
        /// DPG1NS - Disable Group 1 Non-secure (GICv3.1)
        DPG1NS OFFSET(25) NUMBITS(1) [],
        /// DPG0 - Disable Group 0 (GICv3.1)
        DPG0 OFFSET(26) NUMBITS(1) [],
        /// Reserved bits
        Reserved OFFSET(4) NUMBITS(20) [],
    ]
}

register_bitfields! {
    u64,
    pub GICR_TYPER [
        /// Affinity value 3 (most significant)
        AFF3 OFFSET(48) NUMBITS(16) [],
        /// Affinity value 2
        AFF2 OFFSET(32) NUMBITS(16) [],
        /// Affinity value 1
        AFF1 OFFSET(16) NUMBITS(16) [],
        /// Affinity value 0 (least significant)
        AFF0 OFFSET(0) NUMBITS(16) [],
        /// Processor Number
        ProcessorNumber OFFSET(8) NUMBITS(8) [],
        /// DPGS - Disable Processor Group 0 Support (GICv3.1)
        DPGS OFFSET(5) NUMBITS(1) [],
        /// Last - Last redistributor in the system
        Last OFFSET(4) NUMBITS(1) [
            NotLast = 0,
            Last = 1
        ],
        /// Reserved bits
        Reserved OFFSET(6) NUMBITS(58) [],
    ]
}

register_bitfields! {
    u32,
    pub GICR_WAKER [
        /// Processor sleep status
        Sleep OFFSET(2) NUMBITS(1) [
            Awake = 0,
            Sleeping = 1
        ],
        /// Child-asleep status
        ChildrenAsleep OFFSET(1) NUMBITS(1) [
            ChildAwake = 0,
            ChildAsleep = 1
        ],
        /// Processor sleep
        ProcessorSleep OFFSET(0) NUMBITS(1) [
            Awake = 0,
            Sleep = 1
        ],
        Reserved OFFSET(3) NUMBITS(29) [],
    ]
}

register_bitfields! {
    u32,
    pub GICR_IIDR [
        /// Implementer - JEDEC code
        Implementer OFFSET(0) NUMBITS(12) [],
        /// Revision
        Revision OFFSET(12) NUMBITS(4) [],
        /// Variant
        Variant OFFSET(16) NUMBITS(4) [],
        /// Product ID
        ProductID OFFSET(24) NUMBITS(8) [],
    ]
}

register_bitfields! {
    u32,
    pub GICR_IPRIORITYR [
        /// Priority for interrupt 0 (bits [7:0])
        Priority0 OFFSET(0) NUMBITS(8) [],
        /// Priority for interrupt 1 (bits [15:8])
        Priority1 OFFSET(8) NUMBITS(8) [],
        /// Priority for interrupt 2 (bits [23:16])
        Priority2 OFFSET(16) NUMBITS(8) [],
        /// Priority for interrupt 3 (bits [31:24])
        Priority3 OFFSET(24) NUMBITS(8) [],
    ]
}

register_bitfields! {
    u32,
    pub GICR_ICFGR [
        /// Configuration for interrupt 0 (bits [1:0])
        Config0 OFFSET(0) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 1 (bits [3:2])
        Config1 OFFSET(2) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 2 (bits [5:4])
        Config2 OFFSET(4) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 3 (bits [7:6])
        Config3 OFFSET(6) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 4 (bits [9:8])
        Config4 OFFSET(8) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 5 (bits [11:10])
        Config5 OFFSET(10) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 6 (bits [13:12])
        Config6 OFFSET(12) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 7 (bits [15:14])
        Config7 OFFSET(14) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 8 (bits [17:16])
        Config8 OFFSET(16) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 9 (bits [19:18])
        Config9 OFFSET(18) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 10 (bits [21:20])
        Config10 OFFSET(20) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 11 (bits [23:22])
        Config11 OFFSET(22) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 12 (bits [25:24])
        Config12 OFFSET(24) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 13 (bits [27:26])
        Config13 OFFSET(26) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 14 (bits [29:28])
        Config14 OFFSET(28) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 15 (bits [31:30])
        Config15 OFFSET(30) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
    ]
}

register_bitfields! {
    u32,
    pub GICR_ISENABLER [
        /// Set enable for interrupts [31:0]
        SetEnable OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICR_ICENABLER [
        /// Clear enable for interrupts [31:0]
        ClearEnable OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICR_ISPENDR [
        /// Set pending for interrupts [31:0]
        SetPending OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICR_ICPENDR [
        /// Clear pending for interrupts [31:0]
        ClearPending OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICR_ISACTIVER [
        /// Set active for interrupts [31:0]
        SetActive OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICR_ICACTIVER [
        /// Clear active for interrupts [31:0]
        ClearActive OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICR_IGROUPR [
        /// Group for interrupts [31:0]
        Group OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICR_IGRPMODR [
        /// Group modifier for interrupts [31:0]
        GroupModifier OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICR_PIDR4 [
        /// 4KB count
        SIZE OFFSET(0) NUMBITS(4) [],
        /// JEP106 continuation
        JEP106Cont OFFSET(4) NUMBITS(4) [],
        Reserved OFFSET(8) NUMBITS(24) [],
    ]
}

register_bitfields! {
    u32,
    pub GICR_PIDR [
        /// Part number
        PartNumber OFFSET(0) NUMBITS(12) [],
        /// JEP106 identification code
        JEP106 OFFSET(12) NUMBITS(8) [],
        /// Revision
        Revision OFFSET(20) NUMBITS(4) [],
        /// Customer modified
        CMOD OFFSET(24) NUMBITS(4) [],
        /// JEP106 continuation
        JEP106Cont OFFSET(28) NUMBITS(4) [],
    ]
}

register_bitfields! {
    u32,
    pub GICR_CIDR [
        /// Part number
        PartNumber OFFSET(0) NUMBITS(12) [],
        /// JEP106 identification code
        JEP106 OFFSET(12) NUMBITS(8) [],
        /// Revision
        Revision OFFSET(20) NUMBITS(4) [],
        /// Customer modified
        CMOD OFFSET(24) NUMBITS(4) [],
        /// JEP106 continuation
        JEP106Cont OFFSET(28) NUMBITS(4) [],
    ]
}

// ============================================================================
// Type Aliases
// ============================================================================
// 所有寄存器使用 ReadWrite，因为虚拟化环境需要设置初始值

/// GICR_CTLR register (RW)
pub type GicrCtlrRegister = ReadWrite<u32, GICR_CTLR::Register>;
pub type GicrCtlrLocal = LocalRegisterCopy<u32, GICR_CTLR::Register>;

/// GICR_TYPER register (64-bit, 硬件规范 RO，虚拟化用 RW 设置初始值)
pub type GicrTyperRegister = ReadWrite<u64, GICR_TYPER::Register>;
pub type GicrTyperLocal = LocalRegisterCopy<u64, GICR_TYPER::Register>;

/// GICR_WAKER register (RW)
pub type GicrWakerRegister = ReadWrite<u32, GICR_WAKER::Register>;
pub type GicrWakerLocal = LocalRegisterCopy<u32, GICR_WAKER::Register>;

/// GICR_IIDR register (硬件规范 RO，虚拟化用 RW 设置初始值)
pub type GicrIidrRegister = ReadWrite<u32, GICR_IIDR::Register>;
pub type GicrIidrLocal = LocalRegisterCopy<u32, GICR_IIDR::Register>;

/// GICR_IPRIORITYR register (RW)
pub type GicrIpriorityRegister = ReadWrite<u32, GICR_IPRIORITYR::Register>;
pub type GicrIpriorityLocal = LocalRegisterCopy<u32, GICR_IPRIORITYR::Register>;

/// GICR_ICFGR register (RW)
pub type GicrIcfgRegister = ReadWrite<u32, GICR_ICFGR::Register>;

/// GICR_ISENABLER (硬件规范 WO，虚拟化用 RW)
pub type GicrIsenablerRegister = ReadWrite<u32, GICR_ISENABLER::Register>;

/// GICR_ICENABLER (硬件规范 WO，虚拟化用 RW)
pub type GicrIcenablerRegister = ReadWrite<u32, GICR_ICENABLER::Register>;

/// GICR_ISPENDR (硬件规范 WO，虚拟化用 RW)
pub type GicrIspendrRegister = ReadWrite<u32, GICR_ISPENDR::Register>;

/// GICR_ICPENDR (硬件规范 WO，虚拟化用 RW)
pub type GicrIcpendrRegister = ReadWrite<u32, GICR_ICPENDR::Register>;

/// GICR_ISACTIVER (硬件规范 WO，虚拟化用 RW)
pub type GicrIsactiverRegister = ReadWrite<u32, GICR_ISACTIVER::Register>;

/// GICR_ICACTIVER (硬件规范 WO，虚拟化用 RW)
pub type GicrIcactiverRegister = ReadWrite<u32, GICR_ICACTIVER::Register>;

/// GICR_IGROUPR (RW)
pub type GicrIgroupRegister = ReadWrite<u32, GICR_IGROUPR::Register>;
pub type GicrIgroupLocal = LocalRegisterCopy<u32, GICR_IGROUPR::Register>;

/// GICR_IGRPMODR (RW)
pub type GicrIgrpmodRegister = ReadWrite<u32, GICR_IGRPMODR::Register>;
pub type GicrIgrpmodLocal = LocalRegisterCopy<u32, GICR_IGRPMODR::Register>;

/// GICR_PIDR/CIDR (硬件规范 RO，虚拟化用 RW 设置 GICv3 标识)
pub type GicrPidr4Register = ReadWrite<u32, GICR_PIDR4::Register>;
pub type GicrPidrRegister = ReadWrite<u32, GICR_PIDR::Register>;
pub type GicrCidrRegister = ReadWrite<u32, GICR_CIDR::Register>;

// ============================================================================
// Register Block Structures
// ============================================================================

/// GICR_BASE 区域 (0x0000-0x1FFF): 控制寄存器
register_structs! {
    #[allow(non_snake_case)]
    pub GicrBlockBase {
        // === 基础寄存器 (0x0000-0x001C) ===
        (0x0000 => pub CTLR: GicrCtlrRegister),
        (0x0004 => _reserved_0x0004),
        (0x0008 => pub IIDR: GicrIidrRegister),
        (0x000C => _reserved_0x000c),
        // TYPER 是 64 位寄存器，需要 8 字节对齐，放在 0x0010
        (0x0010 => pub TYPER: GicrTyperRegister),
        (0x0018 => pub WAKER: GicrWakerRegister),
        (0x001C => _reserved_0x001c),

        // === 保留区域 (0x001C-0x0FD0) ===
        (0x0FD0 => pub PIDR4: GicrPidr4Register),
        (0x0FD4 => _reserved_0_0fd4),
        (0x0FE0 => pub PIDR0: GicrPidrRegister),
        (0x0FE4 => pub PIDR1: GicrPidrRegister),
        (0x0FE8 => pub PIDR2: GicrPidrRegister),
        (0x0FEC => pub PIDR3: GicrPidrRegister),
        (0x0FF0 => pub CIDR0: GicrCidrRegister),
        (0x0FF4 => pub CIDR1: GicrCidrRegister),
        (0x0FF8 => pub CIDR2: GicrCidrRegister),
        (0x0FFC => pub CIDR3: GicrCidrRegister),

        (0x1000 => @END),
    }
}

/// GICR_SGI 区域 (0x0000-0x0FFF): SGI/PPI 寄存器（中断 0-31）
register_structs! {
    #[allow(non_snake_case)]
    pub GicrBlockSgi {
        // === 分组寄存器 (0x2000-0x2C04) ===
        (0x0000 => pub IGROUPR0: GicrIgroupRegister),
        (0x0004 => _reserved_0x0004),
        (0x0080 => _reserved_0x0080),
        (0x0100 => pub ISENABLER0: GicrIsenablerRegister),
        (0x0104 => pub ISENABLER1: GicrIsenablerRegister),
        (0x0108 => _reserved_0x0108),
        (0x0180 => pub ICENABLER0: GicrIcenablerRegister),
        (0x0184 => pub ICENABLER1: GicrIcenablerRegister),
        (0x0188 => _reserved_0x0188),
        (0x0200 => pub ISPENDR0: GicrIspendrRegister),
        (0x0204 => pub ISPENDR1: GicrIspendrRegister),
        (0x0208 => _reserved_0x0208),
        (0x0280 => pub ICPENDR0: GicrIcpendrRegister),
        (0x0284 => pub ICPENDR1: GicrIcpendrRegister),
        (0x0288 => _reserved_0x0288),
        (0x0300 => pub ISACTIVER0: GicrIsactiverRegister),
        (0x0304 => pub ISACTIVER1: GicrIsactiverRegister),
        (0x0308 => _reserved_0x0308),
        (0x0380 => pub ICACTIVER0: GicrIcactiverRegister),
        (0x0384 => pub ICACTIVER1: GicrIcactiverRegister),
        (0x0388 => _reserved_0_0388),
        (0x0400 => pub IPRIORITYR: [GicrIpriorityRegister; 8]),
        (0x0420 => _reserved_0_0420),
        (0x0C00 => pub ICFGR0: GicrIcfgRegister),
        (0x0C04 => pub ICFGR1: GicrIcfgRegister),
        (0x0C08 => _reserved_0_0c08),
        (0x0D00 => pub IGRPMODR0: GicrIgrpmodRegister),
        (0x0D04 => _reserved_0_0d04),

        // === ID 寄存器 (0x0FD0-0x0FFF) ===
        (0x0FD0 => pub PIDR4: GicrPidr4Register),
        (0x0FD4 => _reserved_0_0fd4),
        (0x0FE0 => pub PIDR0: GicrPidrRegister),
        (0x0FE4 => pub PIDR1: GicrPidrRegister),
        (0x0FE8 => pub PIDR2: GicrPidrRegister),
        (0x0FEC => pub PIDR3: GicrPidrRegister),
        (0x0FF0 => pub CIDR0: GicrCidrRegister),
        (0x0FF4 => pub CIDR1: GicrCidrRegister),
        (0x0FF8 => pub CIDR2: GicrCidrRegister),
        (0x0FFC => pub CIDR3: GicrCidrRegister),

        (0x1000 => @END),
    }
}
