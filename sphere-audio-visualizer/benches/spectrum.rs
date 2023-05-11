//! Benchmarks the spectrum analysis algorithm

use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use sphere_audio_visualizer::audio_analysis::{Samples, Spectrum};

pub fn spectrum_benchmark(c: &mut Criterion) {
    const SPHERE_COUNT: usize = 64;
    const SAMPLES: usize = 44100;

    let mut spectrum = Spectrum::default();
    let mut levels = vec![0.0f32; SPHERE_COUNT];

    c.bench_function("spectrum", |b| {
        b.iter(|| {
            for _ in 0..SAMPLES {
                let samples = Samples {
                    sample_rate: SAMPLES as f64,
                    samples: &[0.1],
                };

                for (spectrum_level, level) in spectrum.tick(samples).zip(&mut levels) {
                    *level = spectrum_level;
                }
            }
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = spectrum_benchmark
}
criterion_main!(benches);
