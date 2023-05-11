//! Benchmarks the speed of the used physics simulation framework

use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use rand::{prelude::StdRng, Rng, SeedableRng};
use sphere_audio_visualizer::simulation::{Simulation2D, Simulation3D, Simulator};

/// Benchmakrs the speed of the 2d physics simulation
pub fn simulation_2d_benchmark(c: &mut Criterion) {
    const SPHERE_COUNT: usize = 64;
    const COUNT: usize = 60;

    let mut simulation = Simulation2D::new(0.1);

    let mut rng = StdRng::from_seed([0; 32]);

    let levels = vec![vec![rng.gen::<f32>(); SPHERE_COUNT]; COUNT];

    c.bench_function("simulation_2d", |b| {
        b.iter(|| {
            for levels in &levels {
                simulation.step(Duration::from_secs_f64(1.0 / 60.0), &levels);
                let _ = simulation.scene();
            }
        })
    });
}

/// Benchmakrs the speed of the 3d physics simulation
pub fn simulation_3d_benchmark(c: &mut Criterion) {
    const SPHERE_COUNT: usize = 64;
    const COUNT: usize = 60;

    let mut simulation = Simulation3D::new(0.1);

    let mut rng = StdRng::from_seed([0; 32]);

    let levels = vec![vec![rng.gen::<f32>(); SPHERE_COUNT]; COUNT];

    c.bench_function("simulation_3d", |b| {
        b.iter(|| {
            for levels in &levels {
                simulation.step(Duration::from_secs_f64(1.0 / 60.0), &levels);
                let _ = simulation.scene();
            }
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = simulation_2d_benchmark, simulation_3d_benchmark
}

criterion_main!(benches);
