//! Contains implementation of the physics simulation

pub use self::{scene_2d::*, scene_3d::*};

mod scene_2d;
mod scene_3d;

const SPHERE_MIN_RADIUS: f32 = 0.1;

/// Stores the settings of the [`Simulation2D`] [`Simulation3D`] physics simulations
#[derive(Clone)]
pub struct SimulationSettings {
    /// The minimum radius for the spheres in the simulation.
    pub min_radius: f32,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            min_radius: SPHERE_MIN_RADIUS,
        }
    }
}

/// A [`Simulator`] is used to turn the level output from the audio analysis
/// into a scene using physics simulation.
pub trait Simulator {
    /// The Output Scene Type used.
    type Scene;

    /// Advances the simulation
    fn step(&mut self, delta_time: f32, levels: &[f32]);

    /// Creates as snapshot of the current scene
    fn scene(&self) -> Self::Scene;
}
