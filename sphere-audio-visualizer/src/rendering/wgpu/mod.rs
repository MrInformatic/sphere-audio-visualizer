//! Contains the WGPU Implementation for the rendering

use std::path::Path;

use thiserror::Error;
use wgpu::{
    Backends, Device, DeviceDescriptor, Instance, PowerPreference, Queue, RequestAdapterOptions,
    RequestDeviceError, TextureFormat, TextureView,
};
use winit::window::Window;

use self::utils::CommandQueue;
pub use self::{pipeline::*, target::*};

mod pipeline;
mod target;
pub mod utils;

const SHADER: &[u8] = include_bytes!(env!("sphere_audio_visualizer_spirv.spv"));

/// Represents the errors which could happen when initializing the WGPU
/// Rendering
#[derive(Debug, Error)]
pub enum WGPURendererInitError {
    /// There was no compatible adapter found
    #[error("no adapter found!")]
    NoAdapterFound,
    /// The device request failed
    #[error("device request failed!")]
    DeviceRequestFailed(#[from] RequestDeviceError),
}

/// Contains all necessary information for rendering with WGPU
pub struct WGPURenderer {
    device: Device,
    queue: Queue,
}

impl WGPURenderer {
    /// Creates a new instance which is onscreen or offscreen depending on if
    /// the window is Some or not.
    /// Optionally a trace path can be specified for debugging purposes.
    pub async fn new(
        window: Option<&Window>,
        trace_path: Option<&Path>,
    ) -> Result<(Self, Option<SurfaceTarget>), WGPURendererInitError> {
        let instance = Instance::new(Backends::all());

        let surface = window.map(|window| unsafe { instance.create_surface(window) });

        let adapter = {
            let request_adapter_options = RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: surface.as_ref(),
                ..Default::default()
            };

            instance
                .request_adapter(&request_adapter_options)
                .await
                .ok_or_else(|| WGPURendererInitError::NoAdapterFound)?
        };

        let device_descriptor = DeviceDescriptor {
            label: Some("sphere-visualizer-device"),
            features: adapter.features(),
            limits: adapter.limits(),
        };

        let (device, queue) = adapter
            .request_device(&device_descriptor, trace_path)
            .await?;

        let target = surface.map(|surface| SurfaceTarget::new(surface, &adapter));

        Ok((Self { device, queue }, target))
    }

    /// Creates a instance for onscreen rendering.
    /// Optionally a trace path can be specified for debugging purposes.
    pub async fn onscreen(
        window: &Window,
        trace_path: Option<&Path>,
    ) -> Result<(Self, SurfaceTarget), WGPURendererInitError> {
        let (this, surface) = Self::new(Some(window), trace_path).await?;

        Ok((this, surface.unwrap()))
    }

    /// Creates a instance for offscreen rendering
    /// Optionally a trace path can be specified for debugging purposes.
    pub async fn offscreen(trace_path: Option<&Path>) -> Result<Self, WGPURendererInitError> {
        Ok(Self::new(None, trace_path).await?.0)
    }

    /// Returns the WGPU [`Device`].
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Returns the WGPU [`Queue`].
    pub fn queue(&self) -> &Queue {
        &self.queue
    }
}

/// A pipeline used for rendering.
pub trait Pipeline<S> {
    /// renders a new frame.
    fn render(
        &mut self,
        scene: S,
        device: &Device,
        command_queue: &mut CommandQueue,
        output_format: TextureFormat,
        output_texture: &TextureView,
    );
}

/// Specifies the different supported shading languages
#[derive(Clone, PartialEq, Eq)]
pub enum ShadingLanguage {
    /// Rust using rust-gpu <https://github.com/EmbarkStudios/rust-gpu>
    Rust,
    /// WGSL <https://gpuweb.github.io/gpuweb/wgsl/>
    WGSL,
}
