use std::any::Any;

use winit::window::Window;

pub use self::{dynamic_visualizer::*, wgpu::*};
use crate::{
    audio_analysis::Samples,
    module::ModuleManager,
    rendering::wgpu::{
        EGUIScene, {OffscreenTargetOutput, OutputFormat},
    },
};

mod dynamic_visualizer;
mod wgpu;

/// Base trait for the [`OnlineVisualizer`] and [`OfflineVisualizer`]
pub trait Visualizer: Any + Send + Sync {
    /// Deconstructs the visualizer into modules which are stored inside the
    /// module manager.
    fn module_bin(self: Box<Self>, module_manager: &mut ModuleManager);
}

/// An online visualizer is used to draw onto a window. It also support drawing
/// of UI.
pub trait OnlineVisualizer: Visualizer {
    /// Visualizes onto a window. Supports drawing of UI.
    fn visualize(&mut self, samples: Samples, width: u32, height: u32, egui_scene: EGUIScene);
}

/// An offline visualizer is used to draw offscreen.
pub trait OfflineVisualizer: Visualizer {
    /// Visualizes offscreen
    fn visualize(&mut self, samples: Samples, width: u32, height: u32) -> OffscreenTargetOutput;
}

/// A Factory for creating
pub trait VisualizerFactory {
    /// The type of online visualizer created by this factory.
    type OnlineVisualizer: OnlineVisualizer;

    /// The type of offline visualizer created by this factory.
    type OfflineVisualizer: OfflineVisualizer;

    /// Creates a new online visualizer instance.
    /// The `module_manager` is used to recycle modules and retrive stored
    /// settings.
    fn new_online(window: &Window, module_manager: ModuleManager) -> Self::OnlineVisualizer;

    /// Creates a new offline visualizer instance.
    /// The `module_manager` is used to recycle modules and retrive stored
    /// settings.
    fn new_offline(format: OutputFormat, module_manager: ModuleManager) -> Self::OfflineVisualizer;
}
