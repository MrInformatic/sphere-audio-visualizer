//! This crate implements the platform independent functionality of the sphere audio visualizer.
//! To start look at the [`crate::frontend::Application`] struct

#![feature(
    ptr_metadata,
    layout_for_ptr,
    unsize,
    int_roundings,
    box_into_inner,
    downcast_unchecked,
    type_alias_impl_trait,
    div_duration,
    drain_filter
)]
#![warn(missing_docs)]

pub use self::{frontend::*, module::*, visualizer::*};

pub mod audio_analysis;
mod frontend;
mod module;
pub mod rendering;
pub mod simulation;
pub mod utils;
mod visualizer;
