use egui::{DragValue, Ui};

use crate::{simulation::SimulationResamplerSettings, UiDrawer};

impl UiDrawer for SimulationResamplerSettings {
    fn ui(&mut self, ui: &mut Ui) {
        ui.label("Simulator Frame Rate: ");
        ui.add_sized([124.0, 20.0], DragValue::new(&mut self.simulator_framerate));
        ui.end_row();
    }
}
