use std::{marker::PhantomData, time::Instant};

use winit::window::Window;

use crate::{
    audio_analysis::{Samples, Spectrum},
    module::{Module, ModuleManager},
    rendering::{
        wgpu::{
            utils::CommandQueue,
            Pipeline, WGPURenderer, {EGUIRenderer, EGUIScene},
            {
                RenderTarget, RenderTargetTexture, SurfaceTarget,
                {OffscreenTarget, OffscreenTargetOutput, OutputFormat},
            },
        },
        SceneConverter,
    },
    simulation::Simulator,
};

use super::{OfflineVisualizer, OnlineVisualizer, Visualizer, VisualizerFactory};

/// A Visualizer Implementation for WGPU based visualizers.
pub struct WGPUVisualizer<S, SC, P, T>
where
    S: Simulator,
    SC: SceneConverter<S::Scene>,
    P: Pipeline<SC::Scene>,
    T: RenderTarget,
{
    pub(crate) spectrum: Spectrum,
    pub(crate) simulator: S,
    pub(crate) scene_converter: SC,
    pub(crate) pipeline: P,
    renderer: WGPURenderer,
    target: T,
    egui_renderer: EGUIRenderer,
    levels: Vec<f32>,
    last_instant: Instant,
}

impl<S, SC, P, T> WGPUVisualizer<S, SC, P, T>
where
    S: Simulator + 'static,
    SC: SceneConverter<S::Scene> + 'static,
    P: Pipeline<SC::Scene> + 'static,
    T: RenderTarget + 'static,
{
    fn visualize(
        &mut self,
        samples: Samples,
        width: u32,
        height: u32,
        egui_scene: Option<EGUIScene>,
    ) -> <T::Texture as RenderTargetTexture>::Output {
        let current_instant = Instant::now();
        let delta_time = current_instant.duration_since(self.last_instant);

        self.levels = self.spectrum.tick_par(samples).collect();

        self.simulator.step(delta_time.as_secs_f32(), &self.levels);

        let simulator_scene = self.simulator.scene();

        let renderer_scene =
            self.scene_converter
                .convert(simulator_scene, width as f32, height as f32);

        let output_texture = self
            .target
            .target_texture(width, height, &self.renderer.device());

        let mut command_queue = CommandQueue::new(self.renderer.queue());

        {
            let output_texture_view = output_texture.texture_view();

            self.pipeline.render(
                renderer_scene,
                self.renderer.device(),
                &mut command_queue,
                self.target.target_format(),
                &output_texture_view,
            );

            if let Some(egui_scene) = egui_scene {
                self.egui_renderer.render(
                    egui_scene,
                    self.renderer.device(),
                    &mut command_queue,
                    self.target.target_format(),
                    &output_texture_view,
                );
            }
        }

        let output = output_texture.present(self.renderer.device(), &mut command_queue);

        self.last_instant = current_instant;

        output
    }
}

impl<S, SC, P, T> Visualizer for WGPUVisualizer<S, SC, P, T>
where
    S: Simulator + Module + 'static,
    SC: SceneConverter<S::Scene> + Module + 'static,
    P: Pipeline<SC::Scene> + Module + 'static,
    T: RenderTarget + 'static,
{
    fn module_bin(self: Box<Self>, module_manager: &mut ModuleManager) {
        module_manager.insert(self.spectrum);
        module_manager.insert(self.simulator);
        module_manager.insert(self.scene_converter);
        module_manager.insert(self.pipeline);
        module_manager.insert_lossy(self.renderer);
        module_manager.insert_lossy(self.target);
        module_manager.insert_lossy(self.egui_renderer);
    }
}

impl<S, SC, P> OnlineVisualizer for WGPUVisualizer<S, SC, P, SurfaceTarget>
where
    S: Simulator + Module + 'static,
    SC: SceneConverter<S::Scene> + Module + 'static,
    P: Pipeline<SC::Scene> + Module + 'static,
{
    fn visualize(&mut self, samples: Samples, width: u32, height: u32, egui_scene: EGUIScene) {
        self.visualize(samples, width, height, Some(egui_scene))
    }
}

impl<S, SC, P> OfflineVisualizer for WGPUVisualizer<S, SC, P, OffscreenTarget>
where
    S: Simulator + Module + 'static,
    SC: SceneConverter<S::Scene> + Module + 'static,
    P: Pipeline<SC::Scene> + Module + 'static,
{
    fn visualize(&mut self, samples: Samples, width: u32, height: u32) -> OffscreenTargetOutput {
        self.visualize(samples, width, height, None)
    }
}

/// The [`VisualizerFactory`] for the [`WGPUVisualizer`]
pub struct WGPUVisualizerFactory<S, SC, P>(PhantomData<(S, SC, P)>);

impl<S, SC, P> VisualizerFactory for WGPUVisualizerFactory<S, SC, P>
where
    S: Simulator + Module + 'static,
    SC: SceneConverter<S::Scene> + Module + 'static,
    P: Pipeline<SC::Scene> + Module + 'static,
{
    type OnlineVisualizer = WGPUVisualizer<S, SC, P, SurfaceTarget>;
    type OfflineVisualizer = WGPUVisualizer<S, SC, P, OffscreenTarget>;

    fn new_online(window: &Window, mut module_manager: ModuleManager) -> Self::OnlineVisualizer {
        let spectrum = module_manager.extract::<Spectrum>();
        let simulator = module_manager.extract::<S>();
        let scene_converter = module_manager.extract::<SC>();
        let pipeline = module_manager.extract::<P>();

        let (renderer, target) = match (
            module_manager.extract_optional::<WGPURenderer>(),
            module_manager.extract_optional::<SurfaceTarget>(),
        ) {
            (Some(renderer), Some(surface_target)) => (renderer, surface_target),
            _ => pollster::block_on(WGPURenderer::onscreen(window, None)).unwrap(),
        };

        let egui_renderer = module_manager.extract_or_default::<EGUIRenderer>();

        Self::OnlineVisualizer {
            spectrum,
            simulator,
            scene_converter,
            pipeline,
            renderer,
            target,
            egui_renderer,
            levels: vec![],
            last_instant: Instant::now(),
        }
    }

    fn new_offline(
        format: OutputFormat,
        mut module_manager: ModuleManager,
    ) -> Self::OfflineVisualizer {
        let spectrum = module_manager.extract::<Spectrum>();
        let simulator = module_manager.extract::<S>();
        let scene_converter = module_manager.extract::<SC>();
        let pipeline = module_manager.extract::<P>();

        let renderer = module_manager
            .extract_or_else(|| pollster::block_on(WGPURenderer::offscreen(None)).unwrap());

        let target = module_manager
            .extract_optional::<OffscreenTarget>()
            .filter(|target| target.format() == format)
            .unwrap_or_else(|| OffscreenTarget::new(format));

        let egui_renderer = module_manager.extract_or_default::<EGUIRenderer>();

        Self::OfflineVisualizer {
            spectrum,
            simulator,
            scene_converter,
            pipeline,
            renderer,
            target,
            egui_renderer,
            levels: vec![],
            last_instant: Instant::now(),
        }
    }
}
