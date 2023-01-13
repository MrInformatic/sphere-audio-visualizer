use egui::{epaint::ClippedShape, ClippedMesh, Context, TexturesDelta};
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use wgpu::{Device, TextureFormat, TextureView};

use crate::rendering::wgpu::{utils::CommandQueue, Pipeline};

struct EGUIRenderPipeline {
    egui_render_pass: RenderPass,
    output_format: TextureFormat,
}

impl EGUIRenderPipeline {
    fn new(device: &Device, output_format: TextureFormat) -> Self {
        Self {
            egui_render_pass: RenderPass::new(device, output_format, 1),
            output_format,
        }
    }
}

/// A [`Pipeline`] for rendering [`EGUIScene`]
#[derive(Default)]
pub struct EGUIRenderer {
    egui_render_pipeline: Option<EGUIRenderPipeline>,
}

/// The Scene representation for the [`EGUIRenderer`]
pub struct EGUIScene {
    screen_descriptor: ScreenDescriptor,
    paint_jobs: Vec<ClippedMesh>,
    textures: TexturesDelta,
}

impl EGUIScene {
    /// Creates a new Instance from the for egui rendering relevant data.
    pub fn new(
        context: &Context,
        textures_delta: TexturesDelta,
        shapes: Vec<ClippedShape>,
        screen_descriptor: ScreenDescriptor,
    ) -> Self {
        let paint_jobs = context.tessellate(shapes);

        Self {
            screen_descriptor,
            paint_jobs,
            textures: textures_delta,
        }
    }
}

impl Pipeline<EGUIScene> for EGUIRenderer {
    fn render(
        &mut self,
        scene: EGUIScene,
        device: &Device,
        command_queue: &mut CommandQueue,
        output_format: TextureFormat,
        output_texture: &TextureView,
    ) {
        let egui_render_pass = {
            let egui_render_pipeline = self
                .egui_render_pipeline
                .get_or_insert_with(|| EGUIRenderPipeline::new(device, output_format));

            if egui_render_pipeline.output_format != output_format {
                *egui_render_pipeline = EGUIRenderPipeline::new(device, output_format);
            }

            &mut egui_render_pipeline.egui_render_pass
        };

        egui_render_pass
            .add_textures(device, command_queue.queue(), &scene.textures)
            .unwrap();

        egui_render_pass.update_buffers(
            device,
            command_queue.queue(),
            &scene.paint_jobs,
            &scene.screen_descriptor,
        );

        egui_render_pass
            .execute(
                command_queue.command_encoder(device),
                output_texture,
                &scene.paint_jobs,
                &scene.screen_descriptor,
                None,
            )
            .unwrap();

        egui_render_pass.remove_textures(scene.textures).unwrap();
    }
}
