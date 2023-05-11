use egui::Ui;

use crate::{
    module::Module,
    rendering::{
        wgpu::{Pipeline, RenderTarget},
        SceneConverter,
    },
    simulation::Simulator,
    visualizer::WGPUVisualizer,
};

use super::{module::draw_module, UiDrawer};

impl<S, SC, P, T> UiDrawer for WGPUVisualizer<S, SC, P, T>
where
    S: Simulator + Module + 'static,
    SC: SceneConverter<S::Scene> + Module + 'static,
    P: Pipeline<SC::Scene> + Module + 'static,
    T: RenderTarget + 'static,
    <S as Module>::Settings: UiDrawer,
    <SC as Module>::Settings: UiDrawer,
    <P as Module>::Settings: UiDrawer,
{
    fn ui(&mut self, ui: &mut Ui) {
        draw_module(&mut self.spectrum, ui);
        draw_module(&mut self.simulator, ui);
        draw_module(&mut self.scene_converter, ui);
        draw_module(&mut self.pipeline, ui);
    }
}
