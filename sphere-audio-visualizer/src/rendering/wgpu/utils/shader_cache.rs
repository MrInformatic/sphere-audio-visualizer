use wgpu::{Device, TextureFormat};

use crate::utils::TypeMap;

/// A ShaderEntry is stored and loaded inside the [`ShaderCache`]
pub trait ShaderEntry: Send + Sync {
    /// Loads the shader of the ShaderEntry using the specified
    /// [`TextureFormat`]
    fn new(device: &Device, target_format: TextureFormat) -> Self;
}

impl ShaderEntry for () {
    fn new(_device: &Device, _target_format: TextureFormat) -> Self {}
}

/// Chaches Shaders
pub struct ShaderCache {
    target_format: TextureFormat,
    cache: TypeMap,
}

impl ShaderCache {
    /// Creates a new Instance. All cached shader will have the specified
    /// [`TextureFormat`]
    pub fn new(target_format: TextureFormat) -> Self {
        Self {
            target_format,
            cache: TypeMap::new(),
        }
    }

    /// Gets a shader from the cache if it is loaded or otherwise loads it.
    pub fn shader<K: ShaderEntry + 'static>(&mut self, device: &Device) -> &K {
        self.cache
            .entry()
            .or_insert_with(|| K::new(device, self.target_format))
    }
}
