use sphere_audio_visualizer_core::glam::Vec3;

/// Implements a simple gradient with equal distant stops
#[derive(Debug)]
pub struct Gradient {
    colors: Vec<Vec3>,
}

impl Gradient {
    /// Creates a new instance using equal distant gradient stops
    pub fn new(colors: Vec<Vec3>) -> Self {
        Gradient { colors }
    }

    /// Retrives one color on the gradient. `t` should be between 0.0-1.0. if
    /// `t` is bigger or smaller the color of the first or last stop are used
    /// respectively.
    pub fn interpolate(&self, t: f32) -> Vec3 {
        let i = t * (self.colors.len() - 1) as f32;
        let fract = f32::fract(i);
        let floor = f32::floor(i);

        let a = self.colors[(floor as usize).min(self.colors.len() - 1).max(0)];
        let b = self.colors[(floor as usize + 1).min(self.colors.len() - 1).max(0)];

        return (a * (1.0 - fract)) + (b * fract);
    }
}
