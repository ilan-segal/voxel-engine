use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::{f64::consts::E, sync::Arc};

#[derive(Resource, Clone)]
pub struct WorldGenNoise(Arc<WorldGenNoiseInner>);

struct WorldGenNoiseInner {
    noise_a: StackedNoise,
    noise_b: StackedNoise,
    regime: NoiseGenerator,
    sharpener: NoiseGenerator,
    white_noise: WhiteNoise,
}

impl WorldGenNoise {
    pub fn new(seed: u32) -> Self {
        let inner = WorldGenNoiseInner {
            white_noise: WhiteNoise::new(seed),
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
            sharpener: NoiseGenerator {
                perlin: Perlin::new(seed << 2),
                scale: 150.0,
                amplitude: 1.0,
                offset: 0.0,
            },
        };
        Self(Arc::new(inner))
    }

    pub fn white_noise(&self) -> &impl NoiseFn<i32, 3> {
        &self.0.white_noise
    }
}

impl NoiseFn<i32, 2> for WorldGenNoise {
    fn get(&self, point: [i32; 2]) -> f64 {
        let naive_regime = (self.0.regime.get(point) + 1.0) * 0.5;
        let sharpness = (self.0.sharpener.get(point) + 1.0) * 0.5 * 59.0 + 1.0;
        let regime = sharpen_noise(naive_regime, sharpness);
        let sample_a = self.0.noise_a.get(point);
        let sample_b = self.0.noise_b.get(point) * 0.5 + 1.0;
        return regime * sample_a + (1.0 - regime) * sample_b;
    }
}

impl NoiseFn<i32, 3> for WorldGenNoise {
    fn get(&self, point: [i32; 3]) -> f64 {
        self.0.noise_a.get(point)
    }
}

struct WhiteNoise {
    seed: u32,
}

impl WhiteNoise {
    fn new(seed: u32) -> Self {
        Self { seed }
    }
}

impl NoiseFn<i32, 3> for WhiteNoise {
    fn get(&self, [x, y, z]: [i32; 3]) -> f64 {
        let x = fast_hash(x + 1 * self.seed as i32);
        let y = fast_hash(y ^ self.seed as i32);
        let z = fast_hash(z + self.seed as i32);
        let result = (x ^ y ^ z) as u32;
        const MAXIMUM: u32 = 0xDEADBEEF;
        return (result % MAXIMUM) as f64 / (MAXIMUM as f64);
    }
}

fn fast_hash(a: i32) -> u32 {
    let mut a = a.abs() as u32;
    a = (a ^ 61) ^ (a >> 16);
    a = a + (a << 3);
    a = a ^ (a >> 4);
    a = a.wrapping_mul(0x27d4eb2d);
    a = a ^ (a >> 15);
    return a;
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
        return self.get([x, y, 0]);
    }
}

impl NoiseFn<i32, 3> for NoiseGenerator {
    fn get(&self, point: [i32; 3]) -> f64 {
        let [x, y, z] = point;
        let sample_x = x as f64 / self.scale + self.offset;
        let sample_y = y as f64 / self.scale + self.offset;
        let sample_z = z as f64 / self.scale + self.offset;
        return self
            .perlin
            .get([sample_x, sample_y, sample_z])
            * self.amplitude;
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

impl NoiseFn<i32, 3> for StackedNoise {
    fn get(&self, point: [i32; 3]) -> f64 {
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
