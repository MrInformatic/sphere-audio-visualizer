use std::sync::Arc;

use egui::{ComboBox, Grid, Ui};
use gstreamer::prelude::{DeviceMonitorExtManual, ElementExtManual};
use gstreamer::traits::{DeviceExt, ElementExt, GstBinExt};
use gstreamer::{Device, DeviceMonitor, Element, ElementFactory, Pipeline, State};
use sphere_audio_visualizer::{audio_analysis::Samples, OnlineSampleSource};

use crate::Settings;

use super::GStreamerSampleSource;

/// A [`OnlineSampleSource`] based on a GStreamer
/// [`DeviceMonitor`] inputs
pub struct SystemSampleSource {
    settings: Arc<Settings>,
    device_monitor: DeviceMonitor,
    device: Option<Device>,
    sample_rate_id: usize,
    inner: Option<StaticSystemSampleSource>,
}

impl SystemSampleSource {
    /// Creates a new instance
    pub fn new(settings: Arc<Settings>) -> Self {
        let device_monitor = DeviceMonitor::new();

        device_monitor.add_filter(Some("Audio/Source"), None);

        let device = device_monitor.devices().pop_front();

        let sample_rate_id = settings.default_sample_rate;

        Self {
            settings,
            device_monitor,
            device,
            sample_rate_id,
            inner: None,
        }
    }

    fn update(&mut self) {
        self.inner = self.recreate_inner();
    }

    fn recreate_inner(&self) -> Option<StaticSystemSampleSource> {
        let element = self.device.as_ref()?.create_element(None).unwrap();

        Some(StaticSystemSampleSource::new(
            &element,
            self.settings.sample_rates[self.sample_rate_id],
        ))
    }

    fn sample_rate(&self) -> u64 {
        self.settings.sample_rates[self.sample_rate_id]
    }
}

impl OnlineSampleSource for SystemSampleSource {
    fn samples(&mut self) -> Samples {
        if let Some(inner) = &mut self.inner {
            inner.samples()
        } else {
            Samples {
                sample_rate: 44100.0,
                samples: &[],
            }
        }
    }

    fn unfocus(&mut self) {
        self.inner = None;
    }

    fn focus(&mut self) {
        self.update();
    }

    fn ui(&mut self, ui: &mut Ui) {
        Grid::new("System Sample Source Settings")
            .num_columns(2)
            .striped(true)
            .min_col_width(72.0)
            .show(ui, |ui| {
                let device_name = self
                    .device
                    .as_ref()
                    .map(|device| device.display_name().to_string())
                    .unwrap_or("".to_string());

                let old_device = self.device.clone();

                ui.label("Device:");
                ComboBox::from_id_source("System Audio Device")
                    .selected_text(&device_name[..device_name.len().min(22)])
                    .width(168.0)
                    .show_ui(ui, |ui| {
                        for device in self.device_monitor.devices() {
                            let name = device.display_name().to_string();
                            ui.selectable_value(&mut self.device, Some(device), name);
                        }
                    });
                ui.end_row();

                let old_sample_rate = self.sample_rate();

                ui.label("Sample Rate:");
                ComboBox::from_id_source("System Audio Sample Rate")
                    .selected_text(self.sample_rate().to_string())
                    .width(168.0)
                    .show_ui(ui, |ui| {
                        for (id, preset) in self.settings.sample_rates.iter().enumerate() {
                            ui.selectable_value(
                                &mut self.sample_rate_id,
                                id,
                                format!("{} hz", preset),
                            );
                        }
                    });
                ui.end_row();

                if old_device != self.device || old_sample_rate != self.sample_rate() {
                    self.update()
                }
            });
    }
}

struct StaticSystemSampleSource {
    pipeline: Pipeline,
    sample_source: GStreamerSampleSource,
    amplification: f32,
}

impl StaticSystemSampleSource {
    pub fn new(src: &Element, max_sample_rate: u64) -> Self {
        let pipeline = Pipeline::new(None);

        let audio_resample = ElementFactory::make("audioresample").build().unwrap();

        let audio_convert = ElementFactory::make("audioconvert").build().unwrap();

        let sample_source = GStreamerSampleSource::new(Some(max_sample_rate));

        pipeline.add(src).unwrap();
        pipeline.add(&audio_resample).unwrap();
        pipeline.add(&audio_convert).unwrap();
        pipeline.add(&sample_source.app_sink).unwrap();

        src.link(&audio_resample).unwrap();
        audio_resample.link(&audio_convert).unwrap();
        audio_convert.link(&sample_source.app_sink).unwrap();

        pipeline.set_state(State::Playing).unwrap();

        Self {
            pipeline,
            sample_source,
            amplification: 256.0,
        }
    }
}

impl OnlineSampleSource for StaticSystemSampleSource {
    fn samples(&mut self) -> Samples {
        let samples = self.sample_source.samples();

        self.amplification *= f64::powf(
            2.0,
            samples.samples.len() as f64 / samples.sample_rate * 2.0,
        ) as f32;
        self.amplification = f32::min(self.amplification, 256.0);

        if let Some(amplification) = samples
            .samples
            .iter()
            .cloned()
            .map(f32::abs)
            .reduce(f32::max)
            .map(f32::recip)
        {
            self.amplification = self.amplification.min(amplification)
        }

        for sample in samples.samples.iter_mut() {
            *sample *= self.amplification;
        }

        samples.into()
    }

    fn unfocus(&mut self) {}

    fn focus(&mut self) {}

    fn ui(&mut self, _ui: &mut Ui) {}
}

impl Drop for StaticSystemSampleSource {
    fn drop(&mut self) {
        self.pipeline.set_state(State::Null).unwrap();
    }
}
