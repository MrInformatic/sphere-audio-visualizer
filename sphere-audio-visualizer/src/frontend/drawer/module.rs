use egui::Ui;

use crate::module::Module;

use super::UiDrawer;

/// Utility function to draw the settings of a module with egui.
pub fn draw_module<'a, M: Module>(module: &'a mut M, ui: &mut Ui)
where
    M::Settings: UiDrawer,
{
    let mut settings = module.settings();

    settings.ui(ui);

    module.set_settings(settings);
}
