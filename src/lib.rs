#![no_std]
#![feature(concat_idents)]

mod devops_impl;

pub mod vgic;
pub use vgic::Vgic;

mod consts;
// mod vgicc;
mod interrupt;
mod list_register;
mod registers;
mod vgicd;
mod vgicdv3;
mod registers_v3;