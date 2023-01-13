//! Contains the implementation to harness the power of GStreamer for the
//! Sphere Audio Visualizer.

use std::sync::{Arc, Mutex};

pub use self::{system::*, uri::*, visualizer::*};
use gstreamer::{
    glib::clone::Downgrade, prelude::ElementExtManual, traits::PadExt, FlowSuccess, Sample,
};
use gstreamer_app::{AppSink, AppSinkCallbacks};
use gstreamer_audio::{AudioCapsBuilder, AUDIO_FORMAT_F32};
use serde::{Deserialize, Serialize};
use sphere_visualizer::audio_analysis::Samples;

mod system;
mod uri;
mod visualizer;

/// Stores resulution settings
#[derive(Serialize, Deserialize, Clone)]
pub struct Resulution {
    /// Represents the width in pixels
    pub width: u32,
    /// Represents the height in pixels
    pub height: u32,
}

/// Stores encoding settings
#[derive(Serialize, Deserialize, Clone)]
pub struct EncodingSettings {
    /// Represents the name that is shown in the UI
    pub name: String,
    /// Represents the GStreamer Caps of the container
    pub container_caps: String,
    /// Represents the GStreamer Caps of the audio stream
    pub audio_caps: String,
    /// Represents the GStreamer Caps of the video stream
    pub video_caps: String,
    /// Represents the extension of the file
    pub extension: String,
}

/// Stores multible samples but content is mutable
pub struct SamplesMut<'a> {
    /// Represents the sample rate of the samples
    pub sample_rate: f64,
    /// Represents the samples
    pub samples: &'a mut [f32],
}

impl<'a> From<SamplesMut<'a>> for Samples<'a> {
    fn from(value: SamplesMut<'a>) -> Self {
        Self {
            sample_rate: value.sample_rate,
            samples: value.samples,
        }
    }
}

/// A wrapper for the AppSink to extract sample on demand rather than callback
pub struct GStreamerSampleSource {
    app_sink: AppSink,
    samples: Vec<f32>,
    sample_buffer: Arc<Mutex<Vec<f32>>>,
}

impl GStreamerSampleSource {
    /// Creates a new instance
    /// - `max_sample_rate` Represents the maximum sample rate that should be accepted by the AppSink
    pub fn new(max_sample_rate: Option<u64>) -> Self {
        let mut sink_caps_builder = AudioCapsBuilder::new()
            .format(AUDIO_FORMAT_F32)
            .channels(1i32);

        if let Some(max_sample_rate) = max_sample_rate {
            sink_caps_builder = sink_caps_builder.rate_range(1..max_sample_rate as i32);
        }

        let sink_caps = sink_caps_builder.build();

        let app_sink = AppSink::builder()
            .caps(&sink_caps)
            .max_buffers(8)
            .drop(true)
            .build();

        let sample_buffer = Arc::new(Mutex::new(Vec::<f32>::new()));

        {
            let sample_buffer = sample_buffer.downgrade();

            app_sink.set_callbacks(
                AppSinkCallbacks::builder()
                    .new_sample(move |app_sink| {
                        if let Some(sample_buffer) = sample_buffer.upgrade() {
                            Self::extend_samples(
                                &mut sample_buffer.lock().unwrap(),
                                app_sink.pull_sample().unwrap(),
                            );
                        }

                        Ok(FlowSuccess::Ok)
                    })
                    .build(),
            );
        }

        Self {
            app_sink,
            sample_buffer,
            samples: vec![],
        }
    }

    fn extend_samples(sample_buffer: &mut Vec<f32>, gst_sample: Sample) {
        let gst_buffer = gst_sample.buffer().unwrap();

        let gst_mapped_buffer = gst_buffer.map_readable().unwrap();

        let slice = gst_mapped_buffer.as_slice();
        let samples = slice.len() * std::mem::size_of::<u8>() / std::mem::size_of::<f32>();
        let ptr = slice.as_ptr() as *const f32;
        let silce = unsafe { &*std::ptr::slice_from_raw_parts(ptr, samples) };

        sample_buffer.extend(silce);
    }

    /// Gets the collected sample also clears the internal buffer.
    pub fn samples(&mut self) -> SamplesMut {
        self.samples.clear();

        std::mem::swap(&mut self.samples, &mut self.sample_buffer.lock().unwrap());

        SamplesMut {
            sample_rate: self.sample_rate().unwrap_or(44100.0),
            samples: &mut self.samples,
        }
    }

    fn sample_rate(&self) -> Option<f64> {
        Some(
            self.app_sink
                .sink_pads()
                .get(0)?
                .caps()?
                .structure(0)?
                .get::<i32>("rate")
                .ok()? as f64,
        )
    }
}
