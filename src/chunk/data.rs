use super::{spatial::SpatiallyMapped, CHUNK_LENGTH, CHUNK_SIZE_I32};
use crate::{block::Block, define_spatial};
use bevy::prelude::*;
use noise::NoiseFn;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

define_spatial!(Terrain, 3, Block);
define_spatial!(Blocks, 3, Block);

impl Blocks {
    pub fn is_meshable(&self) -> bool {
        self.0
            .par_iter()
            .any(|b| b.is_meshable())
    }
}

impl Default for Blocks {
    fn default() -> Self {
        let blocks = vec![default(); CHUNK_LENGTH];
        return Self(blocks);
    }
}

define_spatial!(Noise3d, 3, f32);
define_spatial!(ContinentNoise, 2, f32);
define_spatial!(HeightNoise, 2, f32);
define_spatial!(CaveNetworkNoise, 3, f32);
define_spatial!(TemperatureNoise, 2, f32);
define_spatial!(HumidityNoise, 2, f32);

pub trait FromNoise<const DIM: usize> {
    fn from_noise<Noise>(noise: Noise, chunk_pos: IVec3) -> Self
    where
        Noise: NoiseFn<f64, DIM> + Sync;
}

impl<Data> FromNoise<2> for Data
where
    Data: SpatiallyMapped<2, Item = f32>,
{
    fn from_noise<Noise>(noise: Noise, chunk_pos: IVec3) -> Self
    where
        Noise: NoiseFn<f64, 2> + Sync,
    {
        Self::from_fn(|[x, y]| {
            noise.get([
                (x as f64 + f64::from(chunk_pos.x * CHUNK_SIZE_I32)),
                (y as f64 + f64::from(chunk_pos.z * CHUNK_SIZE_I32)),
            ]) as f32
        })
    }
}

impl<Data> FromNoise<3> for Data
where
    Data: SpatiallyMapped<3, Item = f32>,
{
    fn from_noise<Noise>(noise: Noise, chunk_pos: IVec3) -> Self
    where
        Noise: NoiseFn<f64, 3> + Sync,
    {
        Self::from_fn(|[x, y, z]| {
            noise.get([
                (x as f64 + f64::from(chunk_pos.x * CHUNK_SIZE_I32)),
                (y as f64 + f64::from(chunk_pos.y * CHUNK_SIZE_I32)),
                (z as f64 + f64::from(chunk_pos.z * CHUNK_SIZE_I32)),
            ]) as f32
        })
    }
}
