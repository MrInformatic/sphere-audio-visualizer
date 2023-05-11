use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use egui::{Button, ComboBox, Grid, ProgressBar, Ui};
use gstreamer::{
    prelude::{ElementExtManual, ObjectExt},
    traits::{ElementExt, GstBinExt, PadExt},
    Bus, Caps, ClockTime, ElementFactory, Fraction, MessageType, MessageView, Pipeline, SeekFlags,
    State,
};
use gstreamer_pbutils::{
    encoding_profile::EncodingProfileBuilder, EncodingAudioProfile, EncodingContainerProfile,
    EncodingVideoProfile,
};
use gstreamer_video::VideoCapsBuilder;
use rfd::FileDialog;
use sphere_audio_visualizer::{
    audio_analysis::Samples,
    rendering::wgpu::OutputFormat,
    OfflineVisualizer, {ExportProcess, Exporter, OnlineSampleSource},
};

use crate::Settings;

use super::{visualizer::VisualizerElement, EncodingSettings, GStreamerSampleSource, Resulution};

const PLAY: &'static str = "▶";
const PAUSE: &'static str = "⏸";
const SKIP_FORWARD: &'static str = "⏩";
const SKIP_BACKWARD: &'static str = "⏪";

/// A [`OnlineSampleSource`] and [`Exporter`] based on a GStreamer
/// `uridecodebin`
pub struct URISampleSource {
    settings: Arc<Settings>,
    file_path: Option<PathBuf>,
    sample_rate_id: usize,
    frame_rate_id: usize,
    resulution_id: usize,
    encoding_id: usize,
    inner: Option<StaticURISampleSource>,
}

impl URISampleSource {
    /// Creates a new instance.
    pub fn new(settings: Arc<Settings>) -> Self {
        let sample_rate_id = settings.default_sample_rate;
        let frame_rate_id = settings.default_frame_rate;
        let resulution_id = settings.default_resulution;
        let encoding_id = settings.default_encoding;

        let mut this = Self {
            settings,
            file_path: None,
            sample_rate_id,
            frame_rate_id,
            resulution_id,
            encoding_id,
            inner: None,
        };

        this.update();

        this
    }

    fn update(&mut self) {
        self.inner = self.recreate_inner();
    }

    fn recreate_inner(&self) -> Option<StaticURISampleSource> {
        Some(StaticURISampleSource::new(
            self.settings.sample_rates[self.sample_rate_id],
            self.file_path.as_ref()?,
        ))
    }

    fn sample_rate(&self) -> u64 {
        self.settings.sample_rates[self.sample_rate_id]
    }

    fn frame_rate(&self) -> u64 {
        self.settings.frame_rates[self.frame_rate_id]
    }

    fn resulution(&self) -> &Resulution {
        &self.settings.resulutions[self.resulution_id]
    }

    fn encoding(&self) -> &EncodingSettings {
        &self.settings.encodings[self.encoding_id]
    }
}

impl OnlineSampleSource for URISampleSource {
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
        if let Some(inner) = &mut self.inner {
            inner.unfocus()
        }
    }

    fn focus(&mut self) {
        if let Some(inner) = &mut self.inner {
            inner.focus()
        }
    }

    fn ui(&mut self, ui: &mut Ui) {
        let mut changed = false;

        if ui.add_sized([256.0, 20.0], Button::new("Open")).clicked() {
            if let Some(file_path) = FileDialog::new().pick_file() {
                self.file_path = Some(file_path);
                changed = true;
            }
        }

        if let Some(inner) = &mut self.inner {
            if inner.eof() {
                changed = true;
            }
        }

        let old_sample_rate = self.sample_rate();

        Grid::new("Audio Sample Rate Grid")
            .num_columns(2)
            .min_col_width(72.0)
            .show(ui, |ui| {
                ui.label("Sample Rate:");

                ComboBox::from_id_source("URI Audio Sample Rate")
                    .selected_text(format!("{} hz", old_sample_rate))
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
            });

        let position = self
            .inner
            .as_ref()
            .and_then(StaticURISampleSource::position)
            .map(ClockTime::nseconds)
            .unwrap_or(0);

        let duration = self
            .inner
            .as_ref()
            .and_then(StaticURISampleSource::duration)
            .map(ClockTime::nseconds)
            .unwrap_or(1);

        ui.add_enabled_ui(self.inner.is_some(), |ui| {
            if ui
                .add(ProgressBar::new(position as f32 / duration as f32).desired_width(256.0))
                .changed()
            {
                if let Some(inner) = &self.inner {
                    inner.seek(ClockTime::from_nseconds(position))
                }
            }
            ui.horizontal(|ui| {
                if ui
                    .add_sized([80.0, 20.0], Button::new(SKIP_BACKWARD))
                    .clicked()
                {
                    if let Some(inner) = &self.inner {
                        if let Some(position) = inner.position() {
                            inner.seek(position.saturating_sub(ClockTime::from_seconds(5)))
                        }
                    }
                }

                let is_playing = self
                    .inner
                    .as_ref()
                    .map(StaticURISampleSource::is_playing)
                    .unwrap_or(false);

                let play_text = if is_playing { PAUSE } else { PLAY };

                if ui.add_sized([80.0, 20.0], Button::new(play_text)).clicked() {
                    if let Some(inner) = &mut self.inner {
                        inner.set_playing(!is_playing)
                    }
                }

                if ui
                    .add_sized([80.0, 20.0], Button::new(SKIP_FORWARD))
                    .clicked()
                {
                    if let Some(inner) = &self.inner {
                        if let Some(position) = inner.position() {
                            inner.seek(position.saturating_add(ClockTime::from_seconds(5)))
                        }
                    }
                }
            });
        });

        if changed || old_sample_rate != self.sample_rate() {
            self.update()
        }
    }
}

impl Exporter for URISampleSource {
    fn format(&self) -> OutputFormat {
        OutputFormat::RGBA8
    }

    fn can_export(&self) -> bool {
        self.file_path.is_some()
    }

    fn export(&mut self, visualizer: Box<dyn OfflineVisualizer>) -> Option<Box<dyn ExportProcess>> {
        let open_path = self.file_path.as_ref()?;
        let encoding = self.encoding();

        let save_path = FileDialog::new()
            .add_filter(&encoding.extension, &[&encoding.extension])
            .save_file()?;

        let resulution = self.resulution();
        let frame_rate = self.frame_rate();

        let export = URIExport::new(
            visualizer, resulution, frame_rate, encoding, open_path, save_path,
        );

        Some(Box::new(export))
    }

    fn ui(&mut self, ui: &mut Ui) {
        Grid::new("URI Export Settings Table")
            .num_columns(2)
            .striped(true)
            .min_col_width(72.0)
            .show(ui, |ui| {
                ui.label("Resulution:");
                let resulution = self.resulution();
                ComboBox::from_id_source("URI Video Resulution")
                    .selected_text(format!("{}x{}", resulution.width, resulution.height))
                    .width(168.0)
                    .show_ui(ui, |ui| {
                        for (id, preset) in self.settings.resulutions.iter().enumerate() {
                            ui.selectable_value(
                                &mut self.resulution_id,
                                id,
                                format!("{}x{}", preset.width, preset.height),
                            );
                        }
                    });
                ui.end_row();

                ui.label("Frame Rate:");
                ComboBox::from_id_source("URI Video Frame Rate")
                    .selected_text(format!("{} hz", self.frame_rate()))
                    .width(168.0)
                    .show_ui(ui, |ui| {
                        for (id, preset) in self.settings.frame_rates.iter().enumerate() {
                            ui.selectable_value(
                                &mut self.frame_rate_id,
                                id,
                                format!("{} hz", preset),
                            );
                        }
                    });
                ui.end_row();

                ui.label("Encoding:");
                ComboBox::from_id_source("URI Video Encoding")
                    .selected_text(&self.encoding().name)
                    .width(168.0)
                    .show_ui(ui, |ui| {
                        for (id, preset) in self.settings.encodings.iter().enumerate() {
                            ui.selectable_value(&mut self.encoding_id, id, &preset.name);
                        }
                    });
                ui.end_row();
            });
    }
}

/// The inner implementation of the [URISampleSource]
pub struct StaticURISampleSource {
    pipeline: Pipeline,
    bus: Bus,
    sample_source: GStreamerSampleSource,
    is_playing: bool,
    eof: bool,
}

impl StaticURISampleSource {
    /// Creates a new instance
    pub fn new(max_sample_rate: u64, path: impl AsRef<Path>) -> Self {
        let pipeline = Pipeline::new(None);

        let uri_decode_bin = ElementFactory::make("uridecodebin")
            .property("uri", format!("file://{}", path.as_ref().display()))
            .property("caps", Caps::builder("audio/x-raw").build())
            .build()
            .unwrap();

        let tee = ElementFactory::make("tee").build().unwrap();
        let queue = ElementFactory::make("queue").build().unwrap();

        let app_audio_resample = ElementFactory::make("audioresample").build().unwrap();
        let app_audio_convert = ElementFactory::make("audioconvert").build().unwrap();
        let sample_source = GStreamerSampleSource::new(Some(max_sample_rate));

        let audio_resample = ElementFactory::make("audioresample").build().unwrap();
        let audio_convert = ElementFactory::make("audioconvert").build().unwrap();
        let autoaudiosink = ElementFactory::make("autoaudiosink").build().unwrap();

        let app_sink = sample_source.app_sink.clone();

        pipeline.add(&uri_decode_bin).unwrap();

        pipeline.add(&tee).unwrap();
        pipeline.add(&queue).unwrap();
        pipeline.add(&app_audio_resample).unwrap();
        pipeline.add(&app_audio_convert).unwrap();
        pipeline.add(&app_sink).unwrap();
        pipeline.add(&audio_resample).unwrap();
        pipeline.add(&audio_convert).unwrap();
        pipeline.add(&autoaudiosink).unwrap();

        uri_decode_bin.connect_pad_added(move |uri_decode_bin, _src_pad| {
            tee.sync_state_with_parent().unwrap();
            queue.sync_state_with_parent().unwrap();
            audio_resample.sync_state_with_parent().unwrap();
            audio_convert.sync_state_with_parent().unwrap();
            app_sink.sync_state_with_parent().unwrap();
            audio_resample.sync_state_with_parent().unwrap();
            audio_convert.sync_state_with_parent().unwrap();
            autoaudiosink.sync_state_with_parent().unwrap();

            uri_decode_bin.link(&tee).unwrap();
            tee.link(&queue).unwrap();
            queue.link(&app_audio_resample).unwrap();
            app_audio_resample.link(&app_audio_convert).unwrap();
            app_audio_convert.link(&app_sink).unwrap();
            tee.link(&audio_resample).unwrap();
            audio_resample.link(&audio_convert).unwrap();
            audio_convert.link(&autoaudiosink).unwrap();
        });

        pipeline.set_state(State::Playing).unwrap();

        let bus = pipeline.bus().unwrap();

        Self {
            pipeline,
            bus,
            sample_source,
            is_playing: true,
            eof: false,
        }
    }

    /// Returns if the source is currently playing
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Sets the playing state of the source
    pub fn set_playing(&mut self, is_playing: bool) {
        if is_playing {
            self.play()
        } else {
            self.pause()
        }
    }

    /// Sets the playing state of the source to playing
    pub fn play(&mut self) {
        self.is_playing = true;
        self.pipeline.set_state(State::Playing).unwrap();
    }

    /// Sets the playing state of the source to paused
    pub fn pause(&mut self) {
        self.is_playing = false;
        self.pipeline.set_state(State::Paused).unwrap();
    }

    /// Returns the duration of the playing track
    pub fn duration(&self) -> Option<ClockTime> {
        self.pipeline.query_duration()
    }

    /// Returns the position of the playing track
    pub fn position(&self) -> Option<ClockTime> {
        self.pipeline.query_position()
    }

    /// Seeks to the given position
    pub fn seek(&self, mut position: ClockTime) {
        if position < ClockTime::ZERO {
            position = ClockTime::ZERO;
        }

        if let Some(duration) = self.duration() {
            if position >= duration {
                position = duration;
            }
        }

        self.pipeline
            .seek_simple(SeekFlags::FLUSH | SeekFlags::ACCURATE, position)
            .unwrap();
    }

    /// Returns true if the the pipline has reached the end of the file
    pub fn eof(&mut self) -> bool {
        if self.eof {
            return true;
        }

        while let Some(_) = self.bus.pop_filtered(&[MessageType::Eos]) {
            self.eof = true;
            return true;
        }

        return false;
    }
}

impl OnlineSampleSource for StaticURISampleSource {
    fn samples(&mut self) -> Samples {
        self.sample_source.samples().into()
    }

    fn unfocus(&mut self) {
        self.pause();
    }

    fn focus(&mut self) {}

    fn ui(&mut self, _ui: &mut Ui) {}
}

impl Drop for StaticURISampleSource {
    fn drop(&mut self) {
        self.pipeline.set_state(State::Null).unwrap();
    }
}

/// An [`ExportProcess`] for GStreamer `uridecodebin`
pub struct URIExport {
    pipeline: Pipeline,
    bus: Bus,
    name: String,
    finished: bool,
}

impl URIExport {
    /// Creates a new instance
    pub fn new(
        visualizer: Box<dyn OfflineVisualizer>,
        resulution: &Resulution,
        frame_rate: u64,
        encoding: &EncodingSettings,
        open_path: impl AsRef<Path>,
        save_path: impl AsRef<Path>,
    ) -> Self {
        let open_path = open_path.as_ref();
        let save_path = save_path.as_ref();

        let pipeline = Pipeline::new(None);

        let visualizer_caps = VideoCapsBuilder::new()
            .width(resulution.width as i32)
            .height(resulution.height as i32)
            .framerate(Fraction::new(frame_rate as i32, 1))
            .build();

        let uri_decode_bin = ElementFactory::make("uridecodebin")
            .property("uri", format!("file://{}", open_path.display()))
            .property("caps", Caps::builder("audio/x-raw").build())
            .build()
            .unwrap();

        let tee = ElementFactory::make("tee").build().unwrap();

        let audio_convert = ElementFactory::make("audioconvert").build().unwrap();

        let visualizer_element = VisualizerElement::new(visualizer);

        let container_caps = Caps::from_str(&encoding.container_caps).unwrap();
        let audio_caps = Caps::from_str(&encoding.audio_caps).unwrap();
        let video_caps = Caps::from_str(&encoding.video_caps).unwrap();

        let audio_profile = EncodingAudioProfile::builder(&audio_caps)
            .presence(0)
            .build();

        let video_profile = EncodingVideoProfile::builder(&video_caps)
            .presence(0)
            .build();

        let container_profile = EncodingContainerProfile::builder(&container_caps)
            .name("container")
            .add_profile(video_profile)
            .add_profile(audio_profile)
            .build();

        let encode_bin = ElementFactory::make("encodebin").build().unwrap();

        encode_bin.set_property("profile", &container_profile);

        let file_sink = ElementFactory::make("filesink")
            .property("location", format!("{}", save_path.display()))
            .build()
            .unwrap();

        pipeline.add(&uri_decode_bin).unwrap();
        pipeline.add(&encode_bin).unwrap();
        pipeline.add(&file_sink).unwrap();

        encode_bin.link(&file_sink).unwrap();

        {
            let pipeline = pipeline.downgrade();

            uri_decode_bin.connect_pad_added(move |_uri_decode_bin, src_pad| {
                let pipeline = if let Some(pipeline) = pipeline.upgrade() {
                    pipeline
                } else {
                    return;
                };

                pipeline.add(&tee).unwrap();
                pipeline.add(&audio_convert).unwrap();
                pipeline.add(&visualizer_element).unwrap();

                src_pad.link(&tee.static_pad("sink").unwrap()).unwrap();
                tee.link(&audio_convert).unwrap();
                audio_convert.link(&visualizer_element).unwrap();

                tee.link_pads(Some("src_%u"), &encode_bin, Some("audio_%u"))
                    .unwrap();

                visualizer_element
                    .link_pads_filtered(
                        Some("src"),
                        &encode_bin,
                        Some("video_%u"),
                        &visualizer_caps,
                    )
                    .unwrap();

                tee.sync_state_with_parent().unwrap();
                audio_convert.sync_state_with_parent().unwrap();
                visualizer_element.sync_state_with_parent().unwrap();
            });
        }

        pipeline.set_state(State::Playing).unwrap();

        let bus = pipeline
            .bus()
            .expect("Pipeline without bus. Shouldn't happen!");

        Self {
            pipeline,
            bus,
            name: format!("{}", save_path.file_name().unwrap().to_str().unwrap()),
            finished: false,
        }
    }
}

impl ExportProcess for URIExport {
    fn progress(&self) -> Option<f64> {
        Some(
            self.pipeline.query_position::<ClockTime>()?.nseconds() as f64
                / self.pipeline.query_duration::<ClockTime>()?.nseconds() as f64,
        )
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn finished(&self) -> bool {
        self.finished
    }

    fn update(&mut self) {
        for msg in self.bus.iter() {
            match msg.view() {
                MessageView::Eos(..) => {
                    self.finished = true;
                    break;
                }
                _ => (),
            }
        }
    }
}

impl Drop for URIExport {
    fn drop(&mut self) {
        self.pipeline.set_state(State::Null).unwrap();
    }
}
