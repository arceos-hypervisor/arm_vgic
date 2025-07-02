pub mod gits;
mod registers;
mod utils;
#[cfg(target_arch = "aarch64")]
pub mod vgicd;
#[cfg(target_arch = "aarch64")]
pub mod vgicr;
