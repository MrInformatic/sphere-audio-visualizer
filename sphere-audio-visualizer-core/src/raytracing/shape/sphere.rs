use glam::Vec3A;

#[cfg(target_arch = "spirv")]
use core::arch::asm;
#[cfg(target_arch = "spirv")]
use num_traits::Float;

use crate::{
    raytracing::{Ray, SurfaceProperties},
    utils::{
        math::{distance, dot, normalize, reflect, shlick},
        {OptionPolyfill, Uninit},
    },
};

use super::{Reflection, Shading, Shape, AABB};

/// Implements a sphere shape with glossy material.
#[repr(C, align(16))]
pub struct Sphere {
    position: Vec3A,
    color: Vec3A,
    radius: f32,
    n: f32,
}

impl Sphere {
    /// Creates a new Sphere shape
    /// - `position` Represents the position of the sphere in world space
    /// - `color` Represents the color of the sphere
    /// - `radius` Represents the radius of the sphere
    /// - `n` refractive factor of the sphere material
    pub fn new(position: Vec3A, color: Vec3A, radius: f32, n: f32) -> Self {
        Self {
            position,
            color,
            radius,
            n,
        }
    }
}

impl Sphere {
    fn sphere_hit(&self, ray: &Ray) -> OptionPolyfill<SphereHit> {
        let oc = ray.origin() - self.position;
        let direction = ray.direction();

        let a = dot(&direction, &direction);
        let b = 2.0 * dot(&oc, &direction);
        let c = dot(&oc, &oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        OptionPolyfill::new(discriminant >= 0.0, SphereHit { a, b, discriminant })
    }
}

impl Shape for Sphere {
    fn intersect(&self, ray: &Ray) -> OptionPolyfill<f32> {
        let sphere_hit = self.sphere_hit(ray);

        if sphere_hit.is_some() {
            unsafe { sphere_hit.unwrap() }.hit(ray)
        } else {
            OptionPolyfill::none()
        }
    }

    fn distance(&self, point: &Vec3A) -> f32 {
        distance(&self.position, point) - self.radius
    }

    fn shade(
        &self,
        ray: &Ray,
        hit: f32,
        intensity: impl Fn(&SurfaceProperties) -> Vec3A,
    ) -> Shading {
        let position = ray.point_at(hit);
        let normal = normalize(&(position - self.position));
        let ray_direction = ray.direction();

        let reflection_ray = Ray::new(position, reflect(&ray_direction, &normal), 0.0001, 1000.0);

        let surface = SurfaceProperties { position, normal };

        let shlick = shlick(&ray_direction, &normal, 1.0, self.n);

        Shading {
            emission: (intensity)(&surface) * self.color * (1.0 - shlick),
            reflection: OptionPolyfill::some(Reflection {
                ray: reflection_ray,
                color: Vec3A::splat(shlick),
            }),
        }
    }

    fn bounding_box(&self) -> AABB {
        AABB {
            min: self.position - self.radius,
            max: self.position + self.radius,
        }
    }
}

#[derive(Clone, Copy)]
struct SphereHit {
    a: f32,
    b: f32,
    discriminant: f32,
}

impl Uninit for SphereHit {
    fn uninit() -> Self {
        Self {
            a: 0.0,
            b: 0.0,
            discriminant: 0.0,
        }
    }
}

impl SphereHit {
    fn hit(&self, ray: &Ray) -> OptionPolyfill<f32> {
        let t = (-self.b - self.discriminant.sqrt()) / (2.0 * self.a);

        OptionPolyfill::new(ray.valid_t(t), t)
    }
}
