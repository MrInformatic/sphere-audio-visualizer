//! Contains the algorithim used for audio analysis

pub use self::{filter::*, spectrum::*};

mod filter;
mod spectrum;
pub mod utils;

/// Stores multible samples together with the coresponding sample rate
#[derive(Clone)]
pub struct Samples<'a> {
    /// The sample rate
    pub sample_rate: f64,
    /// The samples
    pub samples: &'a [f32],
}
