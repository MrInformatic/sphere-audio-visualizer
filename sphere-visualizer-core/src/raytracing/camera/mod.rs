//! Contains implementations of of the supported raytracing cameras.

use glam::Vec2;

pub use self::perspective::*;
use super::Ray;

mod perspective;

/// A Camera is used to generate prime rays for raytracing
pub trait Camera {
    /// Generates a prime ray for a screen position
    fn prime_ray(&self, sample: &Vec2) -> Ray;
}
