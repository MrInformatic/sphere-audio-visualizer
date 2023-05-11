use std::ops::{Deref, DerefMut};

use winit::window::Window;

use crate::{
    audio_analysis::Samples,
    module::ModuleManager,
    rendering::wgpu::{EGUIScene, OutputFormat},
    utils::TypeMap,
};

use super::{OfflineVisualizer, OnlineVisualizer, Visualizer, VisualizerFactory};

/// This Visualizer forwards all Visualizer calls to the internal Visualizer.
/// This internal Viusualizer can dynamically swaped at runtime.
/// Also the settings of previous visualizers are store and passed to the
/// creation of new visualizers.
/// Modules are recycled from the previous visualizer.
pub struct DynamicVisualizer {
    settings_bin: TypeMap,
    online_visualizer: Option<Box<dyn OnlineVisualizer>>,
    offline_visualizer_factory:
        Option<fn(OutputFormat, &mut TypeMap) -> Box<dyn OfflineVisualizer>>,
}

impl DynamicVisualizer {
    /// Creates a new Instance
    pub fn new() -> Self {
        Self {
            settings_bin: TypeMap::new(),
            online_visualizer: None,
            offline_visualizer_factory: None,
        }
    }

    /// Get the settings of the previous and current visualizers
    pub fn settings_bin(&self) -> &TypeMap {
        &self.settings_bin
    }

    /// Tries to retrive the current internal visualizer. Fails when the type
    /// does not match.
    pub fn online_visualizer<V: OnlineVisualizer>(&self) -> Option<&V> {
        let online_visualizer = self.online_visualizer.as_ref()?.deref();

        if (online_visualizer as &dyn OnlineVisualizer).type_id() == std::any::TypeId::of::<V>() {
            unsafe { Some(&*(online_visualizer as *const _ as *const V)) }
        } else {
            None
        }
    }

    /// Tries to retrive the current internal visualizer. Fails when the type
    /// does not match.
    pub fn online_visualizer_mut<V: OnlineVisualizer>(&mut self) -> Option<&mut V> {
        let online_visualizer = self.online_visualizer.as_mut()?.deref_mut();

        if (online_visualizer as &dyn OnlineVisualizer).type_id() == std::any::TypeId::of::<V>() {
            unsafe { Some(&mut *(online_visualizer as *mut _ as *mut V)) }
        } else {
            None
        }
    }

    /// Tries to create an offline visualizer matching the settings of the
    /// current inner visualizer.
    pub fn offline_visualizer(
        &mut self,
        format: OutputFormat,
    ) -> Option<Box<dyn OfflineVisualizer>> {
        Some((self.offline_visualizer_factory?)(
            format,
            &mut self.settings_bin,
        ))
    }

    /// Changes the internal Visualizer. Modules from the previous visualizer
    /// are recycled. Also module settings from previous visualizers are
    /// reused.
    pub fn change_visualizer<F: VisualizerFactory>(&mut self, window: &Window) {
        let mut module_manager = ModuleManager::new(&mut self.settings_bin);

        if let Some(visualizer) = self.online_visualizer.take() {
            visualizer.module_bin(&mut module_manager);
        }

        self.online_visualizer = Some(Box::new(F::new_online(window, module_manager)));

        self.offline_visualizer_factory =
            Some(|format, settings_bin| -> Box<dyn OfflineVisualizer> {
                Box::new(F::new_offline(format, ModuleManager::new(settings_bin)))
            });
    }
}

impl Visualizer for DynamicVisualizer {
    fn module_bin(mut self: Box<Self>, module_manager: &mut ModuleManager) {
        if let Some(visualizer) = self.online_visualizer.take() {
            visualizer.module_bin(module_manager);
        }
    }
}

impl OnlineVisualizer for DynamicVisualizer {
    fn visualize(&mut self, samples: Samples, width: u32, height: u32, egui_scene: EGUIScene) {
        if let Some(online_visualizer) = &mut self.online_visualizer {
            online_visualizer.visualize(samples, width, height, egui_scene);
        }
    }
}
