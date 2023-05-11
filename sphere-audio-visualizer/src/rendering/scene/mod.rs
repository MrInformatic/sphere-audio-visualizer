mod metaballs;
mod raytracing;

pub use self::{metaballs::*, raytracing::*};

/// A [`SceneConverter`] is used to convert one scene definition to a renderer
/// specific scene definition.
/// For Example, it is used to convert scene from the physics simulation to the
/// format used by the metaballs or raytracing renderer by e.g. adding lights,
/// cameras or whatever else a renderer needs for it's process.
pub trait SceneConverter<S> {
    /// The input scene type
    type Scene;

    /// Converts a scene to the renderer specific format
    fn convert(&self, scene: S, width: f32, height: f32) -> Self::Scene;
}
