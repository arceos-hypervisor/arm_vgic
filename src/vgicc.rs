
use crate::consts::*;
pub(crate) struct Vgicc {
    id: u32,
    pending_lr: [u32; SPI_ID_MAX],
    saved_lr: [u32; GICD_LR_NUM],

    saved_elsr0: u32,
    saved_apr: u32,
    saved_hcr: u32,

    isenabler: u32, // 0..31
    priorityr: [u8; PPI_ID_MAX],
}
