use sphere_visualizer_core::metaballs::MetaballsArgs;
use wgpu::{
    include_wgsl, util::make_spirv_raw, BindGroupDescriptor, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages, Color, ColorTargetState,
    ColorWrites, Device, FragmentState, LoadOp, Operations, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptorSpirV, ShaderStages,
    TextureFormat, TextureView, VertexState,
};

use crate::{
    module::Module,
    rendering::{
        scene::MetaballsScene,
        wgpu::{
            utils::{
                CommandQueue, {TypedBufferDeviceExt, TypedBufferInitDescriptor},
            },
            Pipeline, ShadingLanguage, SHADER,
        },
    },
};

struct MetaballsWGSLPipeline(RenderPipeline, TextureFormat);

impl MetaballsWGSLPipeline {
    fn new(device: &Device, target_format: TextureFormat) -> Self {
        let shader_module = device.create_shader_module(&include_wgsl!("metaballs.wgsl"));

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("sphere-visualizer-metaballs-pipeline"),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vertex",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "fragment",
                targets: &[ColorTargetState {
                    format: target_format,
                    blend: None,
                    write_mask: ColorWrites::COLOR,
                }],
            }),
            depth_stencil: None,
            multiview: None,
            layout: None,
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                polygon_mode: PolygonMode::Fill,
                ..Default::default()
            },
            multisample: Default::default(),
        });

        Self(pipeline, target_format)
    }
}

struct MetaballsRustPipeline(RenderPipeline, TextureFormat);

impl MetaballsRustPipeline {
    fn new(device: &Device, target_format: TextureFormat) -> Self {
        let shader_module = unsafe {
            device.create_shader_module_spirv(&ShaderModuleDescriptorSpirV {
                label: None,
                source: make_spirv_raw(SHADER),
            })
        };

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    ty: BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: BufferBindingType::Storage { read_only: true },
                    },
                    visibility: ShaderStages::FRAGMENT,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    ty: BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: BufferBindingType::Storage { read_only: true },
                    },
                    visibility: ShaderStages::FRAGMENT,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            vertex: VertexState {
                module: &shader_module,
                entry_point: "metaballs_vs",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "metaballs_fs",
                targets: &[ColorTargetState {
                    format: target_format,
                    blend: None,
                    write_mask: ColorWrites::COLOR,
                }],
            }),
            depth_stencil: None,
            multiview: None,
            layout: Some(&pipeline_layout),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                polygon_mode: PolygonMode::Fill,
                ..Default::default()
            },
            multisample: Default::default(),
        });

        Self(pipeline, target_format)
    }
}

/// The pipeline module for rendering metaballs scenes
pub struct Metaballs {
    implementation: ShadingLanguage,
    rust_pipeline: Option<MetaballsRustPipeline>,
    wgsl_pipeline: Option<MetaballsWGSLPipeline>,
}

impl Metaballs {
    /// Creates a new instance using the specified Shading Language
    pub fn from_implementation(implementation: ShadingLanguage) -> Self {
        Self {
            implementation,
            rust_pipeline: None,
            wgsl_pipeline: None,
        }
    }

    /// Sets the Shading Language that should be used going forward.
    pub fn with_implementation(mut self, implementation: ShadingLanguage) -> Self {
        self.set_implementation(implementation);
        self
    }

    /// Sets the Shading Language that should be used going forward.
    pub fn set_implementation(&mut self, implementation: ShadingLanguage) -> &mut Self {
        self.implementation = implementation;
        self
    }

    /// Gets the currently used Shading Language.
    pub fn implementation(&self) -> ShadingLanguage {
        self.implementation.clone()
    }
}

/// Stores the settings of the [`Metaballs`] pipeline module
#[derive(Clone)]
pub struct MetaballsSettings {
    /// The used [`ShadingLanguage`]
    pub shading_language: ShadingLanguage,
}

impl Default for MetaballsSettings {
    fn default() -> Self {
        Self {
            shading_language: ShadingLanguage::Rust,
        }
    }
}

impl Module for Metaballs {
    type Settings = MetaballsSettings;

    fn set_settings(&mut self, settings: Self::Settings) -> &mut Self {
        self.set_implementation(settings.shading_language)
    }

    fn settings(&self) -> Self::Settings {
        MetaballsSettings {
            shading_language: self.implementation(),
        }
    }
}

impl Default for Metaballs {
    fn default() -> Self {
        Self {
            implementation: ShadingLanguage::WGSL,
            rust_pipeline: None,
            wgsl_pipeline: None,
        }
    }
}

impl Pipeline<MetaballsScene> for Metaballs {
    fn render(
        &mut self,
        scene: MetaballsScene,
        device: &Device,
        command_queue: &mut CommandQueue,
        output_format: TextureFormat,
        output_texture: &TextureView,
    ) {
        let pipeline = match self.implementation {
            ShadingLanguage::Rust => {
                let rust_pipeline = self
                    .rust_pipeline
                    .get_or_insert_with(|| MetaballsRustPipeline::new(device, output_format));

                if rust_pipeline.1 != output_format {
                    *rust_pipeline = MetaballsRustPipeline::new(device, output_format);
                }

                &rust_pipeline.0
            }
            ShadingLanguage::WGSL => {
                let wgsl_pipeline = self
                    .wgsl_pipeline
                    .get_or_insert_with(|| MetaballsWGSLPipeline::new(device, output_format));

                if wgsl_pipeline.1 != output_format {
                    *wgsl_pipeline = MetaballsWGSLPipeline::new(device, output_format);
                }

                &wgsl_pipeline.0
            }
        };

        let metaballs_buffer = device.create_typed_buffer_init(&TypedBufferInitDescriptor {
            label: None,
            usage: BufferUsages::STORAGE,
            value: scene.metaballs.as_slice(),
        });

        let args = MetaballsArgs {
            color: scene.color,
            size: scene.size,
            zoom: scene.zoom,
        };

        let args_buffer = device.create_typed_buffer_init(&TypedBufferInitDescriptor {
            label: None,
            usage: BufferUsages::STORAGE,
            value: &args,
        });

        let layout = pipeline.get_bind_group_layout(0);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            entries: &[
                args_buffer.bind_group_entry(0).unwrap(),
                metaballs_buffer.bind_group_entry(1).unwrap(),
            ],
            layout: &layout,
        });

        let command_encoder = command_queue.command_encoder(device);

        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachment {
                    view: output_texture,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);

            render_pass.draw(0..4, 0..1);
        }
    }
}
