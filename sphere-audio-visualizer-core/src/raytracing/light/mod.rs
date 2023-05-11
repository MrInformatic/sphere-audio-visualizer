//!Contains implementations of of the supported lights.

use glam::{vec3a, Vec3A};

pub use self::point::*;

use super::{Ray, SurfaceProperties};

mod point;

/// A light is used to light diffuse surfaces
pub trait Light: Send + Sync {
    /// Retuns the light intesity on the given point `surface`. `intersect`
    /// is used for shadow calculations.
    fn intensity(
        &self,
        surface: &SurfaceProperties,
        intersect: impl Fn(&Ray) -> bool + Copy,
    ) -> Vec3A;
}

/// A wrapper for a collection of multiple lights that implements the [`Light`]
/// trait.
pub struct LightGroup<'a, L: Light>(pub &'a [L]);

impl<'a, L: Light> Light for LightGroup<'a, L> {
    fn intensity(
        &self,
        surface: &SurfaceProperties,
        intersect: impl Fn(&Ray) -> bool + Copy,
    ) -> Vec3A {
        let mut intensity = vec3a(0.0, 0.0, 0.0);

        for id in 0..self.0.len() {
            intensity += self.0[id].intensity(surface, intersect);
        }

        intensity
    }
}

/// Stores the light setup of a scene. Every supported light type should be
/// represented in this struct with a [`LightGroup`] field.
pub struct LightScene<'a> {
    /// The [`LightGroup`] for [`PointLight`]
    pub point_lights: LightGroup<'a, PointLight>,
}

impl<'a> Light for LightScene<'a> {
    fn intensity(
        &self,
        surface: &SurfaceProperties,
        intersect: impl Fn(&Ray) -> bool + Copy,
    ) -> Vec3A {
        self.point_lights.intensity(surface, intersect)
    }
}
