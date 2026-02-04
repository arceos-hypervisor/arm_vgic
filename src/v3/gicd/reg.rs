//! GICv3 Distributor (GICD) Register Definitions
//!
//! This module defines all registers for the ARM GICv3 Distributor component
//! using the tock-registers library. All fields are public as required.

use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::register_bitfields;
use tock_registers::LocalRegisterCopy;

// ============================================================================
// Bit Field Definitions
// ============================================================================

register_bitfields! {
    u32,
    pub GICD_CTLR [
        /// Enable Group 1 Non-secure interrupts
        /// When ARE_NS==1, this bit enables Group 1 Non-secure interrupts
        EnableGrp1NS OFFSET(2) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        /// Enable Group 0 interrupts
        /// When ARE_S==1, this bit enables Group 0 interrupts
        EnableGrp0 OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        /// Affinity routing for Non-secure state
        /// 1 = Affinity routing enabled for Non-secure state
        /// 0 = Legacy operation
        ARE_NS OFFSET(5) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        /// Affinity routing for Secure state
        /// 1 = Affinity routing enabled for Secure state
        /// 0 = Legacy operation
        ARE_S OFFSET(4) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],
        /// Disable Security
        /// 0 = Two security states
        /// 1 = Single security state (Non-secure only)
        DS OFFSET(6) NUMBITS(1) [
            SecureEnabled = 0,
            SecurityDisabled = 1
        ],
        /// RSS - Common
        RSS OFFSET(3) NUMBITS(1) [],
        /// Reserved bits
        Reserved OFFSET(7) NUMBITS(25) [],
    ]
}

register_bitfields! {
    u32,
    pub GICD_TYPER [
        /// Number of implemented interrupt lines
        /// ITLinesNumber = (N - 1), where N is number of interrupts
        ITLinesNumber OFFSET(0) NUMBITS(5) [],
        /// Number of CPUs
        CPUNumber OFFSET(5) NUMBITS(3) [],
        /// MBIS - Maintains backward compatibility
        MBIS OFFSET(16) NUMBITS(1) [
            No = 0,
            Yes = 1
        ],
        /// Security extension
        SecurityExtn OFFSET(10) NUMBITS(1) [
            NotImpl = 0,
            Impl = 1
        ],
        /// LPIS - LPI support
        LPIS OFFSET(17) NUMBITS(1) [
            NotImpl = 0,
            Impl = 1
        ],
        /// IDbits - Number of identifier bits
        IDbits OFFSET(19) NUMBITS(5) [],
        /// A3V - Affinity3 valid
        A3V OFFSET(22) NUMBITS(1) [
            NotValid = 0,
            Valid = 1
        ],
        /// No 1-N - Distributor not 1-of-N
        No1ofN OFFSET(23) NUMBITS(1) [
            OneOfN = 0,
            NotOneOfN = 1
        ],
        Reserved OFFSET(24) NUMBITS(8) [],
    ]
}

register_bitfields! {
    u32,
    pub GICD_IIDR [
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
    pub GICD_IGROUPR [
        /// Interrupt group for bits 0-31
        /// 0 = Group 0, 1 = Group 1
        Group OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICD_IGRPMODR [
        /// Group modifier for each interrupt
        /// 0 = Group 0 or Non-secure Group 1
        /// 1 = Secure Group 1
        GroupModifier OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICD_ISENABLER [
        /// Set enable for 32 interrupts
        /// Write 1 to enable, Write 0 has no effect
        SetEnable OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICD_ICENABLER [
        /// Clear enable for 32 interrupts
        /// Write 1 to disable, Write 0 has no effect
        ClearEnable OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICD_ISPENDR [
        /// Set pending for 32 interrupts
        SetPending OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICD_ICPENDR [
        /// Clear pending for 32 interrupts
        ClearPending OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICD_ISACTIVER [
        /// Set active for 32 interrupts
        SetActive OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICD_ICACTIVER [
        /// Clear active for 32 interrupts
        ClearActive OFFSET(0) NUMBITS(32) []
    ]
}

register_bitfields! {
    u32,
    pub GICD_IPRIORITYR [
        /// Priority for interrupt n (bits 0-7)
        Priority0 OFFSET(0) NUMBITS(8) [],
        /// Priority for interrupt n+1 (bits 8-15)
        Priority1 OFFSET(8) NUMBITS(8) [],
        /// Priority for interrupt n+2 (bits 16-23)
        Priority2 OFFSET(16) NUMBITS(8) [],
        /// Priority for interrupt n+3 (bits 24-31)
        Priority3 OFFSET(24) NUMBITS(8) [],
    ]
}

register_bitfields! {
    u32,
    pub GICD_ICFGR [
        /// Configuration for interrupt 0
        Config0 OFFSET(0) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 1
        Config1 OFFSET(2) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 2
        Config2 OFFSET(4) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 3
        Config3 OFFSET(6) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 4
        Config4 OFFSET(8) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 5
        Config5 OFFSET(10) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 6
        Config6 OFFSET(12) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 7
        Config7 OFFSET(14) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 8
        Config8 OFFSET(16) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 9
        Config9 OFFSET(18) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 10
        Config10 OFFSET(20) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 11
        Config11 OFFSET(22) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 12
        Config12 OFFSET(24) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 13
        Config13 OFFSET(26) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 14
        Config14 OFFSET(28) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
        /// Configuration for interrupt 15
        Config15 OFFSET(30) NUMBITS(2) [
            Level = 0b00,
            Edge = 0b10
        ],
    ]
}

register_bitfields! {
    u64,
    pub GICD_IROUTER [
        /// Interrupt Routing Mode
        /// 0 = Specific PE
        /// 1 = Any PE (affinity values ignored)
        IRM OFFSET(31) NUMBITS(1) [
            SpecificPE = 0,
            AnyPE = 1
        ],
        /// Affinity level 3
        AFF3 OFFSET(48) NUMBITS(16) [],
        /// Affinity level 2
        AFF2 OFFSET(32) NUMBITS(16) [],
        /// Affinity level 1
        AFF1 OFFSET(16) NUMBITS(16) [],
        /// Affinity level 0
        AFF0 OFFSET(0) NUMBITS(16) [],
        Reserved OFFSET(40) NUMBITS(24) [],
    ]
}

register_bitfields! {
    u32,
    pub GICD_SGIR [
        /// Target List Filter
        /// 00 = Send to all PE(s) specified in Target List
        /// 01 = Send to all PE(s) except requesting PE
        /// 10 = Send to requesting PE only
        /// 11 = Reserved
        TargetListFilter OFFSET(24) NUMBITS(2) [
            AllSpecified = 0b00,
            AllExceptRequester = 0b01,
            RequesterOnly = 0b10
        ],
        /// CPUTargetList - bit field of target CPUs
        CPUTargetList OFFSET(16) NUMBITS(8) [],
        /// NSATT - Non-secure Target
        NSATT OFFSET(15) NUMBITS(1) [
            Secure = 0,
            NonSecure = 1
        ],
        Reserved OFFSET(4) NUMBITS(11) [],
        /// SGIINTID - Interrupt ID (0-15 for SGIs)
        SGIINTID OFFSET(0) NUMBITS(4) [],
    ]
}

register_bitfields! {
    u32,
    pub GICD_NSACR [
        /// Non-secure access control for interrupt 0 (2 bits)
        NsAccess0 OFFSET(0) NUMBITS(2) [],
        /// Non-secure access control for interrupt 1 (2 bits)
        NsAccess1 OFFSET(2) NUMBITS(2) [],
        /// Non-secure access control for interrupt 2 (2 bits)
        NsAccess2 OFFSET(4) NUMBITS(2) [],
        /// Non-secure access control for interrupt 3 (2 bits)
        NsAccess3 OFFSET(6) NUMBITS(2) [],
        /// Non-secure access control for interrupt 4 (2 bits)
        NsAccess4 OFFSET(8) NUMBITS(2) [],
        /// Non-secure access control for interrupt 5 (2 bits)
        NsAccess5 OFFSET(10) NUMBITS(2) [],
        /// Non-secure access control for interrupt 6 (2 bits)
        NsAccess6 OFFSET(12) NUMBITS(2) [],
        /// Non-secure access control for interrupt 7 (2 bits)
        NsAccess7 OFFSET(14) NUMBITS(2) [],
        /// Non-secure access control for interrupt 8 (2 bits)
        NsAccess8 OFFSET(16) NUMBITS(2) [],
        /// Non-secure access control for interrupt 9 (2 bits)
        NsAccess9 OFFSET(18) NUMBITS(2) [],
        /// Non-secure access control for interrupt 10 (2 bits)
        NsAccess10 OFFSET(20) NUMBITS(2) [],
        /// Non-secure access control for interrupt 11 (2 bits)
        NsAccess11 OFFSET(22) NUMBITS(2) [],
        /// Non-secure access control for interrupt 12 (2 bits)
        NsAccess12 OFFSET(24) NUMBITS(2) [],
        /// Non-secure access control for interrupt 13 (2 bits)
        NsAccess13 OFFSET(26) NUMBITS(2) [],
        /// Non-secure access control for interrupt 14 (2 bits)
        NsAccess14 OFFSET(28) NUMBITS(2) [],
        /// Non-secure access control for interrupt 15 (2 bits)
        NsAccess15 OFFSET(30) NUMBITS(2) [],
    ]
}

register_bitfields! {
    u32,
    pub GICD_PIDR4 [
        /// 4KB count
        SIZE OFFSET(0) NUMBITS(4) [],
        /// JEP106 continuation code
        JEP106Cont OFFSET(4) NUMBITS(4) [],
        Reserved OFFSET(8) NUMBITS(24) [],
    ]
}

register_bitfields! {
    u32,
    pub GICD_PIDR [
        /// Part number
        PartNumber OFFSET(0) NUMBITS(12) [],
        /// JEP106 identification code
        JEP106 OFFSET(12) NUMBITS(8) [],
        /// Revision
        Revision OFFSET(20) NUMBITS(4) [],
        /// Customer modified
        CMOD OFFSET(24) NUMBITS(4) [],
        /// JEP106 continuation code
        JEP106Cont OFFSET(28) NUMBITS(4) [],
    ]
}

register_bitfields! {
    u32,
    pub GICD_CIDR [
        /// Part number
        PartNumber OFFSET(0) NUMBITS(12) [],
        /// JEP106 identification code
        JEP106 OFFSET(12) NUMBITS(8) [],
        /// Revision
        Revision OFFSET(20) NUMBITS(4) [],
        /// Customer modified
        CMOD OFFSET(24) NUMBITS(4) [],
        /// JEP106 continuation code
        JEP106Cont OFFSET(28) NUMBITS(4) [],
    ]
}

// ============================================================================
// Type Aliases for Register Access
// ============================================================================

/// GICD_CTLR register using MMIO
/// - Offset: 0x000
/// - Reset: 0x00000000
pub type GicdCtlrRegister = ReadWrite<u32, GICD_CTLR::Register>;

/// Local copy of GICD_CTLR
pub type GicdCtlrLocal = LocalRegisterCopy<u32, GICD_CTLR::Register>;

/// GICD_TYPER register using MMIO
/// - Offset: 0x004
pub type GicdTyperRegister = ReadOnly<u32, GICD_TYPER::Register>;

/// Local copy of GICD_TYPER
pub type GicdTyperLocal = LocalRegisterCopy<u32, GICD_TYPER::Register>;

/// GICD_IIDR register using MMIO
/// - Offset: 0x008
pub type GicdIidrRegister = ReadOnly<u32, GICD_IIDR::Register>;

/// Local copy of GICD_IIDR
pub type GicdIidrLocal = LocalRegisterCopy<u32, GICD_IIDR::Register>;

/// GICD_IGROUPR register using MMIO
pub type GicdIgroupRegister = ReadWrite<u32, GICD_IGROUPR::Register>;

/// Local copy of GICD_IGROUPR
pub type GicdIgroupLocal = LocalRegisterCopy<u32, GICD_IGROUPR::Register>;

/// GICD_IGRPMODR register using MMIO
pub type GicdIgrpmodRegister = ReadWrite<u32, GICD_IGRPMODR::Register>;

/// Local copy of GICD_IGRPMODR
pub type GicdIgrpmodLocal = LocalRegisterCopy<u32, GICD_IGRPMODR::Register>;

/// GICD_ISENABLER register using MMIO
pub type GicdIsenablerRegister = WriteOnly<u32, GICD_ISENABLER::Register>;

/// GICD_ICENABLER register using MMIO
pub type GicdIcenablerRegister = WriteOnly<u32, GICD_ICENABLER::Register>;

/// GICD_ISPENDR register using MMIO
pub type GicdIspendrRegister = WriteOnly<u32, GICD_ISPENDR::Register>;

/// GICD_ICPENDR register using MMIO
pub type GicdIcpendrRegister = WriteOnly<u32, GICD_ICPENDR::Register>;

/// GICD_ISACTIVER register using MMIO
pub type GicdIsactiverRegister = WriteOnly<u32, GICD_ISACTIVER::Register>;

/// GICD_ICACTIVER register using MMIO
pub type GicdIcactiverRegister = WriteOnly<u32, GICD_ICACTIVER::Register>;

/// GICD_IPRIORITYR register using MMIO
pub type GicdIpriorityRegister = ReadWrite<u32, GICD_IPRIORITYR::Register>;

/// Local copy of GICD_IPRIORITYR
pub type GicdIpriorityLocal = LocalRegisterCopy<u32, GICD_IPRIORITYR::Register>;

/// GICD_ICFGR register using MMIO
pub type GicdIcfgRegister = ReadWrite<u32, GICD_ICFGR::Register>;

/// GICD_IROUTER register using MMIO (64-bit)
pub type GicdIrouterRegister = ReadWrite<u64, GICD_IROUTER::Register>;

/// GICD_SGIR register using MMIO
pub type GicdSgirRegister = WriteOnly<u32, GICD_SGIR::Register>;

/// GICD_NSACR register using MMIO
pub type GicdNsacrRegister = ReadWrite<u32, GICD_NSACR::Register>;

/// GICD_PIDR4 register using MMIO
/// 虚拟 GIC 使用 ReadWrite 以便设置初始值
pub type GicdPidr4Register = ReadWrite<u32, GICD_PIDR4::Register>;

/// GICD_PIDR register using MMIO
/// 虚拟 GIC 使用 ReadWrite 以便设置初始值
pub type GicdPidrRegister = ReadWrite<u32, GICD_PIDR::Register>;

/// GICD_CIDR register using MMIO
/// 虚拟 GIC 使用 ReadWrite 以便设置初始值
pub type GicdCidrRegister = ReadWrite<u32, GICD_CIDR::Register>;

// ============================================================================
// Main Register Block Structure
// ============================================================================
//
// GICv3 Distributor 寄存器布局（64KB MMIO 区域）:
// - 页面 0 (0x0000-0x0FFF): 基础控制寄存器 + 中断 0-31 的配置
// - 页面 1-5 (0x1000-0x5FFF): Extended SPI 寄存器 (IMPLEMENTATION DEFINED)
// - 页面 6-7 (0x6000-0x7FFF): IROUTER 寄存器 (SPI 32-1019)
// - 页面 8-9 (0x8000-0x9FFF): Extended SPI IROUTER (IMPLEMENTATION DEFINED)
// - 页面 10-14 (0xA000-0xEFFF): Reserved / IMPLEMENTATION DEFINED
// - 页面 15 (0xF000-0xFFFF): 保留 + ID 寄存器
//
// 注意：每个 4KB 页面的末尾（偏移 0xFD0-0xFFC）都包含 ID 寄存器
//       这是 GICv3 规范的要求，用于支持 64KB 寻址空间

register_structs! {
    #[allow(non_snake_case)]
    pub GicdBlock {
        // ====================================================================
        // 页面 0: 基础控制寄存器 (0x0000 - 0x0FFF)
        // ====================================================================

        // === 基础控制寄存器 (0x000 - 0x00C) ===
        (0x0000 => pub CTLR: GicdCtlrRegister),
        (0x0004 => pub TYPER: GicdTyperRegister),
        (0x0008 => pub IIDR: GicdIidrRegister),
        (0x000C => _reserved_0_000c),

        // === 中断组寄存器 (0x080 - 0x0FF) ===
        (0x0080 => pub IGROUPR: [GicdIgroupRegister; 32]),

        // === 使能控制寄存器 (0x100 - 0x1FF) ===
        (0x0100 => pub ISENABLER: [GicdIsenablerRegister; 32]),
        (0x0180 => pub ICENABLER: [GicdIcenablerRegister; 32]),

        // === 挂起状态寄存器 (0x200 - 0x2FF) ===
        (0x0200 => pub ISPENDR: [GicdIspendrRegister; 32]),
        (0x0280 => pub ICPENDR: [GicdIcpendrRegister; 32]),

        // === 活跃状态寄存器 (0x300 - 0x3FF) ===
        (0x0300 => pub ISACTIVER: [GicdIsactiverRegister; 32]),
        (0x0380 => pub ICACTIVER: [GicdIcactiverRegister; 32]),

        // === 优先级寄存器 (0x400 - 0x7FF) ===
        (0x0400 => pub IPRIORITYR: [GicdIpriorityRegister; 255]),
        (0x07FC => _reserved_0_07fc),

        // === 中断配置寄存器 (0xC00 - 0xCFF) ===
        // ICFGR0: SGI (0-15) 只读，必须为边沿触发
        // ICFGR1: PPI (16-31) 可配置
        (0x0C00 => pub ICFGR: [GicdIcfgRegister; 64]),

        // === 中断组修饰符寄存器 (0xD00 - 0xD7F) ===
        (0x0D00 => pub IGRPMODR: [GicdIgrpmodRegister; 32]),
        (0x0D80 => _reserved_0_0d80),

        // === 非安全访问控制寄存器 (0xE00 - 0xE7F) ===
        (0x0E00 => pub NSACR: [GicdNsacrRegister; 32]),
        (0x0E80 => _reserved_0_0e80),

        // === 软件生成中断寄存器 (0xF00 - 0xF03) ===
        (0x0F00 => pub SGIR: GicdSgirRegister),
        (0x0F04 => _reserved_0_0f04),

        // === 页面 0 ID 寄存器 (0xFD0 - 0xFFF) ===
        (0x0FD0 => pub PIDR4_P0: GicdPidr4Register),
        (0x0FD4 => _reserved_0_0fd4),
        (0x0FE0 => pub PIDR0_P0: GicdPidrRegister),
        (0x0FE4 => pub PIDR1_P0: GicdPidrRegister),
        (0x0FE8 => pub PIDR2_P0: GicdPidrRegister),  // 关键：客户机读 0xFFE8
        (0x0FEC => pub PIDR3_P0: GicdPidrRegister),
        (0x0FF0 => pub CIDR0_P0: GicdCidrRegister),
        (0x0FF4 => pub CIDR1_P0: GicdCidrRegister),
        (0x0FF8 => pub CIDR2_P0: GicdCidrRegister),
        (0x0FFC => pub CIDR3_P0: GicdCidrRegister),

        // ====================================================================
        // 页面 1-5: Extended SPI 寄存器 (0x1000 - 0x5FFF)
        // 注意：此区域的某些寄存器为 IMPLEMENTATION DEFINED
        // ====================================================================

        // 页面 1 (0x1000-0x1FFF)
        (0x1000 => _reserved_page1_start),
        (0x1FD0 => pub PIDR4_P1: GicdPidr4Register),
        (0x1FD4 => _reserved_1_0fd4),
        (0x1FE0 => pub PIDR0_P1: GicdPidrRegister),
        (0x1FE4 => pub PIDR1_P1: GicdPidrRegister),
        (0x1FE8 => pub PIDR2_P1: GicdPidrRegister),
        (0x1FEC => pub PIDR3_P1: GicdPidrRegister),
        (0x1FF0 => pub CIDR0_P1: GicdCidrRegister),
        (0x1FF4 => pub CIDR1_P1: GicdCidrRegister),
        (0x1FF8 => pub CIDR2_P1: GicdCidrRegister),
        (0x1FFC => pub CIDR3_P1: GicdCidrRegister),

        // 页面 2 (0x2000-0x2FFF)
        (0x2000 => _reserved_page2_start),
        (0x2FD0 => pub PIDR4_P2: GicdPidr4Register),
        (0x2FD4 => _reserved_2_0fd4),
        (0x2FE0 => pub PIDR0_P2: GicdPidrRegister),
        (0x2FE4 => pub PIDR1_P2: GicdPidrRegister),
        (0x2FE8 => pub PIDR2_P2: GicdPidrRegister),
        (0x2FEC => pub PIDR3_P2: GicdPidrRegister),
        (0x2FF0 => pub CIDR0_P2: GicdCidrRegister),
        (0x2FF4 => pub CIDR1_P2: GicdCidrRegister),
        (0x2FF8 => pub CIDR2_P2: GicdCidrRegister),
        (0x2FFC => pub CIDR3_P2: GicdCidrRegister),

        // 页面 3 (0x3000-0x3FFF)
        (0x3000 => _reserved_page3_start),
        (0x3FD0 => pub PIDR4_P3: GicdPidr4Register),
        (0x3FD4 => _reserved_3_0fd4),
        (0x3FE0 => pub PIDR0_P3: GicdPidrRegister),
        (0x3FE4 => pub PIDR1_P3: GicdPidrRegister),
        (0x3FE8 => pub PIDR2_P3: GicdPidrRegister),
        (0x3FEC => pub PIDR3_P3: GicdPidrRegister),
        (0x3FF0 => pub CIDR0_P3: GicdCidrRegister),
        (0x3FF4 => pub CIDR1_P3: GicdCidrRegister),
        (0x3FF8 => pub CIDR2_P3: GicdCidrRegister),
        (0x3FFC => pub CIDR3_P3: GicdCidrRegister),

        // 页面 4 (0x4000-0x4FFF)
        (0x4000 => _reserved_page4_start),
        (0x4FD0 => pub PIDR4_P4: GicdPidr4Register),
        (0x4FD4 => _reserved_4_0fd4),
        (0x4FE0 => pub PIDR0_P4: GicdPidrRegister),
        (0x4FE4 => pub PIDR1_P4: GicdPidrRegister),
        (0x4FE8 => pub PIDR2_P4: GicdPidrRegister),
        (0x4FEC => pub PIDR3_P4: GicdPidrRegister),
        (0x4FF0 => pub CIDR0_P4: GicdCidrRegister),
        (0x4FF4 => pub CIDR1_P4: GicdCidrRegister),
        (0x4FF8 => pub CIDR2_P4: GicdCidrRegister),
        (0x4FFC => pub CIDR3_P4: GicdCidrRegister),

        // 页面 5 (0x5000-0x5FFF)
        (0x5000 => _reserved_page5_start),
        (0x5FD0 => pub PIDR4_P5: GicdPidr4Register),
        (0x5FD4 => _reserved_5_0fd4),
        (0x5FE0 => pub PIDR0_P5: GicdPidrRegister),
        (0x5FE4 => pub PIDR1_P5: GicdPidrRegister),
        (0x5FE8 => pub PIDR2_P5: GicdPidrRegister),
        (0x5FEC => pub PIDR3_P5: GicdPidrRegister),
        (0x5FF0 => pub CIDR0_P5: GicdCidrRegister),
        (0x5FF4 => pub CIDR1_P5: GicdCidrRegister),
        (0x5FF8 => pub CIDR2_P5: GicdCidrRegister),
        (0x5FFC => pub CIDR3_P5: GicdCidrRegister),

        // ====================================================================
        // 页面 6-7: IROUTER 寄存器 (0x6000 - 0x7FFF)
        // GICD_IROUTER<n> 用于 SPI 中断 32-1019 的亲和路由
        // 每个 IROUTER 寄存器 64 位 (8 字节)
        // ====================================================================

        // 页面 6 (0x6000-0x6FFF): IROUTER 寄存器
        // 可用空间: 0x6FD0 - 0x6000 = 0xFD0 = 4048 字节
        // 每个 IROUTER 8 字节，前面预留 8 字节对齐
        (0x6000 => _reserved_6_6000),
        (0x6008 => pub IROUTER_P6: [GicdIrouterRegister; 503]),
        (0x6FC0 => _reserved_6_6fc0),
        (0x6FD0 => pub PIDR4_P6: GicdPidr4Register),
        (0x6FD4 => _reserved_6_0fd4),
        (0x6FE0 => pub PIDR0_P6: GicdPidrRegister),
        (0x6FE4 => pub PIDR1_P6: GicdPidrRegister),
        (0x6FE8 => pub PIDR2_P6: GicdPidrRegister),
        (0x6FEC => pub PIDR3_P6: GicdPidrRegister),
        (0x6FF0 => pub CIDR0_P6: GicdCidrRegister),
        (0x6FF4 => pub CIDR1_P6: GicdCidrRegister),
        (0x6FF8 => pub CIDR2_P6: GicdCidrRegister),
        (0x6FFC => pub CIDR3_P6: GicdCidrRegister),

        // 页面 7 (0x7000-0x7FFF): IROUTER 寄存器
        // 可用空间: 0x7FD0 - 0x7000 = 0xFD0 = 4048 字节
        // 每个 IROUTER 8 字节，前面预留 8 字节对齐
        (0x7000 => _reserved_7_7000),
        (0x7008 => pub IROUTER_P7: [GicdIrouterRegister; 503]),
        (0x7FC0 => _reserved_7_7fc0),
        (0x7FD0 => pub PIDR4_P7: GicdPidr4Register),
        (0x7FD4 => _reserved_7_0fd4),
        (0x7FE0 => pub PIDR0_P7: GicdPidrRegister),
        (0x7FE4 => pub PIDR1_P7: GicdPidrRegister),
        (0x7FE8 => pub PIDR2_P7: GicdPidrRegister),
        (0x7FEC => pub PIDR3_P7: GicdPidrRegister),
        (0x7FF0 => pub CIDR0_P7: GicdCidrRegister),
        (0x7FF4 => pub CIDR1_P7: GicdCidrRegister),
        (0x7FF8 => pub CIDR2_P7: GicdCidrRegister),
        (0x7FFC => pub CIDR3_P7: GicdCidrRegister),

        // ====================================================================
        // 页面 8-9: Extended SPI IROUTER (0x8000 - 0x9FFF)
        // IMPLEMENTATION DEFINED - 用于扩展 SPI 中断
        // ====================================================================

        // 页面 8 (0x8000-0x8FFF)
        (0x8000 => _reserved_page8_start),
        (0x8FD0 => pub PIDR4_P8: GicdPidr4Register),
        (0x8FD4 => _reserved_8_0fd4),
        (0x8FE0 => pub PIDR0_P8: GicdPidrRegister),
        (0x8FE4 => pub PIDR1_P8: GicdPidrRegister),
        (0x8FE8 => pub PIDR2_P8: GicdPidrRegister),
        (0x8FEC => pub PIDR3_P8: GicdPidrRegister),
        (0x8FF0 => pub CIDR0_P8: GicdCidrRegister),
        (0x8FF4 => pub CIDR1_P8: GicdCidrRegister),
        (0x8FF8 => pub CIDR2_P8: GicdCidrRegister),
        (0x8FFC => pub CIDR3_P8: GicdCidrRegister),

        // 页面 9 (0x9000-0x9FFF)
        (0x9000 => _reserved_page9_start),
        (0x9FD0 => pub PIDR4_P9: GicdPidr4Register),
        (0x9FD4 => _reserved_9_0fd4),
        (0x9FE0 => pub PIDR0_P9: GicdPidrRegister),
        (0x9FE4 => pub PIDR1_P9: GicdPidrRegister),
        (0x9FE8 => pub PIDR2_P9: GicdPidrRegister),
        (0x9FEC => pub PIDR3_P9: GicdPidrRegister),
        (0x9FF0 => pub CIDR0_P9: GicdCidrRegister),
        (0x9FF4 => pub CIDR1_P9: GicdCidrRegister),
        (0x9FF8 => pub CIDR2_P9: GicdCidrRegister),
        (0x9FFC => pub CIDR3_P9: GicdCidrRegister),

        // ====================================================================
        // 页面 10-14: Reserved / IMPLEMENTATION DEFINED (0xA000 - 0xEFFF)
        // ====================================================================

        // 页面 10 (0xA000-0xAFFF)
        (0xA000 => _reserved_page10_start),
        (0xAFD0 => pub PIDR4_P10: GicdPidr4Register),
        (0xAFD4 => _reserved_10_0fd4),
        (0xAFE0 => pub PIDR0_P10: GicdPidrRegister),
        (0xAFE4 => pub PIDR1_P10: GicdPidrRegister),
        (0xAFE8 => pub PIDR2_P10: GicdPidrRegister),
        (0xAFEC => pub PIDR3_P10: GicdPidrRegister),
        (0xAFF0 => pub CIDR0_P10: GicdCidrRegister),
        (0xAFF4 => pub CIDR1_P10: GicdCidrRegister),
        (0xAFF8 => pub CIDR2_P10: GicdCidrRegister),
        (0xAFFC => pub CIDR3_P10: GicdCidrRegister),

        // 页面 11 (0xB000-0xBFFF)
        (0xB000 => _reserved_page11_start),
        (0xBFD0 => pub PIDR4_P11: GicdPidr4Register),
        (0xBFD4 => _reserved_11_0fd4),
        (0xBFE0 => pub PIDR0_P11: GicdPidrRegister),
        (0xBFE4 => pub PIDR1_P11: GicdPidrRegister),
        (0xBFE8 => pub PIDR2_P11: GicdPidrRegister),
        (0xBFEC => pub PIDR3_P11: GicdPidrRegister),
        (0xBFF0 => pub CIDR0_P11: GicdCidrRegister),
        (0xBFF4 => pub CIDR1_P11: GicdCidrRegister),
        (0xBFF8 => pub CIDR2_P11: GicdCidrRegister),
        (0xBFFC => pub CIDR3_P11: GicdCidrRegister),

        // 页面 12 (0xC000-0xCFFF)
        (0xC000 => _reserved_page12_start),
        (0xCFD0 => pub PIDR4_P12: GicdPidr4Register),
        (0xCFD4 => _reserved_12_0fd4),
        (0xCFE0 => pub PIDR0_P12: GicdPidrRegister),
        (0xCFE4 => pub PIDR1_P12: GicdPidrRegister),
        (0xCFE8 => pub PIDR2_P12: GicdPidrRegister),
        (0xCFEC => pub PIDR3_P12: GicdPidrRegister),
        (0xCFF0 => pub CIDR0_P12: GicdCidrRegister),
        (0xCFF4 => pub CIDR1_P12: GicdCidrRegister),
        (0xCFF8 => pub CIDR2_P12: GicdCidrRegister),
        (0xCFFC => pub CIDR3_P12: GicdCidrRegister),

        // 页面 13 (0xD000-0xDFFF)
        (0xD000 => _reserved_page13_start),
        (0xDFD0 => pub PIDR4_P13: GicdPidr4Register),
        (0xDFD4 => _reserved_13_0fd4),
        (0xDFE0 => pub PIDR0_P13: GicdPidrRegister),
        (0xDFE4 => pub PIDR1_P13: GicdPidrRegister),
        (0xDFE8 => pub PIDR2_P13: GicdPidrRegister),
        (0xDFEC => pub PIDR3_P13: GicdPidrRegister),
        (0xDFF0 => pub CIDR0_P13: GicdCidrRegister),
        (0xDFF4 => pub CIDR1_P13: GicdCidrRegister),
        (0xDFF8 => pub CIDR2_P13: GicdCidrRegister),
        (0xDFFC => pub CIDR3_P13: GicdCidrRegister),

        // 页面 14 (0xE000-0xEFFF)
        (0xE000 => _reserved_page14_start),
        (0xEFD0 => pub PIDR4_P14: GicdPidr4Register),
        (0xEFD4 => _reserved_14_0fd4),
        (0xEFE0 => pub PIDR0_P14: GicdPidrRegister),
        (0xEFE4 => pub PIDR1_P14: GicdPidrRegister),
        (0xEFE8 => pub PIDR2_P14: GicdPidrRegister),
        (0xEFEC => pub PIDR3_P14: GicdPidrRegister),
        (0xEFF0 => pub CIDR0_P14: GicdCidrRegister),
        (0xEFF4 => pub CIDR1_P14: GicdCidrRegister),
        (0xEFF8 => pub CIDR2_P14: GicdCidrRegister),
        (0xEFFC => pub CIDR3_P14: GicdCidrRegister),

        // ====================================================================
        // 页面 15: 保留区域 (0xF000 - 0xFFFF)
        // ====================================================================

        (0xF000 => _reserved_page15_start),
        (0xFFD0 => pub PIDR4_P15: GicdPidr4Register),
        (0xFFD4 => _reserved_15_0fd4),
        (0xFFE0 => pub PIDR0_P15: GicdPidrRegister),
        (0xFFE4 => pub PIDR1_P15: GicdPidrRegister),
        (0xFFE8 => pub PIDR2_P15: GicdPidrRegister),  // 0xFFE8: 客户机访问的地址
        (0xFFEC => pub PIDR3_P15: GicdPidrRegister),
        (0xFFF0 => pub CIDR0_P15: GicdCidrRegister),
        (0xFFF4 => pub CIDR1_P15: GicdCidrRegister),
        (0xFFF8 => pub CIDR2_P15: GicdCidrRegister),
        (0xFFFC => pub CIDR3_P15: GicdCidrRegister),

        (0x10000 => @END),
    }
}

// === 路由寄存器块 (GICv3 特有) ===
// IROUTER 用于 SPI 中断（32+），每个 8 字节
// 注意：这些寄存器位于独立的 MMIO 区域（偏移 0x6000+）
// 使用时需要在基地址上加上 0x6000
register_structs! {
    #[allow(non_snake_case)]
    pub GicdRouterBlock {
        (0x0000 => pub IROUTER: [GicdIrouterRegister; 988]),
        (0x1EE0 => @END),
    }
}
