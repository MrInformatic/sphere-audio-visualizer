use glam::{vec2, vec3a, Mat4, Vec2, Vec3A};

#[cfg(target_arch = "spirv")]
use num_traits::Float;

use crate::{raytracing::Ray, utils::math::normalize};

use super::Camera;

/// Implements a Perspective Camera
#[repr(C, align(16))]
#[derive(Clone)]
pub struct PerspectiveCamera {
    transform: Mat4,
    screen_size: Vec2,
    tan_fov: f32,
    t_min: f32,
    t_max: f32,
}

impl PerspectiveCamera {
    /// Creates a new instance
    /// - `transform` represents the transform of the camera in world space
    /// - `screen_size` represents the screen size in pixels
    /// - `fov` represents the field of view in radians of the camera
    /// - `t_min` represents the near plane of the camera.
    /// - `t_max` represents the far plane of the camera.
    pub fn new(transform: Mat4, screen_size: Vec2, fov: f32, t_min: f32, t_max: f32) -> Self {
        Self {
            transform,
            screen_size,
            tan_fov: fov.tan(),
            t_min,
            t_max,
        }
    }
}

impl Camera for PerspectiveCamera {
    fn prime_ray(&self, sample: &Vec2) -> Ray {
        let sensor = (*sample / self.screen_size * 2.0 - Vec2::splat(1.0))
            * self.tan_fov
            * vec2(1.0, -(self.screen_size.y / self.screen_size.x));

        let ray = Ray::new(
            vec3a(0.0, 0.0, 0.0),
            normalize(&Vec3A::from(sensor.extend(1.0))),
            self.t_min,
            self.t_max,
        );

        ray.transform(&self.transform)
    }
}
