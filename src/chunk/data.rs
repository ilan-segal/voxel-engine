use super::{spatial::SpatiallyMapped, CHUNK_LENGTH};
use crate::{block::Block, define_spatial};
use bevy::prelude::*;
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

define_spatial!(Perlin2d, 2, f32);
define_spatial!(Noise3d, 3, f32);
define_spatial!(ContinentNoise, 2, f32);
