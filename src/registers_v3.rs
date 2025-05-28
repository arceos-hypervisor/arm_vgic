//! TODO: merge this with `src/registers.rs`

/// Maximum number of IRQs supported by GICv3. We count special interrupt numbers from 1020 to 1023,
/// to make the number more intuitive and easier to work with.
pub const MAX_IRQ_V3: usize = 1024;

macro_rules! register_range {
    ($reg:ident, $range:ident, $number:literal, $size:literal) => {
        pub const $range: core::ops::Range<usize> = ($reg)..($reg + $number * $size);
    };
}

pub const GICD_CTLR: usize = 0x0000;
pub const GICD_CTLR_ARE_NS: usize = 1 << 5;
pub const GICD_CTLR_GRP1NS_ENA: usize = 1 << 1;

pub const GICD_TYPER: usize = 0x0004;
pub const GICD_IIDR: usize = 0x0008;
pub const GICD_TYPER2: usize = 0x000c;

pub const GICD_IGROUPR: usize = 0x0080;
pub const GICD_ISENABLER: usize = 0x0100;
pub const GICD_ICENABLER: usize = 0x0180;
pub const GICD_ISPENDR: usize = 0x0200;
pub const GICD_ICPENDR: usize = 0x0280;
pub const GICD_ISACTIVER: usize = 0x0300;
pub const GICD_ICACTIVER: usize = 0x0380;
pub const GICD_IPRIORITYR: usize = 0x0400;
pub const GICD_ITARGETSR: usize = 0x0800;
pub const GICD_ICFGR: usize = 0x0c00;
pub const GICD_NSACR: usize = 0x0e00;
pub const GICD_SGIR: usize = 0x0f00;
pub const GICD_CPENDSGIR: usize = 0x0f10;
pub const GICD_SPENDSGIR: usize = 0x0f20;
pub const GICD_IROUTER: usize = 0x6000;

pub const GICDV3_CIDR0: usize = 0xfff0;
pub const GICDV3_PIDR0: usize = 0xffe0;
pub const GICDV3_PIDR2: usize = 0xffe8;
pub const GICDV3_PIDR4: usize = 0xffd0;

register_range!(GICD_IROUTER, GICD_IROUTER_RANGE, 1024, 8);
register_range!(GICD_ITARGETSR, GICD_ITARGETSR_RANGE, 1024, 1);
register_range!(GICD_ICENABLER, GICD_ICENABLER_RANGE, 32, 4);
register_range!(GICD_ISENABLER, GICD_ISENABLER_RANGE, 32, 4);
register_range!(GICD_ICPENDR, GICD_ICPENDR_RANGE, 32, 4);
register_range!(GICD_ISPENDR, GICD_ISPENDR_RANGE, 32, 4);
register_range!(GICD_ICACTIVER, GICD_ICACTIVER_RANGE, 32, 4);
register_range!(GICD_ISACTIVER, GICD_ISACTIVER_RANGE, 32, 4);
register_range!(GICD_IGROUPR, GICD_IGROUPR_RANGE, 32, 4);
register_range!(GICD_ICFGR, GICD_ICFGR_RANGE, 64, 4);
register_range!(GICD_IPRIORITYR, GICD_IPRIORITYR_RANGE, 255, 4);
register_range!(GICDV3_CIDR0, GICDV3_CIDR0_RANGE, 4, 4);
register_range!(GICDV3_PIDR0, GICDV3_PIDR0_RANGE, 4, 4);
register_range!(GICDV3_PIDR4, GICDV3_PIDR4_RANGE, 4, 4);

pub const GITS_CTRL: usize = 0x0000; // enable / disable
pub const GITS_IIDR: usize = 0x0004; // read-only
pub const GITS_TYPER: usize = 0x0008; // read-only
pub const GITS_MPAMIDR: usize = 0x0010; // read-only
pub const GITS_PARTIDR: usize = 0x0014; // supported MPAM sizes
pub const GITS_MPIDR: usize = 0x0018; // read-only, its affinity
pub const GITS_STATUSR: usize = 0x0040; // error reporting
pub const GITS_UMSIR: usize = 0x0048; // unmapped msi
pub const GITS_CBASER: usize = 0x0080; // the addr of command queue
pub const GITS_CWRITER: usize = 0x0088; // rw, write an command to the cmdq, write this reg to tell hw
pub const GITS_CREADR: usize = 0x0090; // read-only, hardware changes it
pub const GITS_BASER: usize = 0x0100; // itt, desc
pub const GITS_DT_BASER: usize = GITS_BASER; // its table 0 used as device table
pub const GITS_CT_BASER: usize = GITS_BASER + 0x8; // its table 1 used as command table
pub const GITS_COLLECTION_BASER: usize = GITS_BASER + 0x8;
pub const GITS_TRANSLATER: usize = 0x10000 + 0x0040; // to signal an interrupt, written by devices
