//! Contains implementation of the 2D physics simulation

use nalgebra_glm::{vec2, Vec2};
use rand::{thread_rng, Rng};
use rapier2d::prelude::{
    Ball, BroadPhase, CCDSolver, ColliderBuilder, ColliderHandle, ColliderSet, EventHandler,
    IntegrationParameters, IslandManager, JointSet, NarrowPhase, PhysicsHooks, PhysicsPipeline,
    RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
};

use crate::module::Module;

use super::{SimulationSettings, Simulator, SPHERE_MIN_RADIUS};

/// Stores data from a 2D sphere
pub struct Sphere2D {
    /// The radius of the sphere
    pub radius: f32,
    /// The position of the sphere
    pub position: Vec2,
}

struct SphereData2D {
    origin: Vec2,
    rigid_body: RigidBodyHandle,
    collider: ColliderHandle,
}

/// Implements the 2D Physics simulation
pub struct Simulation2D {
    physics_pipeline: PhysicsPipeline,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    joint_set: JointSet,
    ccd_solver: CCDSolver,
    spheres: Vec<SphereData2D>,
    min_radius: f32,
}

impl Simulation2D {
    /// Creates a new instance
    pub fn new(min_radius: f32) -> Self {
        let physics_pipeline = PhysicsPipeline::new();

        let integration_parameters = IntegrationParameters::default();

        let island_manager = IslandManager::new();

        let broad_phase = BroadPhase::new();

        let narrow_phase = NarrowPhase::new();

        let collider_set = ColliderSet::new();

        let rigid_body_set = RigidBodySet::new();

        let joint_set = JointSet::new();

        let ccd_solver = CCDSolver::new();

        let spheres = vec![];

        Self {
            physics_pipeline,
            integration_parameters,
            island_manager,
            broad_phase,
            narrow_phase,
            rigid_body_set,
            collider_set,
            joint_set,
            ccd_solver,
            spheres,
            min_radius,
        }
    }

    /// Gets the min radius of the spheres
    pub fn min_radius(&self) -> f32 {
        self.min_radius
    }

    /// Sets the min radius of the spheres
    pub fn set_min_radius(&mut self, min_radius: f32) -> &mut Self {
        self.min_radius = min_radius;
        self
    }

    /// Sets the min radius of the spheres
    pub fn with_min_radius(mut self, min_radius: f32) -> Self {
        self.set_min_radius(min_radius);
        self
    }
}

impl Simulator for Simulation2D {
    type Scene = Vec<Sphere2D>;

    fn step(&mut self, delta_time: f32, levels: &[f32]) {
        let gravity = vec2(0.0f32, 0.0f32);

        let sphere_count = levels.len();

        let offset = (sphere_count - 1) as f32 * 0.5;
        let factor = 16.0 / sphere_count as f32;

        if sphere_count < self.spheres.len() {
            unsafe { self.spheres.set_len(sphere_count) }
        }

        let mut rng = thread_rng();

        for (i, level) in levels.iter().enumerate() {
            let radius = self.min_radius.max(*level * 2.0);

            match self.spheres.get_mut(i) {
                Some(sphere) => {
                    sphere.origin.x = (i as f32 - offset) * factor;

                    if let Some(collider) = self.collider_set.get_mut(sphere.collider) {
                        if let Some(sphere) = collider.shape_mut().downcast_mut::<Ball>() {
                            sphere.radius = radius;
                        }
                    }

                    if let Some(rigid_body) = self.rigid_body_set.get_mut(sphere.rigid_body) {
                        let current_position = rigid_body.translation().clone();

                        rigid_body.set_translation(
                            sphere.origin
                                + (current_position - sphere.origin)
                                    * (1.0 - f32::powf(0.99, delta_time)),
                            true,
                        );
                    }
                }
                None => {
                    let origin = vec2((i as f32 - offset) * factor, rng.gen_range(-0.05..0.05));

                    let rigid_body = RigidBodyBuilder::new_dynamic().translation(origin).build();

                    let rigid_body = self.rigid_body_set.insert(rigid_body);

                    let collider = ColliderBuilder::ball(radius)
                        .friction(0.0)
                        .density(1.0)
                        .build();

                    let collider = self.collider_set.insert_with_parent(
                        collider,
                        rigid_body,
                        &mut self.rigid_body_set,
                    );

                    self.spheres.push(SphereData2D {
                        origin,
                        rigid_body,
                        collider,
                    });
                }
            }
        }

        self.integration_parameters.dt = delta_time;

        self.physics_pipeline.step(
            &gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.joint_set,
            &mut self.ccd_solver,
            &() as &dyn PhysicsHooks<RigidBodySet, ColliderSet>,
            &() as &dyn EventHandler,
        );
    }

    fn scene(&self) -> Self::Scene {
        self.spheres
            .iter()
            .filter_map(|sphere| {
                let rigid_body = self.rigid_body_set.get(sphere.rigid_body)?;
                let collider = self.collider_set.get(sphere.collider)?;

                let sphere = collider.shape().downcast_ref::<Ball>()?;

                Some(Sphere2D {
                    radius: sphere.radius,
                    position: rigid_body.translation().clone(),
                })
            })
            .collect()
    }
}

impl Default for Simulation2D {
    fn default() -> Self {
        Self::new(SPHERE_MIN_RADIUS)
    }
}

impl Module for Simulation2D {
    type Settings = SimulationSettings;

    fn set_settings(&mut self, settings: Self::Settings) -> &mut Self {
        self.set_min_radius(settings.min_radius)
    }

    fn settings(&self) -> Self::Settings {
        SimulationSettings {
            min_radius: self.min_radius(),
        }
    }
}
