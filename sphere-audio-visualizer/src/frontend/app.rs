use std::ops::Add;

use egui::{Button, ComboBox, Context, FullOutput, Grid, ProgressBar, RawInput, Ui};
use egui_wgpu_backend::ScreenDescriptor;
use egui_winit::State;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use super::{drawer::UiDrawer, ExportProcess, Exporter, OnlineSampleSource, Samples};
use crate::{
    rendering::wgpu::EGUIScene,
    visualizer::{DynamicVisualizer, OnlineVisualizer, VisualizerFactory},
};

struct VisualizerConfiguration {
    name: String,
    change_visualizer: fn(&mut DynamicVisualizer, &Window),
    settings_drawer: fn(&mut DynamicVisualizer, &mut Ui),
}

struct SampleSourceConfiguration {
    name: String,
    online_sample_source: Box<dyn OnlineSampleSource>,
    exporter_mapper: Option<fn(&mut dyn OnlineSampleSource) -> &mut dyn Exporter>,
}

impl SampleSourceConfiguration {
    fn from_sample_source<T: OnlineSampleSource + Exporter>(
        name: impl ToString,
        sample_source: T,
    ) -> Self {
        Self {
            name: name.to_string(),
            online_sample_source: Box::new(sample_source),
            exporter_mapper: Some(|sample_source| {
                (unsafe { &mut *(sample_source as *mut _ as *mut T) }) as &mut dyn Exporter
            }),
        }
    }

    fn from_online_only_sample_source(
        name: impl ToString,
        sample_source: impl OnlineSampleSource,
    ) -> Self {
        Self {
            name: name.to_string(),
            online_sample_source: Box::new(sample_source),
            exporter_mapper: None,
        }
    }

    pub fn exporter(&mut self) -> Option<&mut dyn Exporter> {
        Some((self.exporter_mapper?)(self.online_sample_source.as_mut()))
    }
}

impl OnlineSampleSource for SampleSourceConfiguration {
    fn samples(&mut self) -> Samples {
        self.online_sample_source.samples()
    }

    fn ui(&mut self, ui: &mut Ui) {
        self.online_sample_source.ui(ui)
    }

    fn unfocus(&mut self) {
        self.online_sample_source.unfocus()
    }

    fn focus(&mut self) {
        self.online_sample_source.focus()
    }
}

/// This is the central struct of the sphere audio visualizer. It manages the
/// audio sample sources, exporter, export processes and visualizers. It also
/// contains the winit event loop and the coarse structure of the UI.
pub struct Application {
    visualizer: DynamicVisualizer,
    window: Window,
    event_loop: Option<EventLoop<()>>,
    context: Context,
    state: State,
    selected_visualizer_id: usize,
    visualizer_configurations: Vec<VisualizerConfiguration>,
    selected_sample_source_id: usize,
    sample_source_configurations: Vec<SampleSourceConfiguration>,
    export_progresses: Vec<Box<dyn ExportProcess>>,
    show_individual_progress: bool,
}

impl Application {
    /// Creates a new instance from a winit [`WindowBuilder`]
    pub fn new(window_builder: WindowBuilder) -> Self {
        let event_loop = EventLoop::new();
        let window = window_builder.build(&event_loop).unwrap();
        let state = State::new(8192, &window);

        let visualizer = DynamicVisualizer::new();

        Self {
            visualizer,
            window,
            event_loop: Some(event_loop),
            context: Context::default(),
            state,
            selected_visualizer_id: 0,
            visualizer_configurations: Vec::new(),
            selected_sample_source_id: 0,
            sample_source_configurations: Vec::new(),
            export_progresses: Vec::new(),
            show_individual_progress: false,
        }
    }

    /// adds a new visualizer configuration. The name is displayed in the UI.
    pub fn with_visualizer_configuration<F, S>(mut self, name: S) -> Self
    where
        F: VisualizerFactory,
        F::OnlineVisualizer: UiDrawer,
        S: ToString,
    {
        if self.visualizer_configurations.is_empty() {
            self.visualizer.change_visualizer::<F>(&self.window);
        }

        self.visualizer_configurations
            .push(VisualizerConfiguration {
                name: name.to_string(),
                change_visualizer: |visualizer, window| visualizer.change_visualizer::<F>(window),
                settings_drawer: |visualizer, ui| {
                    if let Some(online_visualizer) =
                        visualizer.online_visualizer_mut::<F::OnlineVisualizer>()
                    {
                        online_visualizer.ui(ui);
                    }
                },
            });

        self
    }

    /// addss a new online only sample source (without [`Exporter`]).
    /// The name is displayed in the UI.
    pub fn with_online_only_sample_source(
        mut self,
        mut sample_source: impl OnlineSampleSource,
        name: impl ToString,
    ) -> Self {
        if self.sample_source_configurations.len() == self.selected_sample_source_id {
            sample_source.focus()
        }

        self.sample_source_configurations.push(
            SampleSourceConfiguration::from_online_only_sample_source(name, sample_source),
        );
        self
    }

    /// addss a new online only sample source (with [`Exporter`]).
    /// The name is displayed in the UI.
    pub fn with_sample_source(
        mut self,
        mut sample_source: impl OnlineSampleSource + Exporter,
        name: impl ToString,
    ) -> Self {
        if self.sample_source_configurations.len() == self.selected_sample_source_id {
            sample_source.focus()
        }

        self.sample_source_configurations
            .push(SampleSourceConfiguration::from_sample_source(
                name,
                sample_source,
            ));
        self
    }

    /// Starts the winit event loop. Also blocks until the application exists.
    pub fn run(mut self) {
        if let Some(event_loop) = self.event_loop.take() {
            event_loop.run(move |event, _, controll_flow| {
                *controll_flow = ControlFlow::Poll;

                match event {
                    Event::RedrawRequested(_) => self.render(),
                    Event::RedrawEventsCleared => self.window.request_redraw(),
                    Event::WindowEvent { event, window_id } => {
                        if self.window.id() == window_id {
                            self.state.on_event(&self.context, &event);

                            match event {
                                WindowEvent::CloseRequested => {
                                    *controll_flow = ControlFlow::Exit;
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                };
            })
        }
    }

    fn render(&mut self) {
        for process in &mut self.export_progresses {
            process.update()
        }

        self.export_progresses
            .drain_filter(|process| process.finished());

        let new_input = self.state.take_egui_input(&self.window);

        let FullOutput {
            platform_output,
            textures_delta,
            shapes,
            ..
        } = self.show(new_input);
        self.state
            .handle_platform_output(&self.window, &self.context, platform_output);

        let size = self.window.inner_size();

        let scene_descriptor = ScreenDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: self.state.pixels_per_point(),
        };

        let egui_scene = EGUIScene::new(&self.context, textures_delta, shapes, scene_descriptor);

        let samples = self.sample_source_configurations[self.selected_sample_source_id].samples();

        self.visualizer
            .visualize(samples, size.width, size.height, egui_scene);
    }

    fn show(&mut self, new_input: RawInput) -> FullOutput {
        self.context.run(new_input, |ctx| {
            egui::Window::new("Settings").show(ctx, |ui| {
                ui.heading("Audio:");

                Grid::new("Audio Source Grid")
                    .num_columns(2)
                    .min_col_width(72.0)
                    .show(ui, |ui| {
                        ui.label("Source:");
                        let old_selected_sample_source_id = self.selected_sample_source_id;
                        let audio_source_name =
                            &self.sample_source_configurations[self.selected_sample_source_id].name;
                        ComboBox::from_id_source("Audio Source Selector")
                            .selected_text(audio_source_name)
                            .width(168.0)
                            .show_ui(ui, |ui| {
                                for (id, sample_source_configuration) in
                                    self.sample_source_configurations.iter().enumerate()
                                {
                                    ui.selectable_value(
                                        &mut self.selected_sample_source_id,
                                        id,
                                        &sample_source_configuration.name,
                                    );
                                }
                            });
                        ui.end_row();

                        if old_selected_sample_source_id != self.selected_sample_source_id {
                            self.sample_source_configurations[old_selected_sample_source_id]
                                .unfocus();
                            self.sample_source_configurations[self.selected_sample_source_id]
                                .focus();
                        }
                    });

                self.sample_source_configurations[self.selected_sample_source_id].ui(ui);

                ui.heading("Settings:");

                Grid::new("Settings Grid")
                    .num_columns(2)
                    .striped(true)
                    .min_col_width(124.0)
                    .max_col_width(124.0)
                    .show(ui, |ui| {
                        ui.label("Visualizer:");
                        let visualizer_name =
                            &self.visualizer_configurations[self.selected_visualizer_id].name;
                        ComboBox::from_id_source("Visualizer Selector")
                            .selected_text(visualizer_name)
                            .width(116.0)
                            .show_ui(ui, |ui| {
                                for (id, visualizer_configuration) in
                                    self.visualizer_configurations.iter().enumerate()
                                {
                                    if ui
                                        .selectable_value(
                                            &mut self.selected_visualizer_id,
                                            id,
                                            &visualizer_configuration.name,
                                        )
                                        .changed()
                                    {
                                        (visualizer_configuration.change_visualizer)(
                                            &mut self.visualizer,
                                            &self.window,
                                        );
                                    }
                                }
                            });
                        ui.end_row();

                        (self.visualizer_configurations[self.selected_visualizer_id]
                            .settings_drawer)(&mut self.visualizer, ui);
                    });

                if let Some(exporter) =
                    self.sample_source_configurations[self.selected_sample_source_id].exporter()
                {
                    ui.heading("Export:");

                    exporter.ui(ui);

                    ui.add_enabled_ui(exporter.can_export(), |ui| {
                        if ui.add_sized([256.0, 20.0], Button::new("Export")).clicked() {
                            if let Some(visualizer) =
                                self.visualizer.offline_visualizer(exporter.format())
                            {
                                if let Some(process) = exporter.export(visualizer) {
                                    self.export_progresses.push(process)
                                }
                            }
                        }
                    });

                    if let Some(progress) = self
                        .export_progresses
                        .iter()
                        .filter_map(|process| process.progress())
                        .reduce(Add::add)
                        .map(|sum| sum / self.export_progresses.len() as f64)
                    {
                        Grid::new("Export Progress Grid")
                            .num_columns(2)
                            .min_col_width(72.0)
                            .show(ui, |ui| {
                                ui.label("Progress:");

                                ui.add_sized(
                                    [176.0, 20.0],
                                    ProgressBar::new(progress as f32).show_percentage(),
                                );
                            });
                    }

                    if !self.export_progresses.is_empty() {
                        if ui
                            .add_sized(
                                [256.0, 20.0],
                                Button::new(format!(
                                    "Running Processes ({})",
                                    self.export_progresses.len()
                                )),
                            )
                            .clicked()
                        {
                            self.show_individual_progress = !self.show_individual_progress;
                        }
                    }
                }
            });

            if self.export_progresses.is_empty() {
                self.show_individual_progress = false;
            }

            egui::Window::new("Individual Progress")
                .open(&mut self.show_individual_progress)
                .show(ctx, |ui| {
                    Grid::new("individual progress table")
                        .num_columns(3)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.label("Progress:");
                            ui.label("");
                            ui.end_row();

                            self.export_progresses.drain_filter(|process| {
                                ui.label(process.name());
                                if let Some(progress) = process.progress() {
                                    ui.add(ProgressBar::new(progress as f32).show_percentage());
                                } else {
                                    ui.label("Not Avaliable");
                                }
                                let cancel = ui.button("x").clicked();
                                ui.end_row();
                                cancel
                            });
                        })
                });
        })
    }
}
