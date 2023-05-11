use egui::containers::ComboBox;

use crate::rendering::wgpu::{
    ShadingLanguage, {MetaballsSettings, RaytracerSettings},
};

use super::UiDrawer;

impl ShadingLanguage {
    fn display_name(&self) -> &'static str {
        match self {
            ShadingLanguage::Rust => "Rust",
            ShadingLanguage::WGSL => "WGSL",
        }
    }
}

impl UiDrawer for RaytracerSettings {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Shading Language: ");
        ComboBox::from_id_source("Raytracer Shading Language")
            .selected_text(self.shading_language.display_name())
            .width(116.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.shading_language,
                    ShadingLanguage::Rust,
                    ShadingLanguage::Rust.display_name(),
                );
                ui.selectable_value(
                    &mut self.shading_language,
                    ShadingLanguage::WGSL,
                    ShadingLanguage::WGSL.display_name(),
                );
            });
        ui.end_row();
    }
}

impl UiDrawer for MetaballsSettings {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Shading Language: ");
        ComboBox::from_id_source("Metaballs Shading Language")
            .selected_text(self.shading_language.display_name())
            .width(116.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.shading_language,
                    ShadingLanguage::Rust,
                    ShadingLanguage::Rust.display_name(),
                );
                ui.selectable_value(
                    &mut self.shading_language,
                    ShadingLanguage::WGSL,
                    ShadingLanguage::WGSL.display_name(),
                );
            });
        ui.end_row();
    }
}
