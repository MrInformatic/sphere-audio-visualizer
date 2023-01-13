use std::ops::Range;

use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};

use super::Samples;
use crate::{audio_analysis::filter::IIRFilter, module::Module};

/// Defines the default amount of frequency bands for the audio analysis
const SPHERE_COUNT: usize = 64;

/// Defines the default lowest frequency for the audio analysis
const LOW_FREQUENCY: f32 = 20.0;

/// Defines the default highest frequency for the audio analysis
const HIGH_FREQUENCY: f32 = 20000.0;

/// Defines the default envelope attack for the audio analysis
const SPECTRUM_ATTACK: f32 = 0.005;

/// Defines the default envelope release for the audio analysis
const SPECTRUM_RELEASE: f32 = 0.4;

/// Defines the default envelope threshold for the audio analysis
const SPECTRUM_THRESHOLD: f32 = 0.1;

/// Stores the settings of audio analysis module
#[derive(Clone, PartialEq)]
pub struct SpectrumSettings {
    /// The amount of frequency bands
    pub count: usize,
    /// The lowest frequency
    pub low: f32,
    /// The highest frequency
    pub high: f32,
    /// The envelope threshhold
    pub threshold: f32,
    /// The envelope attack
    pub attack: f32,
    /// The envelope release
    pub release: f32,
}

impl Default for SpectrumSettings {
    fn default() -> Self {
        Self {
            count: SPHERE_COUNT,
            low: LOW_FREQUENCY,
            high: HIGH_FREQUENCY,
            threshold: SPECTRUM_THRESHOLD,
            attack: SPECTRUM_ATTACK,
            release: SPECTRUM_RELEASE,
        }
    }
}

/// The audio analysis module
pub struct Spectrum {
    envelope_bands: Vec<FrequencyBand>,
    settings: SpectrumSettings,
    attack: f32,
    release: f32,
    sample_rate: f64,
}

/// Implements the audio anaysis functionalities for one band of the analysis.
struct FrequencyBand {
    low_pass: IIRFilter,
    high_pass: IIRFilter,
    level: f32,
}

impl FrequencyBand {
    /// Creates a new instance. The struct has to be recreated if frequency
    /// range or sample rate is changed.
    pub fn new(range: Range<f32>, sample_rate: f32) -> Self {
        let low_pass = IIRFilter::low_pass(range.end, 1f32, sample_rate);

        let high_pass = IIRFilter::high_pass(range.start, 1f32, sample_rate);

        FrequencyBand {
            low_pass,
            high_pass,
            level: 0.0,
        }
    }

    /// Processes one sample and returns the level.
    /// the attack and release is adjusted the the per sample metric and is
    /// therefore independent from the sample rate.
    pub fn tick(&mut self, sample: f32, attack: f32, release: f32) {
        let sample = self.low_pass.tick(sample);
        let sample = self.high_pass.tick(sample);

        let factor = if self.level < sample { attack } else { release };

        self.level = factor * (self.level - sample) + sample;
    }
}

impl Spectrum {
    /// Processes multiple samples at once.
    /// Returns the levels after processing the last sample of the different
    /// bands as iterator.
    /// [`Spectrum::tick_par`] is prefered over this function on machines where a multi
    /// processor is present.
    pub fn tick(&mut self, samples: Samples) -> impl Iterator<Item = f32> + '_ {
        let old_sample_rate = self.sample_rate;
        self.sample_rate = samples.sample_rate;

        if self.sample_rate != old_sample_rate {
            self.update_envelope();
            self.update_bands();
        }

        for sample in samples.samples {
            for band in self.envelope_bands.iter_mut() {
                band.tick(*sample, self.attack, self.release)
            }
        }

        self.envelope_bands.iter().map(|band| band.level * 2.0)
    }

    /// Processes multiple samples at once.
    /// Returns the levels after processing the last sample of the different
    /// bands as iterator.
    /// This function is prefered over [`Spectrum::tick`] on machines where a multi processor
    /// is present.
    pub fn tick_par(&mut self, samples: Samples) -> impl Iterator<Item = f32> + '_ {
        let old_sample_rate = self.sample_rate;
        self.sample_rate = samples.sample_rate;

        if self.sample_rate != old_sample_rate {
            self.update_envelope();
            self.update_bands();
        }

        let attack = self.attack;
        let release = self.release;

        self.envelope_bands.par_iter_mut().for_each(move |band| {
            for sample in samples.samples {
                band.tick(*sample, attack, release)
            }
        });

        self.envelope_bands.iter().map(|band| band.level * 2.0)
    }

    fn update_envelope(&mut self) {
        let samples_per_attack = self.settings.attack * self.sample_rate as f32;
        let samples_per_release = self.settings.release * self.sample_rate as f32;

        self.attack = self.settings.threshold.powf(1f32 / samples_per_attack);
        self.release = self.settings.threshold.powf(1f32 / samples_per_release);
    }

    fn update_bands(&mut self) {
        self.envelope_bands.clear();

        let exponent =
            (self.settings.high / self.settings.low).powf(1.0 / self.settings.count as f32);

        for i in 0..self.settings.count {
            let low_cutoff = self.settings.low * exponent.powf(i as f32);
            let high_cutoff = self.settings.low * exponent.powf((i + 1) as f32);

            self.envelope_bands
                .push(FrequencyBand::new(low_cutoff..high_cutoff, 44100.0));
        }
    }
}

impl Default for Spectrum {
    fn default() -> Self {
        Self {
            envelope_bands: vec![],
            settings: SpectrumSettings {
                count: 0,
                low: 0.0,
                high: 0.0,
                threshold: 0.0,
                attack: 0.0,
                release: 0.0,
            },
            attack: 0.0,
            release: 0.0,
            sample_rate: 0.0,
        }
    }
}

impl Module for Spectrum {
    type Settings = SpectrumSettings;

    fn set_settings(&mut self, mut settings: Self::Settings) -> &mut Self {
        std::mem::swap(&mut self.settings, &mut settings);

        if self.settings.count != settings.count
            || self.settings.high != settings.high
            || self.settings.low != settings.low
        {
            self.update_bands();
        }

        if self.settings.attack != settings.attack
            || self.settings.release != settings.release
            || self.settings.threshold != settings.threshold
        {
            self.update_envelope();
        }

        self
    }

    fn settings(&self) -> Self::Settings {
        self.settings.clone()
    }
}
