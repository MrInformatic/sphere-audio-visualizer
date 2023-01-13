use wgpu::{
    Adapter, Device, PresentMode, Surface, SurfaceConfiguration, SurfaceTexture, TextureAspect,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};

use crate::rendering::wgpu::utils::CommandQueue;

use super::{RenderTarget, RenderTargetTexture};

/// A [`RenderTarget`] used for rendering on a surface
pub struct SurfaceTarget {
    surface: Surface,
    surface_configuration: SurfaceConfiguration,
}

impl SurfaceTarget {
    /// Creates a new instance
    pub fn new(surface: Surface, adapter: &Adapter) -> Self {
        let surface_configuration = SurfaceConfiguration {
            format: surface
                .get_preferred_format(adapter)
                .unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb),
            width: 0,
            height: 0,
            present_mode: PresentMode::Mailbox,
            usage: TextureUsages::RENDER_ATTACHMENT,
        };

        SurfaceTarget {
            surface: surface,
            surface_configuration,
        }
    }
}

impl RenderTarget for SurfaceTarget {
    type Texture = SurfaceTargetTexture;

    fn target_format(&self) -> TextureFormat {
        self.surface_configuration.format
    }

    fn target_texture(&mut self, width: u32, height: u32, device: &Device) -> Self::Texture {
        if self.surface_configuration.width != width || self.surface_configuration.height != height
        {
            self.surface_configuration = SurfaceConfiguration {
                width,
                height,
                ..self.surface_configuration
            };

            self.surface.configure(device, &self.surface_configuration);
        }

        let texture = self.surface.get_current_texture().unwrap();
        let texture_view = texture.texture.create_view(&TextureViewDescriptor {
            label: None,
            format: None,
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        SurfaceTargetTexture {
            texture,
            texture_view,
        }
    }
}

/// The [`RenderTargetTexture`] of the [`SurfaceTarget`]
pub struct SurfaceTargetTexture {
    texture: SurfaceTexture,
    texture_view: TextureView,
}

impl RenderTargetTexture for SurfaceTargetTexture {
    type Output = ();

    fn texture_view(&self) -> &TextureView {
        &self.texture_view
    }

    fn present(self, _device: &Device, queue: &mut CommandQueue) -> Self::Output {
        queue.submit();

        self.texture.present()
    }
}
