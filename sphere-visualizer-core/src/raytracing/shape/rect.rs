use glam::{vec3a, Mat4, Vec3A};

use crate::{
    raytracing::{Ray, SurfaceProperties},
    utils::{math::transform_point3a, OptionPolyfill},
};

use super::{Shading, Shape, AABB};

/// Implements a rectangle shape with a normal pointing into positive y-axis
/// direction and a side length of 1.0 and emissive material
#[repr(C, align(16))]
pub struct Rect {
    transform: Mat4,
    color: Vec3A,
}

impl Rect {
    /// Creates a new instance:
    /// - `transform` Represents the transform of the rectangle in world space
    /// - `color` Represents the color of the rectangle
    pub fn new(transform: Mat4, color: Vec3A) -> Self {
        Self { transform, color }
    }
}

impl Shape for Rect {
    fn intersect(&self, ray: &Ray) -> OptionPolyfill<f32> {
        let ray = ray.transform(&self.transform);

        let dot = ray.direction.y;

        let t = (-ray.origin.y) / dot;
        let position = ray.point_at(t);

        if ray.valid_t(t)
            && position.x < 0.5
            && position.x > -0.5
            && position.z < 0.5
            && position.z > -0.5
        {
            return OptionPolyfill::some(t);
        }

        OptionPolyfill::none()
    }

    fn distance(&self, _point: &Vec3A) -> f32 {
        f32::INFINITY
    }

    fn shade(
        &self,
        _ray: &Ray,
        _t: f32,
        _intensity: impl Fn(&SurfaceProperties) -> Vec3A,
    ) -> Shading {
        Shading {
            emission: self.color,
            reflection: OptionPolyfill::none(),
        }
    }

    fn bounding_box(&self) -> AABB {
        let transform = self.transform.inverse();

        AABB::empty()
            .with_point(transform_point3a(&transform, &vec3a(0.5, 0.0, 0.5)))
            .with_point(transform_point3a(&transform, &vec3a(-0.5, 0.0, 0.5)))
            .with_point(transform_point3a(&transform, &vec3a(0.5, 0.0, -0.5)))
            .with_point(transform_point3a(&transform, &vec3a(-0.5, 0.0, -0.5)))
    }
}
