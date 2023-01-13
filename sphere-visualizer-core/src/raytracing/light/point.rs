use glam::{vec3a, Vec3A};

#[cfg(target_arch = "spirv")]
use num_traits::Float;

use crate::{
    raytracing::{Ray, SurfaceProperties},
    utils::math::{dot, inverse_sqrt},
};

use super::Light;

/// Implements a point light
#[repr(C, align(16))]
pub struct PointLight {
    position: Vec3A,
    intensity: Vec3A,
}

impl PointLight {
    /// Creates a new instance
    /// - `position` Represents the position of the point light
    /// - `intensity` Represents the intensity and color of the point light
    pub fn new(position: Vec3A, intensity: Vec3A) -> Self {
        Self {
            position,
            intensity,
        }
    }
}

impl Light for PointLight {
    fn intensity(&self, surface: &SurfaceProperties, intersect: impl Fn(&Ray) -> bool) -> Vec3A {
        let dir = self.position - surface.position;

        let shadow_ray = Ray::new(surface.position, dir, 0.0001, 0.9999);

        if (intersect)(&shadow_ray) {
            vec3a(0.0, 0.0, 0.0)
        } else {
            let mag2 = dot(&dir, &dir);
            let dir_normalized = dir * inverse_sqrt(mag2);
            (self.intensity / mag2) * dot(&surface.normal, &dir_normalized).max(0.0)
        }
    }
}
