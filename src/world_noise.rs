use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::{f64::consts::E, sync::Arc};

#[derive(Resource, Clone)]
pub struct WorldGenNoise(Arc<WorldGenNoiseInner>);

struct WorldGenNoiseInner {
    noise_a: StackedNoise,
    noise_b: StackedNoise,
    regime: NoiseGenerator,
}

impl WorldGenNoise {
    pub fn new(seed: u32) -> Self {
        let inner = WorldGenNoiseInner {
            noise_a: StackedNoise(vec![
                NoiseGenerator {
                    perlin: Perlin::new(seed),
                    scale: 100.0,
                    amplitude: 1.0,
                    offset: 0.0,
                },
                NoiseGenerator {
                    perlin: Perlin::new(seed),
                    scale: 50.0,
                    amplitude: 0.5,
                    offset: 10.0,
                },
                NoiseGenerator {
                    perlin: Perlin::new(seed),
                    scale: 25.0,
                    amplitude: 0.25,
                    offset: 20.0,
                },
            ]),
            noise_b: StackedNoise(vec![
                NoiseGenerator {
                    perlin: Perlin::new(!seed),
                    scale: 100.0,
                    amplitude: 1.0,
                    offset: 0.0,
                },
                NoiseGenerator {
                    perlin: Perlin::new(!seed),
                    scale: 50.0,
                    amplitude: 0.5,
                    offset: 10.0,
                },
                NoiseGenerator {
                    perlin: Perlin::new(!seed),
                    scale: 25.0,
                    amplitude: 0.25,
                    offset: 20.0,
                },
            ]),
            regime: NoiseGenerator {
                perlin: Perlin::new(seed << 1),
                scale: 150.0,
                amplitude: 1.0,
                offset: 0.0,
            },
        };
        Self(Arc::new(inner))
    }
}

impl NoiseFn<i32, 2> for WorldGenNoise {
    fn get(&self, point: [i32; 2]) -> f64 {
        let naive_regime = (self.0.regime.get(point) + 1.0) * 0.5;
        let regime = sharpen_noise(naive_regime, 20.0);
        let sample_a = self.0.noise_a.get(point);
        let sample_b = self.0.noise_b.get(point) * 0.5 + 1.0;
        return regime * sample_a + (1.0 - regime) * sample_b;
    }
}

struct NoiseGenerator {
    perlin: Perlin,
    scale: f64,
    amplitude: f64,
    offset: f64,
}

impl NoiseFn<i32, 2> for NoiseGenerator {
    fn get(&self, point: [i32; 2]) -> f64 {
        let [x, y] = point;
        let sample_x = x as f64 / self.scale + self.offset;
        let sample_y = y as f64 / self.scale + self.offset;
        return self.perlin.get([sample_x, sample_y]) * self.amplitude;
    }
}

/*
value in [0, 1]
amount in [1, ∞)
*/
fn sharpen_noise(value: f64, amount: f64) -> f64 {
    if amount < 1.0 {
        panic!();
    }
    let exaggerated = (value - 0.5) * amount;
    return sigmoid(exaggerated);
}

fn sigmoid(x: f64) -> f64 {
    (1.0 + E.powf(-x)).recip()
}

struct StackedNoise(Vec<NoiseGenerator>);

impl NoiseFn<i32, 2> for StackedNoise {
    fn get(&self, point: [i32; 2]) -> f64 {
        let mut total_sample = 0.;
        let mut total_amplitude = 0.;
        for g in &self.0 {
            total_sample += g.get(point);
            total_amplitude += g.amplitude;
        }
        total_sample /= total_amplitude;
        return 0.5 + 0.5 * total_sample;
    }
}
