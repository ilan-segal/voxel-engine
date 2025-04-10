use bevy::prelude::*;
use itertools::Itertools;
use noise::{
    permutationtable::{NoiseHasher, PermutationTable},
    NoiseFn, Perlin,
};
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
            .hash(&[x as isize, y as isize, z as isize]) as u32;
        const MAXIMUM: u32 = 0xFF;
        return (result % MAXIMUM) as f64 / (MAXIMUM as f64);
    }
}

#[derive(Resource, Clone)]
pub struct CaveNetworkNoiseGenerator {
    white_noise: Arc<[WhiteNoise; 3]>,
    scale: f64,
}

impl CaveNetworkNoiseGenerator {
    pub fn new(seed: u32) -> Self {
        let white_noise = [
            WhiteNoise::new(seed),
            WhiteNoise::new(seed.rotate_left(1)),
            WhiteNoise::new(seed.rotate_left(2)),
        ];
        let scale = 100.0;
        Self {
            white_noise: Arc::new(white_noise),
            scale,
        }
    }

    fn pos_in_cell(&self, cell: [i32; 3]) -> [f64; 3] {
        let [x, y, z] = cell;
        [
            self.white_noise[0].get(cell) + x as f64,
            self.white_noise[1].get(cell) + y as f64,
            self.white_noise[2].get(cell) + z as f64,
        ]
    }

    fn points_in_neighborhood(
        &self,
        center_cell: [f64; 3],
    ) -> impl Iterator<Item = [f64; 3]> + use<'_> {
        let size = 3;
        let [x0, y0, z0] = center_cell;
        // let [x_offset, y_offset, z_offset] = center_cell
        //     .map(|x| x - x.floor())
        //     .map(|x| if x < 0.5 { -1 } else { 0 });
        let [x_offset, y_offset, z_offset] = [-1, -1, -1];
        (0..size * size * size)
            .map(move |i| {
                // Offset from center
                let x = i % size;
                let y = (i / size) % size;
                let z = (i / (size * size)) % size;
                (x + x_offset, y + y_offset, z + z_offset)
            })
            .map(move |(x, y, z)| {
                [
                    x0.floor() as i32 + x,
                    y0.floor() as i32 + y,
                    z0.floor() as i32 + z,
                ]
            })
            .map(|cell| self.pos_in_cell(cell))
    }

    fn distance_to_border(&self, point: [f64; 3]) -> f64 {
        let (a, b, c) = self
            .points_in_neighborhood(point)
            .map(|cur_point| euclidean_distance(&cur_point, &point))
            .sorted_by(|a, b| a.partial_cmp(b).unwrap())
            .take(3)
            .collect_tuple()
            .unwrap();
        let d = (a - b).powi(2);
        let e = (a - c).powi(2);
        let f = (b - c).powi(2);
        return (d + e + f).sqrt();
    }
}

fn euclidean_distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let [xa, ya, za] = a;
    let [xb, yb, zb] = b;
    let x = (xa - xb).powi(2);
    let y = (ya - yb).powi(2);
    let z = (za - zb).powi(2);
    return (x + y + z).sqrt();
}

impl NoiseFn<i32, 3> for CaveNetworkNoiseGenerator {
    fn get(&self, [x, y, z]: [i32; 3]) -> f64 {
        // 3-dimensional Worley noise
        // The noise crate has an implementation but it's !Send so we need to implement our own
        let x = (x as f64) / self.scale;
        let y = (y as f64) / self.scale;
        let z = (z as f64) / self.scale;
        self.distance_to_border([x, y, z])
    }
}

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
