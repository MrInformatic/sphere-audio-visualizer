use sphere_audio_visualizer_core::raytracing::{
    light::PointLight,
    shape::{Rect, SceneArgs, Sphere, AABB},
    BasicRaytracingArgsBundle, RaytracerArgs,
};
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
        scene::{BasicRaytracerScene, ShapeCollection},
        wgpu::{
            utils::{
                CommandQueue, {TypedBufferDeviceExt, TypedBufferInitDescriptor},
            },
            Pipeline, ShadingLanguage, SHADER,
        },
    },
};

struct RaytracerWGSLPipeline(RenderPipeline, TextureFormat);

impl RaytracerWGSLPipeline {
    fn new(device: &Device, target_format: TextureFormat) -> Self {
        let shader_module = device.create_shader_module(&include_wgsl!("raytracing.wgsl"));

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("sphere-visualizer-raytracing-pipeline"),
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

struct RaytracerRustPipeline(RenderPipeline, TextureFormat);

impl RaytracerRustPipeline {
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
                BindGroupLayoutEntry {
                    binding: 2,
                    count: None,
                    ty: BindingType::Buffer {
                        has_dynamic_offset: false,
                        min_binding_size: None,
                        ty: BufferBindingType::Storage { read_only: true },
                    },
                    visibility: ShaderStages::FRAGMENT,
                },
                BindGroupLayoutEntry {
                    binding: 3,
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
                entry_point: "raytracing_vs",
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "raytracing_fs",
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

/// The pipeline module used for raytraced rendering
pub struct Raytracer {
    implementation: ShadingLanguage,
    rust_pipeline: Option<RaytracerRustPipeline>,
    wgsl_pipeline: Option<RaytracerWGSLPipeline>,
}

impl Raytracer {
    /// Creates a new instance using the specified [`ShadingLanguage`]
    pub fn from_implementation(implementation: ShadingLanguage) -> Self {
        Self {
            implementation,
            rust_pipeline: None,
            wgsl_pipeline: None,
        }
    }

    /// Sets [`ShadingLanguage`] that should be used going forward
    pub fn with_implementation(mut self, implementation: ShadingLanguage) -> Self {
        self.set_implementation(implementation);
        self
    }

    /// Sets [`ShadingLanguage`] that should be used going forward
    pub fn set_implementation(&mut self, implementation: ShadingLanguage) -> &mut Self {
        self.implementation = implementation;
        self
    }

    /// Gets the used [`ShadingLanguage`]
    pub fn implementation(&self) -> ShadingLanguage {
        self.implementation.clone()
    }
}

/// Stores the settings of the [`Raytracer`] pipeline module
#[derive(Clone)]
pub struct RaytracerSettings {
    /// The used [`ShadingLanguage`]
    pub shading_language: ShadingLanguage,
}

impl Default for RaytracerSettings {
    fn default() -> Self {
        Self {
            shading_language: ShadingLanguage::Rust,
        }
    }
}

impl Module for Raytracer {
    type Settings = RaytracerSettings;

    fn set_settings(&mut self, settings: Self::Settings) -> &mut Self {
        self.set_implementation(settings.shading_language)
    }

    fn settings(&self) -> Self::Settings {
        RaytracerSettings {
            shading_language: self.implementation(),
        }
    }
}

impl Default for Raytracer {
    fn default() -> Self {
        Self {
            implementation: ShadingLanguage::Rust,
            rust_pipeline: None,
            wgsl_pipeline: None,
        }
    }
}

impl Pipeline<BasicRaytracerScene> for Raytracer {
    fn render(
        &mut self,
        mut scene: BasicRaytracerScene,
        device: &Device,
        command_queue: &mut CommandQueue,
        output_format: TextureFormat,
        target_texture: &TextureView,
    ) {
        let pipeline = match self.implementation {
            ShadingLanguage::Rust => {
                let rust_pipeline = self
                    .rust_pipeline
                    .get_or_insert_with(|| RaytracerRustPipeline::new(device, output_format));

                if rust_pipeline.1 != output_format {
                    *rust_pipeline = RaytracerRustPipeline::new(device, output_format);
                }

                &rust_pipeline.0
            }
            ShadingLanguage::WGSL => {
                let wgsl_pipeline = self
                    .wgsl_pipeline
                    .get_or_insert_with(|| RaytracerWGSLPipeline::new(device, output_format));

                if wgsl_pipeline.1 != output_format {
                    *wgsl_pipeline = RaytracerWGSLPipeline::new(device, output_format);
                }

                &wgsl_pipeline.0
            }
        };

        let spheres = scene.shapes::<Sphere>();
        let spheres_bounding_box = spheres
            .map(ShapeCollection::bounding_box)
            .cloned()
            .unwrap_or_else(AABB::empty);

        let spheres_buffer = device.create_typed_buffer_init(&TypedBufferInitDescriptor {
            label: None,
            usage: BufferUsages::STORAGE,
            value: spheres.map(ShapeCollection::shapes).unwrap_or(&[]),
        });

        let rects = scene.shapes::<Rect>();
        let rects_bounding_box = rects
            .map(ShapeCollection::bounding_box)
            .cloned()
            .unwrap_or_else(AABB::empty);

        let rects_buffer = device.create_typed_buffer_init(&TypedBufferInitDescriptor {
            label: None,
            usage: BufferUsages::STORAGE,
            value: rects.map(ShapeCollection::shapes).unwrap_or(&[]),
        });

        let point_lights_buffer = device.create_typed_buffer_init(&TypedBufferInitDescriptor {
            label: None,
            usage: BufferUsages::STORAGE,
            value: scene
                .lights_mut::<PointLight>()
                .map(Vec::as_slice)
                .unwrap_or(&[]),
        });

        let args = BasicRaytracingArgsBundle {
            raytracer_args: RaytracerArgs {
                camera: scene.camera,
                background: scene.background,
                bounces: scene.bounces,
            },
            scene_args: SceneArgs {
                spheres_bounding_box,
                rects_bounding_box,
            },
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
                spheres_buffer.bind_group_entry(1).unwrap(),
                rects_buffer.bind_group_entry(2).unwrap(),
                point_lights_buffer.bind_group_entry(3).unwrap(),
            ],
            layout: &layout,
        });

        let command_encoder = command_queue.command_encoder(device);

        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachment {
                    view: target_texture,
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
