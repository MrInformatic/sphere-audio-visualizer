use std::{num::NonZeroU32, sync::Arc};

use wgpu::{
    Buffer, BufferDescriptor, BufferUsages, Device, Extent3d, ImageCopyBuffer, ImageDataLayout,
    Maintain, Texture, TextureAspect, TextureDescriptor, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor, COPY_BYTES_PER_ROW_ALIGNMENT,
};

use crate::rendering::wgpu::utils::CommandQueue;

use super::{RenderTarget, RenderTargetTexture};

struct TextureBufferBundle {
    texture: Texture,
    buffer: Buffer,
}

/// A [`RenderTarget`] used for offscreen rendering
pub struct OffscreenTarget {
    texture_buffer_bundle: Option<Arc<TextureBufferBundle>>,
    texture_descriptor: TextureDescriptor<'static>,
    image_data_layout: ImageDataLayout,
    bytes_per_row: u32,
    format: OutputFormat,
}

impl OffscreenTarget {
    /// Creates a new instance using the specified [`OutputFormat`]
    pub fn new(format: OutputFormat) -> Self {
        let texture_descriptor = TextureDescriptor {
            label: None,
            dimension: wgpu::TextureDimension::D2,
            format: format.into(),
            mip_level_count: 1,
            sample_count: 1,
            size: Extent3d {
                width: 0,
                height: 0,
                depth_or_array_layers: 1,
            },
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        };

        Self {
            texture_buffer_bundle: None,
            texture_descriptor,
            image_data_layout: ImageDataLayout::default(),
            bytes_per_row: 0,
            format,
        }
    }

    /// Returns the [`OutputFormat`] of target texture
    pub fn format(&self) -> OutputFormat {
        self.format
    }
}

impl RenderTarget for OffscreenTarget {
    type Texture = OffscreenTargetTexture;

    fn target_format(&self) -> TextureFormat {
        self.texture_descriptor.format
    }

    fn target_texture(&mut self, width: u32, height: u32, device: &Device) -> Self::Texture {
        if self.texture_buffer_bundle.is_none()
            || self.texture_descriptor.size.width != width
            || self.texture_descriptor.size.height != height
        {
            self.texture_descriptor = TextureDescriptor {
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                ..self.texture_descriptor
            };

            let texture = device.create_texture(&self.texture_descriptor);

            self.bytes_per_row = (width * self.format.size_per_pixel() as u32)
                .div_ceil(COPY_BYTES_PER_ROW_ALIGNMENT)
                * COPY_BYTES_PER_ROW_ALIGNMENT;

            let size = self.bytes_per_row * height;

            let buffer = device.create_buffer(&BufferDescriptor {
                label: None,
                mapped_at_creation: false,
                usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                size: size as u64,
            });

            self.image_data_layout = ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(self.bytes_per_row),
                rows_per_image: NonZeroU32::new(height),
            };

            self.texture_buffer_bundle = Some(Arc::new(TextureBufferBundle { texture, buffer }));
        }

        let texture_buffer_bundle = self.texture_buffer_bundle.clone().unwrap();

        let texture_view = texture_buffer_bundle
            .texture
            .create_view(&TextureViewDescriptor {
                label: None,
                format: None,
                dimension: None,
                aspect: TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

        OffscreenTargetTexture {
            texture_buffer_bundle,
            texture_view,
            image_data_layout: self.image_data_layout.clone(),
            subpixels_per_row: self.bytes_per_row,
            copy_size: self.texture_descriptor.size.clone(),
            format: self.format,
        }
    }
}

/// The [`RenderTargetTexture`] of the [`OffscreenTarget`]
pub struct OffscreenTargetTexture {
    texture_view: TextureView,
    texture_buffer_bundle: Arc<TextureBufferBundle>,
    image_data_layout: ImageDataLayout,
    subpixels_per_row: u32,
    copy_size: Extent3d,
    format: OutputFormat,
}

impl RenderTargetTexture for OffscreenTargetTexture {
    type Output = OffscreenTargetOutput;

    fn texture_view(&self) -> &TextureView {
        &self.texture_view
    }

    fn present(self, device: &Device, queue: &mut CommandQueue) -> Self::Output {
        let command_encoder = queue.command_encoder(device);

        command_encoder.copy_texture_to_buffer(
            self.texture_buffer_bundle.texture.as_image_copy(),
            ImageCopyBuffer {
                buffer: &self.texture_buffer_bundle.buffer,
                layout: self.image_data_layout,
            },
            self.copy_size,
        );

        let image = {
            let slice = self.texture_buffer_bundle.buffer.slice(..);

            let future = slice.map_async(wgpu::MapMode::Read);
            device.poll(Maintain::Wait);
            pollster::block_on(future).unwrap();

            let view = slice.get_mapped_range();

            let size_per_pixel = self.format.size_per_pixel();

            let mut data = Vec::with_capacity(
                self.copy_size.width as usize * self.copy_size.height as usize * size_per_pixel,
            );

            for y in 0..self.copy_size.height {
                let offset = y * self.subpixels_per_row;
                let end = offset + self.copy_size.width * size_per_pixel as u32;
                data.extend(&view[offset as usize..end as usize])
            }

            OffscreenTargetOutput { data }
        };

        self.texture_buffer_bundle.buffer.unmap();

        image
    }
}

/// Specifies the Supported output formats for offscreen rendering
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum OutputFormat {
    /// 8-Bit Red Green Blue Alpha Color
    RGBA8,
}

impl From<OutputFormat> for TextureFormat {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::RGBA8 => TextureFormat::Rgba8UnormSrgb,
        }
    }
}

impl OutputFormat {
    fn size_per_pixel(&self) -> usize {
        match self {
            OutputFormat::RGBA8 => 4,
        }
    }
}

/// Stores the resulting data after offscreen rendering.
pub struct OffscreenTargetOutput {
    /// The raw texture data
    pub data: Vec<u8>,
}
