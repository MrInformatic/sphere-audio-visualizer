use wgpu::{Device, TextureFormat, TextureView};

pub use self::{offscreen::*, surface::*};
use super::utils::CommandQueue;

mod offscreen;
mod surface;

/// Abstracts a render target
pub trait RenderTarget: Send + Sync {
    /// The type of texture used by the render target
    type Texture: RenderTargetTexture;

    /// The [`TextureFormat`] of the target texture
    fn target_format(&self) -> TextureFormat;

    /// Retrives one texture from the render target
    fn target_texture<'a>(&mut self, width: u32, height: u32, device: &Device) -> Self::Texture;
}

/// Abstracts a render target texture
pub trait RenderTargetTexture {
    /// The output of the texture after presenting.
    type Output;

    /// Gets the WGPU [`TextureView`] used for rendering.
    fn texture_view(&self) -> &TextureView;

    /// Presents the texture.
    fn present(self, device: &Device, queue: &mut CommandQueue) -> Self::Output;
}
