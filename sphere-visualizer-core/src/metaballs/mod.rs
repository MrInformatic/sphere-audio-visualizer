//! Contains the definition of the metaballs algorithm

use glam::{Vec2, Vec3A};

use crate::utils::math::{dot2, inverse_sqrt};

/// Stores the properties of a Metaball
#[repr(C, align(16))]
pub struct Metaball {
    position: Vec2,
    radius: f32,
}

impl Metaball {
    /// Creates a new Instance
    /// - `position` the position of the metaball
    /// - `radius` the radius of the metaball
    pub fn new(position: Vec2, radius: f32) -> Self {
        Self { position, radius }
    }
}

/// Stores properties of the metaball scene
pub struct Metaballs<'a> {
    color: Vec3A,
    size: Vec2,
    zoom: f32,
    metaballs: &'a [Metaball],
}

/// Stores properties of the metaball scene used for shader parameters
#[repr(C, align(16))]
#[derive(Clone)]
pub struct MetaballsArgs {
    /// Represents the color of the halo
    pub color: Vec3A,
    /// Represents the size of the viewport in pixels
    pub size: Vec2,
    /// Represents the zoom factor of the viewport
    pub zoom: f32,
}

impl<'a> Metaballs<'a> {
    /// Creates a new instance from shader parameters
    pub fn from_args(args: MetaballsArgs, metaballs: &'a [Metaball]) -> Self {
        Self {
            color: args.color,
            size: args.size,
            zoom: args.zoom,
            metaballs,
        }
    }

    /// Samples the color at the given sceen position
    pub fn sample(&self, sample: &Vec2) -> Vec3A {
        let mut value: f32 = 0.0;

        let position = (*sample / self.size * 2.0 - 1.0) * self.zoom;

        for id in 0..self.metaballs.len() {
            let oc = position - self.metaballs[id].position;
            let radius = self.metaballs[id].radius;

            value = value + inverse_sqrt(dot2(&oc, &oc)) * radius * 0.05;
        }

        if value <= 0.75 {
            self.color * value
        } else {
            Vec3A::splat(1.0)
        }
    }
}
