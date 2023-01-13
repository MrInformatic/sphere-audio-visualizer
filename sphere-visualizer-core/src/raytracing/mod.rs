//! Contains the implementation of the raytracing algorithm

use glam::{vec3a, Mat4, Vec2, Vec3A, Vec4};

use crate::utils::{
    math::{tonemap_filmic, transform_point3a, transform_vector3a},
    OptionPolyfill,
};

use self::{
    background::{Background, ConstantBackground},
    camera::{Camera, PerspectiveCamera},
    light::Light,
    shape::{Reflection, SceneArgs, Shading, ShapeGroup},
};

#[cfg(target_arch = "spirv")]
use num_traits::Float;

pub mod background;
pub mod camera;
pub mod light;
pub mod shape;

/// Implements a Ray
pub struct Ray {
    origin: Vec4,
    direction: Vec4,
}

impl Ray {
    /// Creates a new Ray.
    /// - `origin` Represents the origin of the ray
    /// - `direction` Represents the direction of the ray
    /// - `t_min` Represents the start point on the ray
    /// - `t_max` Represents the end point on the ray
    pub fn new(origin: Vec3A, direction: Vec3A, t_min: f32, t_max: f32) -> Self {
        Self {
            origin: origin.extend(t_min),
            direction: direction.extend(t_max),
        }
    }

    /// Gets the origin of the ray
    pub fn origin(&self) -> Vec3A {
        self.origin.truncate().into()
    }

    /// Gets the direction of the ray
    pub fn direction(&self) -> Vec3A {
        self.direction.truncate().into()
    }

    /// Gets the start point on the ray
    pub fn t_min(&self) -> f32 {
        self.origin.w
    }

    /// Gets the end point on the ray
    pub fn t_max(&self) -> f32 {
        self.direction.w
    }

    /// Transforms the ray using the passed matrix
    pub fn transform(&self, transform: &Mat4) -> Self {
        Self::new(
            transform_point3a(transform, &self.origin()),
            transform_vector3a(transform, &self.direction()),
            self.t_min(),
            self.t_max(),
        )
    }

    /// Checks weather a point is on this ray.
    pub fn valid_t(&self, t: f32) -> bool {
        t >= self.t_min() && t <= self.t_max()
    }

    /// Returns the position if a point on this ray
    pub fn point_at(&self, t: f32) -> Vec3A {
        self.origin() + self.direction() * t
    }
}

/// Stores properties of a point on a surface
pub struct SurfaceProperties {
    /// the position for the point
    pub position: Vec3A,
    /// the normal of the surface at that position
    pub normal: Vec3A,
}

/// Implements the path tracing algorithm
pub struct Raytracer<C: Camera, S: ShapeGroup, B: Background, L: Light> {
    camera: C,
    shape: S,
    background: B,
    light: L,
    bounces: u32,
}

impl<C: Camera, S: ShapeGroup, B: Background, L: Light> Raytracer<C, S, B, L> {
    /// Creates a new instance from shader parameters
    pub fn from_args(args: RaytracerArgs<C, B>, shape: S, light: L) -> Self {
        Self {
            camera: args.camera,
            shape,
            background: args.background,
            light,
            bounces: args.bounces,
        }
    }

    /// Samples the color of a pixel at the given position
    pub fn sample(&self, sample: &Vec2) -> Vec3A {
        let prime_ray = self.camera.prime_ray(sample);

        tonemap_filmic(&self.radiance(prime_ray))
    }

    /// Querries the radiance of the scene using a ray
    pub fn radiance(&self, ray: Ray) -> Vec3A {
        let mut radiance = vec3a(0.0, 0.0, 0.0);
        let mut reflection = Reflection {
            ray,
            color: vec3a(1.0, 1.0, 1.0),
        };

        for _ in 0..self.bounces {
            let hit = self.intersect(&reflection.ray);

            let shading = if hit.is_some() {
                self.shape_shade(&reflection.ray, unsafe { hit.unwrap() })
                // Safety: checked for some before
            } else {
                Shading {
                    emission: self.background.radiance(&reflection.ray.direction()),
                    reflection: OptionPolyfill::none(),
                }
            };

            radiance += reflection.color * shading.emission;

            if shading.reflection.is_some() {
                let Reflection { ray, color } = unsafe { shading.reflection.unwrap() };
                // Safety: checked for some before

                reflection = Reflection {
                    ray,
                    color: reflection.color * color,
                }
            } else {
                break;
            }
        }

        radiance
    }

    /// Returns the shading of a hit surface
    pub fn shape_shade(&self, ray: &Ray, hit: S::Hit) -> Shading {
        self.shape
            .shade(ray, hit, |surface| self.intensity(surface))
    }

    /// Returns the hit if the scene intersected with the given ray
    pub fn intersect(&self, ray: &Ray) -> OptionPolyfill<S::Hit> {
        self.shape.intersect(ray)
    }

    /// Returns the shortest distance of the given point to a surface of the
    /// scene.
    pub fn distance(&self, point: &Vec3A) -> f32 {
        self.shape.distance(point)
    }

    /// Returns the ambient occlusion of a point on a surface
    pub fn ambient_occlusion(&self, surface: &SurfaceProperties) -> f32 {
        let mut occlusion = 1.0;

        for i in 1u32..6 {
            let sample = i as f32;
            let offset = sample * 0.35;
            occlusion -= (offset - self.distance(&(surface.position + surface.normal * offset)))
                * 0.5f32.powf(sample);
        }

        occlusion.max(0.0)
    }

    /// returns the light instensity of a point on a surface
    pub fn intensity(&self, surface: &SurfaceProperties) -> Vec3A {
        self.background.intensity(&surface.normal) * self.ambient_occlusion(surface)
            + self
                .light
                .intensity(surface, |ray| self.intersect(ray).is_some())
    }
}

/// Stores the arguments of a raytracer used for shader parameters
#[repr(C, align(16))]
#[derive(Clone)]
pub struct RaytracerArgs<C: Camera, B: Background> {
    /// Represents the camera used
    pub camera: C,
    /// Represents the backgtound used
    pub background: B,
    /// Represents the amount of ray bounces that should be simulated
    pub bounces: u32,
}

/// Stores the arguments for raytracing used for shader parameters
#[repr(C, align(16))]
pub struct RaytracingArgsBundle<C: Camera, B: Background> {
    /// Represents the arguments of the raytracer
    pub raytracer_args: RaytracerArgs<C, B>,
    /// Represents the arguments of the scene
    pub scene_args: SceneArgs,
}

/// Defines a basic type configuration for raytracing
pub type BasicRaytracingArgsBundle = RaytracingArgsBundle<PerspectiveCamera, ConstantBackground>;
