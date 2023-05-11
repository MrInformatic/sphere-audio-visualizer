use egui::Ui;

use crate::rendering::{MetaballsSceneConverterSettings, RaytracerSceneConverterSettings};

use super::UiDrawer;

impl UiDrawer for MetaballsSceneConverterSettings {
    fn ui(&mut self, _ui: &mut Ui) {}
}

impl UiDrawer for RaytracerSceneConverterSettings {
    fn ui(&mut self, _ui: &mut Ui) {}
}
