use egui::Ui;

mod module;
mod rendering;
mod scene_converter;
mod simulation_resampler;
mod simulator;
mod spectrum;
mod visualizer;

pub use self::module::*;

/// An [`UiDrawer`] is used to draw the setting of its underling type with egui
pub trait UiDrawer {
    /// Is invoked to draw the setting of its underling type with egui
    fn ui(&mut self, ui: &mut Ui);
}
