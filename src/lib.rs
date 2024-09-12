#![no_std]

mod devops_impl;


pub mod vgic;
pub use vgic::Vgic;

mod consts;
mod vgicc;