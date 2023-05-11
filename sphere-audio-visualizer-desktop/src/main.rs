#![warn(missing_docs)]

//! This trait implements the platform specific quirks of desktop platforms.
//! If you want to configure the application look at the [`Settings`] struct
//! to asses the different options.

use std::{fs::File, io::BufReader, path::PathBuf, sync::Arc};

use crate::gstreamer_visualizer::{
    EncodingSettings, Resulution, SystemSampleSource, URISampleSource,
};
use serde::{Deserialize, Serialize};
use sphere_audio_visualizer::{
    rendering::{
        wgpu::{Metaballs, Raytracer},
        {MetaballsSceneConverter, RaytracerSceneConverter},
    },
    simulation::{Simulation2D, Simulation3D},
    Application, WGPUVisualizerFactory,
};
use winit::window::WindowBuilder;

pub mod gstreamer_visualizer;

/// Stores the settings of the application
#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    /// Represents the different selectable analysis sample rates
    pub sample_rates: Vec<u64>,
    /// Represemts the default selected sample rate. Should be present in `sample_rates`.
    pub default_sample_rate: usize,
    /// Represents the different selectable frame rates
    pub frame_rates: Vec<u64>,
    /// Represents the default selected frame rate. Should be present in `frame_rates`
    pub default_frame_rate: usize,
    /// Represents the different selectable resulutions
    pub resulutions: Vec<Resulution>,
    /// Represents the default selected resulution. Should be present in `resulutions`
    pub default_resulution: usize,
    /// Represents the different selectable encodings
    pub encodings: Vec<EncodingSettings>,
    /// Represents the index of the default selected encoding. Should be between `0..encodings.len()`
    pub default_encoding: usize,
}

fn executable_dir() -> Option<PathBuf> {
    Some(std::env::current_exe().ok()?.parent()?.to_path_buf())
}

fn settings_dirs() -> Vec<PathBuf> {
    let mut pathes = Vec::new();

    pathes.push(PathBuf::from("."));
    pathes.extend(executable_dir());

    pathes
}

fn load_settings() -> Option<Arc<Settings>> {
    settings_dirs()
        .into_iter()
        .map(|dir| dir.join("settings.yaml"))
        .flat_map(File::open)
        .map(BufReader::new)
        .flat_map(serde_yaml::from_reader)
        .map(Arc::new)
        .next()
}

fn main() {
    gstreamer::init().unwrap();

    let settings: Arc<Settings> = load_settings().expect("Failed to load settings");

    let system_sample_source = SystemSampleSource::new(settings.clone());
    let uri_sample_source = URISampleSource::new(settings);

    let window_builder = WindowBuilder::new();

    Application::new(window_builder)
        .with_sample_source(uri_sample_source, "File")
        .with_online_only_sample_source(system_sample_source, "System")
        .with_visualizer_configuration::<WGPUVisualizerFactory<Simulation3D, RaytracerSceneConverter, Raytracer>, _>("Raytracer")
        .with_visualizer_configuration::<WGPUVisualizerFactory<Simulation2D, MetaballsSceneConverter, Metaballs>, _>("Metaballs")
        .run();
}
