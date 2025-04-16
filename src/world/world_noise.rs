use bevy::prelude::*;
use core::f64;
use noise::{
    permutationtable::{NoiseHasher, PermutationTable},
    NoiseFn, ScalePoint, Simplex,
};
use std::{ops::Mul, sync::Arc};

#[derive(Resource, Clone)]
pub struct ContinentNoiseGenerator(pub Arc<StackedNoise>);

impl ContinentNoiseGenerator {
    pub fn new(seed: u32) -> Self {
        let noise = StackedNoise::new(seed, 6, 1000.0);
        Self(Arc::new(noise))
    }
}

#[derive(Resource, Clone)]
pub struct HeightNoiseGenerator(pub Arc<StackedNoise>);

impl HeightNoiseGenerator {
    pub fn new(seed: u32) -> Self {
        let noise = StackedNoise::new(seed, 3, 100.0);
        Self(Arc::new(noise))
    }
}

#[derive(Resource, Clone)]
pub struct WhiteNoise {
    permutation_table: PermutationTable,
}

impl WhiteNoise {
    pub fn new(seed: u32) -> Self {
        let permutation_table = PermutationTable::new(seed);
        Self { permutation_table }
    }
}

impl NoiseFn<i32, 3> for WhiteNoise {
    fn get(&self, [x, y, z]: [i32; 3]) -> f64 {
        let result = self
            .permutation_table
            .hash(&[x as isize, y as isize, z as isize]) as u8;
        const MAXIMUM: u8 = 0xFF;
        return (result % MAXIMUM) as f64 / (MAXIMUM as f64);
    }
}

#[derive(Resource, Clone)]
pub struct CaveNetworkNoiseGenerator {
    noise_a: Arc<ScalePoint<Simplex>>,
    noise_b: Arc<ScalePoint<Simplex>>,
}

const SCALE_A: f64 = 0.01;

impl CaveNetworkNoiseGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            noise_a: Arc::new(
                ScalePoint::new(Simplex::new(seed.rotate_left(0))).set_scale(SCALE_A * 0.9),
            ),
            noise_b: Arc::new(
                ScalePoint::new(Simplex::new(seed.rotate_left(1))).set_scale(SCALE_A * 1.1),
            ),
        }
    }
}

const NOISE_A_MAX: f64 = 0.075;

impl NoiseFn<i32, 3> for CaveNetworkNoiseGenerator {
    fn get(&self, point: [i32; 3]) -> f64 {
        let point = point.map(|x| x as f64);
        let sample_a = self.noise_a.get(point).abs() + 1.0;
        let sample_b = self.noise_b.get(point).abs() + 1.0;
        let sample = sample_a.mul(sample_b) - 1.0;
        if sample > NOISE_A_MAX {
            1.
        } else {
            0.
        }
    }
}

struct NoiseGenerator {
    perlin: Simplex,
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

pub struct StackedNoise {
    noises: Vec<NoiseGenerator>,
}

impl StackedNoise {
    fn new(seed: u32, num_layers: u32, starting_scale: f64) -> Self {
        let noises = (0..num_layers)
            .map(|i| {
                let amplitude = 0.5_f64.powi(i as i32);
                let scale = starting_scale * amplitude;
                let offset = starting_scale * i as f64;
                let perlin = Simplex::new(seed.rotate_left(i));
                return NoiseGenerator {
                    perlin,
                    scale,
                    amplitude,
                    offset,
                };
            })
            .collect();
        Self { noises }
    }
}

impl NoiseFn<i32, 2> for StackedNoise {
    fn get(&self, point: [i32; 2]) -> f64 {
        let (amp, sample) = self
            .noises
            .iter()
            .map(|n| (n.amplitude, n.get(point)))
            .reduce(|(agg_amp, agg_sample), (amp, sample)| ((agg_amp + amp), (agg_sample + sample)))
            .unwrap_or((1., 0.));
        (sample / amp) * 0.5 + 0.5
    }
}

impl NoiseFn<i32, 3> for StackedNoise {
    fn get(&self, point: [i32; 3]) -> f64 {
        let (amp, sample) = self
            .noises
            .iter()
            .map(|n| (n.amplitude, n.get(point)))
            .reduce(|(agg_amp, agg_sample), (amp, sample)| ((agg_amp + amp), (agg_sample + sample)))
            .unwrap_or((1., 0.));
        (sample / amp) * 0.5 + 0.5
    }
}
