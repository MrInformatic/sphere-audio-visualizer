use sphere_visualizer_core::{
    glam::{vec2, vec3, vec3a, Mat4, Vec3, Vec3A},
    raytracing::{
        background::{Background, ConstantBackground},
        camera::{Camera, PerspectiveCamera},
        light::{Light, PointLight},
        shape::{Rect, Shape, Sphere, AABB},
    },
};

use crate::{
    module::Module,
    simulation::Sphere3D,
    utils::{Gradient, TypeMap},
};

use super::SceneConverter;

const SPHERE_N: f32 = 1.45;

/// Stores the scene definition for the raytracer renderer. Not every camera,
/// background, shape or lights combination might be supported by the target
/// renderer.
pub struct RaytracerScene<C: Camera, B: Background> {
    pub(crate) camera: C,
    pub(crate) shapes: TypeMap,
    pub(crate) background: B,
    pub(crate) lights: TypeMap,
    pub(crate) bounces: u32,
}

pub(crate) struct ShapeCollection<S: Shape> {
    pub(crate) bounding_box: AABB,
    pub(crate) shapes: Vec<S>,
}

impl<S: Shape> ShapeCollection<S> {
    fn new() -> Self {
        Self {
            bounding_box: AABB::empty(),
            shapes: Vec::new(),
        }
    }

    fn add(&mut self, shape: S) -> &mut Self {
        self.bounding_box.add_aabb(&shape.bounding_box());
        self.shapes.push(shape);
        self
    }

    pub(crate) fn shapes(&self) -> &[S] {
        &self.shapes
    }

    pub(crate) fn bounding_box(&self) -> &AABB {
        &self.bounding_box
    }
}

impl<C: Camera, B: Background> RaytracerScene<C, B> {
    /// Create a new instance
    /// - `camera` the camera used
    /// - `background` the background used
    /// - `bounces` the amount of ray bounces to simulate
    pub fn new(camera: C, background: B, bounces: u32) -> Self {
        Self {
            camera,
            shapes: TypeMap::new(),
            background,
            lights: TypeMap::new(),
            bounces,
        }
    }

    /// Adds a shape to the scene
    pub fn add_shape<S: Shape + 'static>(&mut self, shape: S) -> &mut Self {
        self.shapes
            .entry()
            .or_insert_with(ShapeCollection::new)
            .add(shape);
        self
    }

    /// Adds a shape to the scene
    pub fn with_shape<S: Shape + 'static>(mut self, shape: S) -> Self {
        self.add_shape(shape);
        self
    }

    pub(crate) fn shapes<S: Shape + 'static>(&mut self) -> Option<&ShapeCollection<S>> {
        self.shapes.get()
    }

    /// Adds a light to the scene
    pub fn add_ligth<L: Light + 'static>(&mut self, light: L) -> &mut Self {
        self.lights.entry().or_insert_with(Vec::new).push(light);
        self
    }

    /// Adds a light to the scene
    pub fn with_light<L: Light + 'static>(mut self, light: L) -> Self {
        self.add_ligth(light);
        self
    }

    pub(crate) fn lights_mut<L: Light + 'static>(&mut self) -> Option<&Vec<L>> {
        self.lights.get()
    }
}

/// Defines the raytracer scene type that is supported by the basic raytracer
/// implementation.
pub type BasicRaytracerScene = RaytracerScene<PerspectiveCamera, ConstantBackground>;

/// Converts the 3D physics simultion result to the raytracer renderer scene
/// format
pub struct RaytracerSceneConverter {
    color_ramp: Gradient,
    n: f32,
}

impl Default for RaytracerSceneConverter {
    fn default() -> Self {
        let color_ramp = Gradient::new(vec![
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.5, 0.0, 1.0),
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 0.5, 1.0),
            vec3(0.0, 0.1, 1.0),
        ]);

        Self {
            color_ramp,
            n: SPHERE_N,
        }
    }
}

impl<S: IntoIterator<Item = Sphere3D>> SceneConverter<S> for RaytracerSceneConverter {
    type Scene = BasicRaytracerScene;

    fn convert(&self, spheres: S, width: f32, height: f32) -> Self::Scene {
        let mut scene = BasicRaytracerScene::new(
            PerspectiveCamera::new(
                Mat4::from_translation(vec3(0.0f32, 0.0f32, -10.0f32)),
                vec2(width, height),
                std::f32::consts::PI / 4.0,
                0.0001,
                1000.0,
            ),
            ConstantBackground {
                color: Vec3A::splat(1.0),
            },
            5,
        );

        for Sphere3D { position, radius } in spheres {
            let color = self.color_ramp.interpolate(radius as f32);

            scene.add_shape(Sphere::new(
                vec3a(position.x, position.y, position.z),
                vec3a(color.x, color.y, color.z),
                radius,
                self.n,
            ));
        }

        let rect_transform = Mat4::from_translation(vec3(-10.0, 10.0, -10.0))
            * Mat4::from_scale(Vec3::splat(10.0))
            * Mat4::from_rotation_y(std::f32::consts::PI * 1.25)
            * Mat4::from_rotation_x(std::f32::consts::PI * 0.25);

        scene
            .with_shape(Rect::new(rect_transform.inverse(), Vec3A::splat(10.0)))
            .with_light(PointLight::new(
                vec3a(-10.0, 10.0, -10.0),
                Vec3A::splat(400.0),
            ))
    }
}

impl Module for RaytracerSceneConverter {
    type Settings = RaytracerSceneConverterSettings;

    fn set_settings(&mut self, _settings: Self::Settings) -> &mut Self {
        self
    }

    fn settings(&self) -> Self::Settings {
        RaytracerSceneConverterSettings
    }
}

/// Stores the settings of the [`RaytracerSceneConverter`]
#[derive(Default, Clone)]
pub struct RaytracerSceneConverterSettings;
