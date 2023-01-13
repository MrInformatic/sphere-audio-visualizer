use glam::Vec3A;

use super::Background;

/// A Background which emits the same amount of light into all directions.
#[repr(C, align(16))]
#[derive(Clone)]
pub struct ConstantBackground {
    ///
    pub color: Vec3A,
}

impl ConstantBackground {
    /// Creates a new instance.
    /// - `color` Represents the color and intensity of the light that is emmited by the background.
    pub fn new(color: Vec3A) -> Self {
        Self { color }
    }
}

impl Background for ConstantBackground {
    fn radiance(&self, _direction: &Vec3A) -> Vec3A {
        self.color
    }

    fn intensity(&self, _normal: &Vec3A) -> Vec3A {
        self.color
    }
}
