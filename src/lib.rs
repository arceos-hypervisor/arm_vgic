#![no_std]
#![feature(concat_idents)]

mod devops_impl;

pub mod vgic;
pub use vgic::Vgic;

mod consts;
// mod vgicc;
mod gits;
mod interrupt;
mod list_register;
mod registers;
mod registers_v3;
mod utils;
mod vgicd;
mod vgicdv3;
mod vgicrv3;
