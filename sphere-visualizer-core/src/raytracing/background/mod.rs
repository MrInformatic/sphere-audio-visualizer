//! Contains implementations of of the supported backgrounds.

use glam::Vec3A;

pub use self::constant::*;

mod constant;

/// The Background defines the radiance returned by the radiance algorithm if
/// nothing was hit.
pub trait Background {
    /// Returns the radiance if nothing was hit.
    fn radiance(&self, direction: &Vec3A) -> Vec3A;

    /// Returns the emitted light intensity of the background in the specified
    /// direction.
    fn intensity(&self, normal: &Vec3A) -> Vec3A;
}
