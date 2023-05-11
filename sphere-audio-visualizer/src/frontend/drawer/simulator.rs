use egui::widgets::DragValue;

use crate::simulation::SimulationSettings;

use super::UiDrawer;

impl UiDrawer for SimulationSettings {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Min Radius: ");
        ui.add_sized([124.0, 20.0], DragValue::new(&mut self.min_radius));
        ui.end_row();
    }
}
