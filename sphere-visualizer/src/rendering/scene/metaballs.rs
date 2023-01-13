use std::time::Instant;

use sphere_visualizer_core::{
    glam::{vec2, vec3a, Vec2, Vec3A},
    metaballs::Metaball,
};

use crate::{module::Module, simulation::Sphere2D};

use super::SceneConverter;

fn hue_to_rgb(hue: f32) -> Vec3A {
    const THIRD_PI: f32 = std::f32::consts::PI / 3.0;

    let scaled_hue = hue / THIRD_PI;
    let interpolation = scaled_hue.fract();

    if scaled_hue >= 0.0 && scaled_hue < 1.0 {
        vec3a(1.0, interpolation, 0.0)
    } else if scaled_hue >= 1.0 && scaled_hue < 2.0 {
        vec3a(1.0 - interpolation, 1.0, 0.0)
    } else if scaled_hue >= 2.0 && scaled_hue < 3.0 {
        vec3a(0.0, 1.0, interpolation)
    } else if scaled_hue >= 3.0 && scaled_hue < 4.0 {
        vec3a(0.0, 1.0 - interpolation, 1.0)
    } else if scaled_hue >= 4.0 && scaled_hue < 5.0 {
        vec3a(interpolation, 0.0, 1.0)
    } else if scaled_hue >= 5.0 && scaled_hue < 6.0 {
        vec3a(1.0, 0.0, 1.0 - interpolation)
    } else {
        vec3a(1.0, 0.0, 0.0)
    }
}

/// Stores the scene definition for the metaballs renderer
pub struct MetaballsScene {
    pub(crate) color: Vec3A,
    pub(crate) size: Vec2,
    pub(crate) zoom: f32,
    pub(crate) metaballs: Vec<Metaball>,
}

impl MetaballsScene {
    /// Creates a new instance.
    /// - `color` defines the hallo color
    /// - `size` defines the size of the viewport
    /// - `zoom` defines the zoom factor of the camera
    pub fn new(color: Vec3A, size: Vec2, zoom: f32) -> Self {
        Self {
            color,
            size,
            zoom,
            metaballs: Vec::new(),
        }
    }

    /// Adds a metaball to the scene
    pub fn add_metaball(&mut self, metaball: Metaball) -> &mut Self {
        self.metaballs.push(metaball);
        self
    }

    /// Adds a metaball to the scene
    pub fn with_metaball(mut self, metaball: Metaball) -> Self {
        self.add_metaball(metaball);
        self
    }
}

/// Converts the 2D physics simultion result to the metaballs renderer scene
/// format
pub struct MetaballsSceneConverter {
    start: Instant,
}

impl Default for MetaballsSceneConverter {
    fn default() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl<S: IntoIterator<Item = Sphere2D>> SceneConverter<S> for MetaballsSceneConverter {
    type Scene = MetaballsScene;

    fn convert(&self, spheres: S, width: f32, height: f32) -> Self::Scene {
        let hue = self.start.elapsed().as_secs_f32();

        let mut scene = MetaballsScene::new(hue_to_rgb(hue % 6.0), vec2(width, height), 10.0);

        for sphere in spheres {
            scene.add_metaball(Metaball::new(
                vec2(sphere.position.x, sphere.position.y),
                sphere.radius,
            ));
        }

        scene
    }
}

impl Module for MetaballsSceneConverter {
    type Settings = MetaballsSceneConverterSettings;

    fn set_settings(&mut self, _settings: Self::Settings) -> &mut Self {
        self
    }

    fn settings(&self) -> Self::Settings {
        MetaballsSceneConverterSettings
    }
}

/// Stores the settings of the [`MetaballsSceneConverter`]
#[derive(Clone, Default)]
pub struct MetaballsSceneConverterSettings;
