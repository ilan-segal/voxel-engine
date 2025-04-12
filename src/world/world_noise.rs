use bevy::{
    math::{DVec2, DVec3},
    prelude::*,
};
use core::f64;
use noise::{
    permutationtable::{NoiseHasher, PermutationTable},
    NoiseFn, Simplex,
};
use std::sync::Arc;

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
        // let x = fast_hash_i32(x + self.seed as i32);
        // let y = fast_hash_i32(y ^ self.seed as i32);
        // let z = fast_hash_i32(z ^ self.seed.wrapping_shl(16) as i32);
        let result = self
            .permutation_table
            .hash(&[x as isize, y as isize, z as isize]) as u8;
        const MAXIMUM: u8 = 0xFF;
        return (result % MAXIMUM) as f64 / (MAXIMUM as f64);
    }
}

#[derive(Resource, Clone)]
pub struct CaveNetworkNoiseGenerator {
    noise: Arc<CaveNoise>,
}

type CaveNoise = Displace3d<GridNoise, StackedNoise>;

impl CaveNetworkNoiseGenerator {
    pub fn new(seed: u32, grid_size: f64, displacement_strength: f64) -> Self {
        let x = StackedNoise::new(seed.rotate_left(1), 4, 500.);
        let y = StackedNoise::new(seed.rotate_left(2), 4, 1000.);
        let z = StackedNoise::new(seed.rotate_left(3), 4, 500.);
        let source = GridNoise {
            size: grid_size,
            dropout_rate_y: 0.75,
            dropout_rate_xz: 0.5,
            permutation_table: PermutationTable::new(seed),
        };
        let displaced = Displace3d {
            source,
            displacement_strength,
            x,
            y,
            z,
        };
        let noise = Arc::new(displaced);
        Self { noise }
    }
}

impl NoiseFn<i32, 3> for CaveNetworkNoiseGenerator {
    fn get(&self, point: [i32; 3]) -> f64 {
        self.noise.get(point)
    }
}

struct Displace3d<Source, Displacer>
where
    Source: NoiseFn<f64, 3>,
    Displacer: NoiseFn<i32, 3>,
{
    source: Source,
    x: Displacer,
    y: Displacer,
    z: Displacer,
    displacement_strength: f64,
}

impl<Source, Displacer> NoiseFn<i32, 3> for Displace3d<Source, Displacer>
where
    Source: NoiseFn<f64, 3>,
    Displacer: NoiseFn<i32, 3>,
{
    fn get(&self, point: [i32; 3]) -> f64 {
        let dx = self.x.get(point) * self.displacement_strength;
        let dy = self.y.get(point) * self.displacement_strength;
        let dz = self.z.get(point) * self.displacement_strength;
        let [x0, y0, z0] = point.map(|x| x as f64);
        let new_point = [x0 + dx, y0 + dy, z0 + dz];
        return self.source.get(new_point);
    }
}

struct GridNoise {
    size: f64,
    dropout_rate_xz: f32,
    dropout_rate_y: f32,
    permutation_table: PermutationTable,
}

impl NoiseFn<f64, 3> for GridNoise {
    fn get(&self, point: [f64; 3]) -> f64 {
        let p = (DVec3::from(point) / self.size).fract_gl();
        let midline = DVec2::ONE * 0.5;

        let [x, y, z] = self
            .get_dropouts(DVec3::from(point) / self.size)
            .map(|drop_segment| if drop_segment { f64::INFINITY } else { 1. });

        let distances = [
            (p.xy() - midline).length() * z,
            (p.xz() - midline).length() * y,
            (p.yz() - midline).length() * x,
        ];
        let distance_to_midline = distances
            .iter()
            .min_by(|a, b| {
                a.partial_cmp(b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        return *distance_to_midline;
    }
}

impl GridNoise {
    fn get_dropouts<T>(&self, p: T) -> [bool; 3]
    where
        DVec3: From<T>,
    {
        let [x, y, z] = DVec3::from(p)
            .abs()
            .as_ivec3()
            .to_array();
        let seed = x ^ y.rotate_left(1) ^ z.rotate_left(2);
        let x_dropout = self
            .permutation_table
            .hash(&[seed as isize]) as f32
            / 255.;
        let y_dropout = self
            .permutation_table
            .hash(&[seed.rotate_left(1) as isize]) as f32
            / 255.;
        let z_dropout = self
            .permutation_table
            .hash(&[seed.rotate_left(2) as isize]) as f32
            / 255.;
        [
            x_dropout < self.dropout_rate_xz,
            y_dropout < self.dropout_rate_y,
            z_dropout < self.dropout_rate_xz,
        ]
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
