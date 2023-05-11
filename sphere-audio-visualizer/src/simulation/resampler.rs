use crate::{audio_analysis::Samples, Module};

const SIMULATION_FRAMERATE: f64 = 240.0;

/// Stores the settings of the [`SimulationResampler`]
#[derive(Clone)]
pub struct SimulationResamplerSettings {
    /// The simulator framerate used
    pub simulator_framerate: f64,
}

impl Default for SimulationResamplerSettings {
    fn default() -> Self {
        Self {
            simulator_framerate: SIMULATION_FRAMERATE,
        }
    }
}

struct SimulationResamplerIterator<'a> {
    samples: Samples<'a>,
    sample_pos: f64,
    samples_per_step: f64,
    samples_len: f64,
    first: bool,
}

impl<'a> SimulationResamplerIterator<'a> {
    pub fn new(samples: Samples<'a>, simulation_framerate: f64) -> Self {
        Self {
            first: true,
            sample_pos: 0.0,
            samples_per_step: samples.sample_rate / simulation_framerate,
            samples_len: samples.samples.len() as f64,
            samples,
        }
    }
}

impl<'a> Iterator for SimulationResamplerIterator<'a> {
    type Item = Samples<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let first = std::mem::replace(&mut self.first, false);

        if self.sample_pos >= self.samples_len {
            return if first {
                Some(self.samples.clone())
            } else {
                None
            };
        }

        let start_sample = self.sample_pos as usize;
        self.sample_pos += self.samples_per_step;
        let end_sample = (self.sample_pos as usize).min(self.samples.samples.len());

        let samples = Samples {
            sample_rate: self.samples.sample_rate,
            samples: &self.samples.samples[start_sample..end_sample],
        };

        Some(samples)
    }
}

/// Resamples the audio samples of one frame to a given framerate to archive consistent frame rate indipendent
/// simulation
pub struct SimulationResampler {
    simulation_framerate: f64,
}

impl SimulationResampler {
    /// Creates a new SimulationResampler with a given simulator framerate
    pub fn new(simulator_framerate: f64) -> Self {
        Self {
            simulation_framerate: simulator_framerate,
        }
    }

    /// Returns the simulator framerate
    pub fn simulator_framerate(&self) -> f64 {
        self.simulation_framerate
    }

    /// Sets the simulator framerate
    pub fn set_simulator_framerate(&mut self, simulator_framerate: f64) -> &mut Self {
        self.simulation_framerate = simulator_framerate;
        self
    }

    /// Sets the simulator framerate
    pub fn with_simulator_framerate(mut self, simulator_framerate: f64) -> Self {
        self.set_simulator_framerate(simulator_framerate);
        self
    }

    /// Resamples the audio samples of one frame to a given framerate to archive consistent frame rate indipendent
    /// simulation
    pub fn resample<'a>(&self, samples: Samples<'a>) -> impl Iterator<Item = Samples<'a>> {
        SimulationResamplerIterator::new(samples, self.simulation_framerate)
    }
}

impl Default for SimulationResampler {
    fn default() -> Self {
        Self::new(SIMULATION_FRAMERATE)
    }
}

impl Module for SimulationResampler {
    type Settings = SimulationResamplerSettings;

    fn set_settings(&mut self, settings: Self::Settings) -> &mut Self {
        self.set_simulator_framerate(settings.simulator_framerate)
    }

    fn settings(&self) -> Self::Settings {
        SimulationResamplerSettings {
            simulator_framerate: self.simulator_framerate(),
        }
    }
}
