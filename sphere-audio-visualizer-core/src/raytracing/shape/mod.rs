//! Contains implementations of of the supported raytracing shapes

use glam::{vec3a, vec4, Vec3A};

use crate::utils::{OptionPolyfill, Uninit};

pub use self::{rect::*, sphere::*};

use super::{Ray, SurfaceProperties};

mod rect;
mod sphere;

/// Stores the shading of a surface
pub struct Shading {
    /// Represents the color and intensity of the emmited light
    pub emission: Vec3A,
    /// Represents a reflection on a surface
    pub reflection: OptionPolyfill<Reflection>,
}

/// Stores reflection properties
pub struct Reflection {
    /// The Ray emmited by the refelction
    pub ray: Ray,
    /// The color of the relected surface
    pub color: Vec3A,
}

impl Uninit for Reflection {
    fn uninit() -> Self {
        Self {
            ray: Ray {
                origin: vec4(0.0, 0.0, 0.0, 0.0),
                direction: vec4(0.0, 0.0, 0.0, 0.0),
            },
            color: vec3a(0.0, 0.0, 0.0),
        }
    }
}

/// Stores diffuse surface properties
pub struct Diffusion {
    /// Represents surface properties
    pub surface: SurfaceProperties,
    /// Represents the color of the surface
    pub color: Vec3A,
}

impl Uninit for Diffusion {
    fn uninit() -> Self {
        Self {
            surface: SurfaceProperties {
                position: vec3a(0.0, 0.0, 0.0),
                normal: vec3a(0.0, 0.0, 0.0),
            },
            color: vec3a(0.0, 0.0, 0.0),
        }
    }
}

/// A Shape can be intersected by rays. It also already contains the material
/// for shading.
pub trait Shape: Send + Sync {
    /// Returns the intersection point of this shape with a ray if they
    /// intersect.
    fn intersect(&self, ray: &Ray) -> OptionPolyfill<f32>;

    /// Returns the shortest distance from the passed point to the surface of
    /// this shape
    fn distance(&self, point: &Vec3A) -> f32;

    /// Returns the shading of a hit event. `intensity` is used for diffuse
    /// lighting
    fn shade(
        &self,
        ray: &Ray,
        hit: f32,
        intensity: impl Fn(&SurfaceProperties) -> Vec3A,
    ) -> Shading;

    /// Returns the bounding box of the shape
    fn bounding_box(&self) -> AABB;
}

/// A ShapeGroup can be intersected by rays. It manages multible shapes.
/// Therefore the Hit also tracks information over the intersected shape to
/// infer the intersected shape when shading.
pub trait ShapeGroup {
    /// The Hit type. Also contains information over the intersected shape
    type Hit;

    /// Intersects a ray with the shapes in the group if one intersects. Also
    /// returns information about the intersected shape.
    fn intersect(&self, ray: &Ray) -> OptionPolyfill<Self::Hit>;

    /// Returns the shortest distance from the passed point to the surface of
    /// the shapes in the group
    fn distance(&self, point: &Vec3A) -> f32;

    /// Returns the shading of a hit event. `intensity` is used for diffuse
    /// lighting
    fn shade(
        &self,
        ray: &Ray,
        hit: Self::Hit,
        intensity: impl Fn(&SurfaceProperties) -> Vec3A,
    ) -> Shading;
}

/// A Shapegroup of shapes with the same type
pub struct Group<'a, S: Shape>(&'a [S]);

/// A hit on a [`Group`]
pub struct GroupHit {
    hit: f32,
    id: usize,
}

impl GroupHit {
    /// Return the smaller of two [`GroupHit`]s
    pub fn min(self, other: Self) -> Self {
        if self.hit > other.hit {
            other
        } else {
            self
        }
    }
}

impl Uninit for GroupHit {
    fn uninit() -> Self {
        Self {
            hit: f32::INFINITY,
            id: usize::MAX,
        }
    }
}

impl<'a, S: Shape> ShapeGroup for Group<'a, S> {
    type Hit = GroupHit;

    fn intersect(&self, ray: &Ray) -> OptionPolyfill<Self::Hit> {
        let mut is_hit = false;
        let mut nearest_hit = GroupHit {
            hit: ray.t_max(),
            id: 0,
        };

        for id in 0..self.0.len() {
            let hit = self.0[id].intersect(ray);

            unsafe {
                let hit_is_some = hit.is_some();
                let hit = hit.unwrap();

                is_hit = is_hit || hit_is_some;

                if hit_is_some && nearest_hit.hit > hit {
                    nearest_hit = GroupHit { hit, id };
                }
            }
        }

        OptionPolyfill::new(is_hit, nearest_hit)
    }

    fn distance(&self, point: &Vec3A) -> f32 {
        let mut distance = f32::INFINITY;

        for id in 0..self.0.len() {
            distance = distance.min(self.0[id].distance(point))
        }

        distance
    }

    fn shade(
        &self,
        ray: &Ray,
        hit: Self::Hit,
        intensity: impl Fn(&SurfaceProperties) -> Vec3A,
    ) -> Shading {
        self.0[hit.id].shade(ray, hit.hit, intensity)
    }
}

/// An Axis Aligned Bounding Box
#[repr(C, align(16))]
#[derive(Clone)]
pub struct AABB {
    min: Vec3A,
    max: Vec3A,
}

impl AABB {
    /// Creates an empty bounding box
    pub fn empty() -> AABB {
        Self {
            min: Vec3A::splat(f32::INFINITY),
            max: Vec3A::splat(f32::NEG_INFINITY),
        }
    }

    /// Creates an bounding box containing everything
    pub fn all() -> AABB {
        Self {
            min: Vec3A::splat(f32::NEG_INFINITY),
            max: Vec3A::splat(f32::INFINITY),
        }
    }

    fn aabb_intersection(&self, ray: &Ray) -> AABBIntersection {
        let di = 1.0 / ray.direction();

        let t1 = (self.min - ray.origin()) * di;
        let t2 = (self.max - ray.origin()) * di;

        let tmin = t1.min(t2);
        let tmax = t1.max(t2);

        let tmin = tmin.x.max(tmin.y.max(tmin.z));
        let tmax = tmax.x.min(tmax.y.min(tmax.z));

        AABBIntersection { tmin, tmax }
    }

    /// intersects a ray with the bounding box
    pub fn intersection(&self, ray: &Ray) -> OptionPolyfill<f32> {
        self.aabb_intersection(ray).intersection(ray)
    }

    /// returns weather a ray intersects with the bounding box
    pub fn intersect(&self, ray: &Ray) -> bool {
        self.aabb_intersection(ray).intersect(ray)
    }

    /// expands the bounding box to contain this point
    pub fn add_point(&mut self, point: Vec3A) -> &mut Self {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
        self
    }

    /// expands the bounding box to contain this point
    pub fn with_point(mut self, point: Vec3A) -> Self {
        self.add_point(point);
        self
    }

    /// expands the bounding box to contain another bounding box
    pub fn add_aabb(&mut self, aabb: &AABB) -> &mut Self {
        self.min = self.min.min(aabb.min);
        self.max = self.max.max(aabb.max);
        self
    }

    /// expands the bounding box to contain another bounding box
    pub fn with_aabb(mut self, aabb: &AABB) -> Self {
        self.add_aabb(aabb);
        self
    }
}

struct AABBIntersection {
    tmin: f32,
    tmax: f32,
}

impl AABBIntersection {
    fn intersection(&self, ray: &Ray) -> OptionPolyfill<f32> {
        let valid = self.tmax >= self.tmin;

        if ray.valid_t(self.tmin) {
            OptionPolyfill::new(valid, self.tmin)
        } else if ray.valid_t(self.tmax) {
            OptionPolyfill::new(valid, self.tmax)
        } else {
            OptionPolyfill::none()
        }
    }

    fn intersect(&self, ray: &Ray) -> bool {
        self.tmax >= self.tmin && (ray.valid_t(self.tmin) || ray.valid_t(self.tmax))
    }
}

/// A wrapper for [`Group`] but with [`AABB`]
pub struct BoundingBoxGroup<'a, S: Shape> {
    bounding_box: AABB,
    group: Group<'a, S>,
}

impl<'a, S: Shape> ShapeGroup for BoundingBoxGroup<'a, S> {
    type Hit = GroupHit;

    fn intersect(&self, ray: &Ray) -> OptionPolyfill<Self::Hit> {
        if self.bounding_box.intersect(ray) {
            self.group.intersect(ray)
        } else {
            OptionPolyfill::none()
        }
    }

    fn distance(&self, point: &Vec3A) -> f32 {
        self.group.distance(point)
    }

    fn shade(
        &self,
        ray: &Ray,
        hit: Self::Hit,
        intensity: impl Fn(&SurfaceProperties) -> Vec3A,
    ) -> Shading {
        self.group.shade(ray, hit, intensity)
    }
}

/// Represents the geometry of an scene. All supported shapes should be
/// represented by a [`BoundingBoxGroup`] Field in this struct.
pub struct Scene<'a, 'b> {
    /// The [`BoundingBoxGroup`] for [`Sphere`]
    pub spheres: BoundingBoxGroup<'a, Sphere>,
    /// The [`BoundingBoxGroup`] for [`Rect`]
    pub rects: BoundingBoxGroup<'b, Rect>,
}

/// Indentifies the different Shape types we support
pub enum ShapeType {
    /// Represents a [`Sphere`]
    Sphere,
    /// Represents a [`Rect`]
    Rect,
}

/// A hit on a [`Scene`]
pub struct SceneHit {
    hit: GroupHit,
    shape_type: ShapeType,
}

impl SceneHit {
    /// Return the smaller of two [`SceneHit`]s
    pub fn min(self, other: Self) -> Self {
        if self.hit.hit > other.hit.hit {
            other
        } else {
            self
        }
    }
}

impl Uninit for SceneHit {
    fn uninit() -> Self {
        Self {
            hit: Uninit::uninit(),
            shape_type: ShapeType::Sphere,
        }
    }
}

impl<'a, 'b> Scene<'a, 'b> {
    /// Creates a scene from shader inputs.
    pub fn from_args(args: SceneArgs, spheres: &'a [Sphere], rects: &'b [Rect]) -> Self {
        Self {
            spheres: BoundingBoxGroup {
                group: Group(spheres),
                bounding_box: args.spheres_bounding_box.clone(),
            },
            rects: BoundingBoxGroup {
                group: Group(rects),
                bounding_box: args.rects_bounding_box.clone(),
            },
        }
    }
}

impl<'a, 'b> ShapeGroup for Scene<'a, 'b> {
    type Hit = SceneHit;

    fn intersect(&self, ray: &Ray) -> OptionPolyfill<Self::Hit> {
        let mut is_hit = false;
        let mut hit = SceneHit {
            hit: GroupHit {
                hit: ray.t_max(),
                id: 0,
            },
            shape_type: ShapeType::Sphere,
        };

        let sphere_hit = self.spheres.intersect(ray);

        unsafe {
            let sphere_is_hit = sphere_hit.is_some();
            let sphere_hit = sphere_hit.unwrap();

            is_hit = is_hit || sphere_is_hit;
            if sphere_is_hit && hit.hit.hit > sphere_hit.hit {
                hit = SceneHit {
                    hit: sphere_hit,
                    shape_type: ShapeType::Sphere,
                };
            }
        }

        let rect_hit = self.rects.intersect(ray);

        unsafe {
            let rect_is_hit = rect_hit.is_some();
            let rect_hit = rect_hit.unwrap();

            is_hit = is_hit || rect_is_hit;
            if rect_is_hit && hit.hit.hit > rect_hit.hit {
                hit = SceneHit {
                    hit: rect_hit,
                    shape_type: ShapeType::Rect,
                };
            }
        }

        OptionPolyfill::new(is_hit, hit)
    }

    fn distance(&self, point: &Vec3A) -> f32 {
        self.spheres.distance(point).min(self.rects.distance(point))
    }

    fn shade(
        &self,
        ray: &Ray,
        hit: Self::Hit,
        intensity: impl Fn(&SurfaceProperties) -> Vec3A,
    ) -> Shading {
        match hit.shape_type {
            ShapeType::Sphere => self.spheres.shade(ray, hit.hit, intensity),
            ShapeType::Rect => self.rects.shade(ray, hit.hit, intensity),
        }
    }
}

/// Stores Scene parameters used for shaders.
#[repr(C, align(16))]
#[derive(Clone)]
pub struct SceneArgs {
    /// bounding box from the [Rect] [Group]
    pub rects_bounding_box: AABB,
    /// bounding box from the [Sphere] [Group]
    pub spheres_bounding_box: AABB,
}
