#![cfg_attr(not(test), no_std)]
// 增加递归限制以支持大型寄存器结构体的宏展开
#![recursion_limit = "512"]

pub use vdev_if::*;

pub mod v3;
