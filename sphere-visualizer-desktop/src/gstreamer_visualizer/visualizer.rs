#![allow(missing_docs)]

use gstreamer::{
    glib::{self, object_subclass, types::Pointee, wrapper, ParamSpec, ParamSpecPointer, Value},
    prelude::ElementExtManual,
    subclass::prelude::{
        ElementImpl, GstObjectImpl, ObjectImpl, ObjectSubclass, ObjectSubclassExt,
    },
    traits::PadExt,
    Element, Object, PadDirection, PadPresence, PadTemplate,
};
use gstreamer_audio::{AudioCapsBuilder, AUDIO_FORMAT_F32};
use gstreamer_pbutils::{subclass::prelude::AudioVisualizerImpl, AudioVisualizer};
use gstreamer_video::{VideoCapsBuilder, VideoFormat};
use lazy_static::__Deref;
use sphere_visualizer::{audio_analysis::Samples, OfflineVisualizer};
use std::{ops::DerefMut, ptr::NonNull, sync::Mutex};

/// Inner Implementation of the [`VisualizerElement`]
pub struct VisualizerElementImpl(Mutex<Option<Box<dyn OfflineVisualizer>>>);

impl VisualizerElementImpl {
    fn sample_rate(&self) -> Option<f64> {
        Some(
            self.obj()
                .sink_pads()
                .get(0)?
                .caps()?
                .structure(0)?
                .get::<i32>("rate")
                .ok()? as f64,
        )
    }
}

impl Default for VisualizerElementImpl {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

#[object_subclass]
impl ObjectSubclass for VisualizerElementImpl {
    const NAME: &'static str = "VisualizerElement";
    type Type = VisualizerElement;
    type ParentType = AudioVisualizer;
    type Interfaces = ();
}

impl ObjectImpl for VisualizerElementImpl {
    fn properties() -> &'static [ParamSpec] {
        lazy_static::lazy_static! {
            static ref PROPERTIES: [ParamSpec; 1] =
                [ParamSpecPointer::builder("visualizer").build()];
        }

        PROPERTIES.deref()
    }

    fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "visualizer" => {
                *self.0.lock().unwrap() = value
                    .get::<Option<NonNull<Pointee>>>()
                    .ok()
                    .flatten()
                    .map(visualizer_from_ptr);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _id: usize, _pspec: &ParamSpec) -> Value {
        unimplemented!()
    }
}

impl GstObjectImpl for VisualizerElementImpl {}

impl ElementImpl for VisualizerElementImpl {
    fn pad_templates() -> &'static [PadTemplate] {
        lazy_static::lazy_static! {
            static ref PAD_TEMPLATES: [PadTemplate; 2] = [
                PadTemplate::new(
                    "sink",
                    PadDirection::Sink,
                    PadPresence::Always,
                    &AudioCapsBuilder::new()
                        .format(AUDIO_FORMAT_F32)
                        .channels(1i32)
                        .build(),
                    )
                    .unwrap(),
                PadTemplate::new(
                    "src",
                    PadDirection::Src,
                    PadPresence::Always,
                    &VideoCapsBuilder::new()
                        .format(VideoFormat::Rgba)
                        .build()
                    )
                    .unwrap()
            ];
        }

        PAD_TEMPLATES.deref()
    }
}

impl AudioVisualizerImpl for VisualizerElementImpl {
    fn render(
        &self,
        audio_buffer: &gstreamer::BufferRef,
        video_frame: &mut gstreamer_video::VideoFrameRef<&mut gstreamer::BufferRef>,
    ) -> Result<(), gstreamer::LoggableError> {
        if let Some(visualizer) = self.0.lock().unwrap().as_mut() {
            let mapped_audio_buffer = audio_buffer.map_readable().unwrap();

            let slice = mapped_audio_buffer.as_slice();
            let sample_count = slice.len() * std::mem::size_of::<u8>() / std::mem::size_of::<f32>();
            let ptr = slice.as_ptr() as *const f32;
            let samples = unsafe { &*std::ptr::slice_from_raw_parts(ptr, sample_count) };

            let samples = Samples {
                sample_rate: self.sample_rate().unwrap_or(44100.0),
                samples: samples,
            };

            let width = video_frame.width();
            let height = video_frame.height();

            let output = visualizer.visualize(samples, width, height);

            video_frame
                .plane_data_mut(0)
                .unwrap()
                .copy_from_slice(&output.data);
        }

        Ok(())
    }
}

fn visualizer_into_ptr(visualizer: &mut Box<dyn OfflineVisualizer>) -> NonNull<Pointee> {
    unsafe { NonNull::new_unchecked(visualizer as *mut _ as *mut Pointee) }
}

fn visualizer_from_ptr(visualizer: NonNull<Pointee>) -> Box<dyn OfflineVisualizer> {
    unsafe {
        Box::from_raw((*(visualizer.as_ptr() as *mut Box<dyn OfflineVisualizer>)).deref_mut())
    }
}

wrapper! {
    /// A GStreamer Elemenet using a visualizer for audio visualization
    pub struct VisualizerElement(ObjectSubclass<VisualizerElementImpl>) @extends AudioVisualizer, Element, Object;
}

impl VisualizerElement {
    /// Creates a new instance.
    pub fn new(mut visualizer: Box<dyn OfflineVisualizer>) -> Self {
        let element = glib::Object::new(&[("visualizer", &visualizer_into_ptr(&mut visualizer))]);

        std::mem::forget(visualizer);

        element
    }
}
