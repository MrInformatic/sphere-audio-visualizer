use egui::{DragValue, Ui};

use crate::audio_analysis::SpectrumSettings;

use super::UiDrawer;

impl UiDrawer for SpectrumSettings {
    fn ui(&mut self, ui: &mut Ui) {
        ui.label("Count: ");
        ui.add_sized([124.0, 20.0], DragValue::new(&mut self.count));
        ui.end_row();

        ui.label("High: ");
        ui.add_sized([124.0, 20.0], DragValue::new(&mut self.high));
        ui.end_row();

        ui.label("Low: ");
        ui.add_sized([124.0, 20.0], DragValue::new(&mut self.low));
        ui.end_row();

        ui.label("Threshold: ");
        ui.add_sized([124.0, 20.0], DragValue::new(&mut self.threshold));
        ui.end_row();

        ui.label("Attack: ");
        ui.add_sized([124.0, 20.0], DragValue::new(&mut self.attack));
        ui.end_row();

        ui.label("Release: ");
        ui.add_sized([124.0, 20.0], DragValue::new(&mut self.release));
        ui.end_row();
    }
}
