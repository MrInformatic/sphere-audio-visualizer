#![feature(maybe_uninit_uninit_array)]
#![cfg_attr(target_arch = "spirv", feature(asm_experimental_arch))]
#![no_std]
#![warn(missing_docs)]

//! This crate contains all the base mathematical algorithms used. This incudes
//! a implemntation of the raytracing algorithm.

pub use glam;

pub mod metaballs;
pub mod raytracing;
pub mod utils;
