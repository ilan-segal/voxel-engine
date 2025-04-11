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
        let noise = StackedNoise::new(seed, 6, 1000.0);
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
            .hash(&[x as isize, y as isize, z as isize]) as u8;
        const MAXIMUM: u8 = 0xFF;
        return (result % MAXIMUM) as f64 / (MAXIMUM as f64);
    }
}

#[derive(Resource, Clone)]
pub struct CaveNetworkNoiseGenerator {
    worley: Arc<Worley>,
    displacement: Arc<[StackedNoise; 3]>,
    displacement_strength: f64,
}

impl CaveNetworkNoiseGenerator {
    pub fn new(seed: u32) -> Self {
        let worley = Arc::new(Worley::new(seed));
        let displacement_x = StackedNoise::new(seed.wrapping_add(1), 3, 100.);
        let displacement_y = StackedNoise::new(seed.wrapping_add(2), 3, 10000.);
        let displacement_z = StackedNoise::new(seed.wrapping_add(3), 3, 100.);
        let displacement = Arc::new([displacement_x, displacement_y, displacement_z]);
        let displacement_strength = 300.0;
        Self {
            worley,
            displacement,
            displacement_strength,
        }
    }
}

impl NoiseFn<i32, 3> for CaveNetworkNoiseGenerator {
    fn get(&self, point: [i32; 3]) -> f64 {
        let x = point[0] as f64 + self.displacement[0].get(point) * self.displacement_strength;
        let y = point[1] as f64 + self.displacement[1].get(point) * self.displacement_strength;
        let z = point[2] as f64 + self.displacement[2].get(point) * self.displacement_strength;
        // let point = point.map(|x| x as f64);
        self.worley.get([x, y, z])
    }
}

#[derive(Clone)]
struct Worley {
    white_noise: Arc<[WhiteNoise; 3]>,
    scale: f32,
}

impl Worley {
    fn new(seed: u32) -> Self {
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

    fn pos_in_cell(&self, cell: IVec3) -> Vec3 {
        let arr = cell.to_array();
        let Vec3 { x, y, z } = cell.as_vec3();
        [
            self.white_noise[0].get(arr) as f32 + x,
            self.white_noise[1].get(arr) as f32 + y,
            self.white_noise[2].get(arr) as f32 + z,
        ]
        .into()
    }

    fn points_in_neighborhood(&self, center_cell: Vec3) -> impl Iterator<Item = Vec3> + use<'_> {
        let size = 3;
        (0..size * size * size)
            .map(move |i| {
                // Offset from center
                let x = i % size;
                let y = (i / size) % size;
                let z = (i / (size * size)) % size;
                IVec3 { x, y, z }
            })
            .map(|p| p - IVec3::ONE)
            .map(move |p| center_cell.floor().as_ivec3() + p)
            .map(|cell| self.pos_in_cell(cell))
    }

    fn distance_to_border(&self, point: Vec3) -> f32 {
        /*
        TODO
        This is a decent approximation but it's not perfect
        We want a perfect measure of the distance to the edges
        Use this for guidance https://www.ronja-tutorials.com/post/028-voronoi-noise/#getting-the-distance-to-the-border
         */
        let (a, b, c) = self
            .points_in_neighborhood(point)
            .map(|cur_point| distance(cur_point, point))
            .sorted_by(|a, b| a.partial_cmp(b).unwrap())
            .take(3)
            .collect_tuple()
            .unwrap();
        let d = ((a - b) / c).powi(2);
        let e = ((a - c) / b).powi(2);
        let f = ((b - a) / a).powi(2);
        return (d + e + f).sqrt();
    }
}

fn distance(a: Vec3, b: Vec3) -> f32 {
    (a - b).length()
}

impl NoiseFn<f64, 3> for Worley {
    fn get(&self, [x, y, z]: [f64; 3]) -> f64 {
        // 3-dimensional Worley noise
        // The noise crate has an implementation but it's !Send so we need to implement our own
        let x = x as f32 / self.scale;
        let y = y as f32 / self.scale;
        let z = z as f32 / self.scale;
        self.distance_to_border([x, y, z].into()) as f64
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

impl StackedNoise {
    fn new(seed: u32, num_layers: u32, starting_scale: f64) -> Self {
        let noises = (0..num_layers)
            .map(|i| {
                let amplitude = 0.5_f64.powi(i as i32);
                let scale = starting_scale * amplitude;
                let offset = starting_scale * i as f64;
                let perlin = Perlin::new(seed.rotate_left(i));
                return NoiseGenerator {
                    perlin,
                    scale,
                    amplitude,
                    offset,
                };
            })
            .collect();
        Self(noises)
    }
}

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
