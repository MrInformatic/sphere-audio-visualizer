//! Contains the implemntation of the frontend for the sphere audio visualizer.

use std::any::Any;

use egui::Ui;

pub use self::{app::*, drawer::*};
use crate::{
    audio_analysis::Samples, rendering::wgpu::OutputFormat, visualizer::OfflineVisualizer,
};

mod app;
mod drawer;

/// An [`OnlineSampleSource`] is used by an [`Application`] get the current
/// samples for analysis from a sample source which creates new samples while
/// the application is running.
pub trait OnlineSampleSource: Any {
    /// Returns a new batch of sampes for analysis.
    fn samples(&mut self) -> Samples;

    /// This function is invoked if the this sample source is selected by the
    /// user in the application.
    fn focus(&mut self);

    /// This function is invoked if another sample source is selected by the
    /// user in the application.
    fn unfocus(&mut self);

    /// Is invoked to draw some aditional UI with egui to configure the
    /// [`OnlineSampleSource`].
    fn ui(&mut self, ui: &mut Ui);
}

/// The [`Exporter`] is used by the [`Application`] request [`ExportProcess`]es.
pub trait Exporter {
    /// The output format that the [`OfflineVisualizer`] should use.
    fn format(&self) -> OutputFormat;

    /// Returns if the exporter is currently able to export. If this is false
    /// the button in the UI is greyed out.
    fn can_export(&self) -> bool;

    /// Creates a new export process from a [`OfflineVisualizer`].
    fn export(&mut self, visualizer: Box<dyn OfflineVisualizer>) -> Option<Box<dyn ExportProcess>>;

    /// Is invoked to draw some aditional UI with egui to configure the
    /// [`Exporter`].
    fn ui(&mut self, ui: &mut Ui);
}

/// Defines the interface that a export process has to support. export
/// processes are created by an [`Exporter`]
pub trait ExportProcess {
    /// Retuns the progress of a export process between 0.0-1.0. The progress
    /// is optional if no progress could be found because the export is not
    /// started yet. If this is the case there is also no progress shown in the
    /// ui.
    fn progress(&self) -> Option<f64>;

    /// The name of the export process. This should be identifiable by the user,
    /// since it is the value shown in the ui.
    fn name(&self) -> &str;

    /// Returns if the export process is finished if this function returns
    /// false the process is poped out of the queue and droped.
    fn finished(&self) -> bool;

    /// Is executed regulary to maintain the internal values this function
    /// should not block. This means export processes should opperate
    /// concurrently in e.g. a different thread.
    fn update(&mut self);
}
