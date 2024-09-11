pub(crate) const SGI_ID_MAX: usize = 16;
pub(crate) const PPI_ID_MAX: usize = 32; /* 16...31 */
pub(crate) const SPI_ID_MAX: usize = 512;
pub(crate) const GICD_LR_NUM: usize = 4;

/* ============ handler use offset ============= */
pub(crate) const VGICD_CTLR: usize = 0x0;
pub(crate) const VGICD_ISENABLER: usize = 0x2;
pub(crate) const VGICD_ICENABLER: usize = 0x3;
pub(crate) const VGICD_ISPENDR: usize = 0x4;
pub(crate) const VGICD_ICPENDR: usize = 0x5;
pub(crate) const VGICD_ISACTIVER: usize = 0x6;
pub(crate) const VGICD_ICACTIVER: usize = 0x7;
pub(crate) const VGICD_ICFGR: usize = 0x18;
pub(crate) const VGICD_SGIR: usize = 0x1e;
