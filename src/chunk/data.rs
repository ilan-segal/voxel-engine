use super::{spatial::SpatiallyMapped, CHUNK_LENGTH, CHUNK_SIZE_I32};
use crate::{block::Block, define_spatial, map_from_noise_2d, map_from_noise_3d};
use bevy::prelude::*;
use noise::NoiseFn;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

define_spatial!(Blocks, 3, Block);

impl Blocks {
    pub fn is_meshable(&self) -> bool {
        self.0.par_iter().any(|b| b.is_meshable())
    }
}

impl Default for Blocks {
    fn default() -> Self {
        let blocks = vec![default(); CHUNK_LENGTH];
        return Self(blocks);
    }
}

define_spatial!(Noise3d, 3, f32);
map_from_noise_3d!(Noise3d);
define_spatial!(ContinentNoise, 2, f32);
map_from_noise_2d!(ContinentNoise);
define_spatial!(HeightNoise, 2, f32);
map_from_noise_2d!(HeightNoise);
