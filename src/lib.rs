#![no_std]
#![feature(concat_idents)]

mod devops_impl;

pub mod vgic;
pub use vgic::Vgic;

mod consts;
// mod vgicc;
#[cfg(feature = "vgicv3")]
mod gits;
mod interrupt;
mod list_register;
mod registers;
#[cfg(feature = "vgicv3")]
mod registers_v3;
#[cfg(feature = "vgicv3")]
mod utils_v3;
mod vgicd;
#[cfg(feature = "vgicv3")]
mod vgicdv3;
#[cfg(feature = "vgicv3")]
mod vgicrv3;
