#![no_std]
#![feature(unbounded_shifts)]

#[cfg(target_arch = "aarch64")]
mod devops_impl;

#[cfg(target_arch = "aarch64")]
pub mod vgic;
#[cfg(target_arch = "aarch64")]
pub use vgic::Vgic;

mod consts;
// mod vgicc;
mod interrupt;
mod list_register;
mod registers;
#[cfg(target_arch = "aarch64")]
mod vgicd;

#[cfg(feature = "vgicv3")]
pub mod v3;
