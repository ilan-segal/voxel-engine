use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::sync::Arc;

#[derive(Resource, Clone)]
pub struct ContinentNoiseGenerator(pub Arc<StackedNoise>);

impl ContinentNoiseGenerator {
    pub fn new(seed: u32) -> Self {
        let noise = StackedNoise(vec![
            NoiseGenerator {
                perlin: Perlin::new(seed.rotate_left(1)),
                scale: 1000.0,
                amplitude: 1.0,
                offset: 0.0,
            },
            NoiseGenerator {
                perlin: Perlin::new(seed.rotate_left(2)),
                scale: 500.0,
                amplitude: 0.5,
                offset: 10.0,
            },
            NoiseGenerator {
                perlin: Perlin::new(seed.rotate_left(3)),
                scale: 250.0,
                amplitude: 0.25,
                offset: 20.0,
            },
            NoiseGenerator {
                perlin: Perlin::new(seed.rotate_left(4)),
                scale: 125.0,
                amplitude: 0.125,
                offset: 20.0,
            },
            NoiseGenerator {
                perlin: Perlin::new(seed.rotate_left(5)),
                scale: 62.5,
                amplitude: 0.0625,
                offset: 20.0,
            },
            NoiseGenerator {
                perlin: Perlin::new(seed.rotate_left(6)),
                scale: 31.25,
                amplitude: 0.03125,
                offset: 20.0,
            },
        ]);
        Self(Arc::new(noise))
    }
}

#[derive(Resource, Clone)]
pub struct HeightNoiseGenerator(pub Arc<StackedNoise>);

impl HeightNoiseGenerator {
    pub fn new(seed: u32) -> Self {
        let noise = StackedNoise(vec![
            NoiseGenerator {
                perlin: Perlin::new(seed.rotate_left(4)),
                scale: 100.0,
                amplitude: 1.0,
                offset: 0.0,
            },
            NoiseGenerator {
                perlin: Perlin::new(seed.rotate_left(5)),
                scale: 50.0,
                amplitude: 0.5,
                offset: 10.0,
            },
            NoiseGenerator {
                perlin: Perlin::new(seed.rotate_left(6)),
                scale: 25.0,
                amplitude: 0.25,
                offset: 20.0,
            },
        ]);
        Self(Arc::new(noise))
    }
}

// struct WhiteNoise {
//     seed: u32,
// }

// impl WhiteNoise {
//     fn new(seed: u32) -> Self {
//         Self { seed }
//     }
// }

// impl NoiseFn<i32, 3> for WhiteNoise {
//     fn get(&self, [x, y, z]: [i32; 3]) -> f64 {
//         let x = fast_hash(x + 1 * self.seed as i32);
//         let y = fast_hash(y ^ self.seed as i32);
//         let z = fast_hash(z + self.seed as i32);
//         let result = (x ^ y ^ z) as u32;
//         const MAXIMUM: u32 = 0xDEADBEEF;
//         return (result % MAXIMUM) as f64 / (MAXIMUM as f64);
//     }
// }

// fn fast_hash(a: i32) -> u32 {
//     let mut a = a.abs() as u32;
//     a = (a ^ 61) ^ (a >> 16);
//     a = a + (a << 3);
//     a = a ^ (a >> 4);
//     a = a.wrapping_mul(0x27d4eb2d);
//     a = a ^ (a >> 15);
//     return a;
// }

struct NoiseGenerator {
    perlin: Perlin,
    scale: f64,
    amplitude: f64,
    offset: f64,
}

impl NoiseFn<i32, 2> for NoiseGenerator {
    fn get(&self, [x, y]: [i32; 2]) -> f64 {
        let sample_x = x as f64 / self.scale + self.offset;
        let sample_y = y as f64 / self.scale + self.offset;
        return self.perlin.get([sample_x, sample_y]) * self.amplitude;
    }
}

impl NoiseFn<i32, 3> for NoiseGenerator {
    fn get(&self, [x, y, z]: [i32; 3]) -> f64 {
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
// */
// fn sharpen_noise(value: f64, amount: f64) -> f64 {
//     if amount < 1.0 {
//         panic!();
//     }
//     let exaggerated = (value - 0.5) * amount;
//     return sigmoid(exaggerated);
// }

// fn sigmoid(x: f64) -> f64 {
//     (1.0 + E.powf(-x)).recip()
// }

pub struct StackedNoise(Vec<NoiseGenerator>);

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
