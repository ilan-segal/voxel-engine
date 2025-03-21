use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::CHUNK_SIZE;

pub trait SpatiallyMapped<const DIM: usize> {
    type Item;

    fn at_pos(&self, pos: [usize; DIM]) -> &Self::Item;
    fn at_pos_mut(&mut self, pos: [usize; DIM]) -> &mut Self::Item;
    fn from_fn<F: Sync + Fn([usize; DIM]) -> Self::Item>(f: F) -> Self;
}

impl<T: Send> SpatiallyMapped<2> for Vec<T> {
    type Item = T;
    fn at_pos(&self, [x, y]: [usize; 2]) -> &T {
        self.get(coords_to_index_2d(x, y)).expect("Index range")
    }

    fn at_pos_mut(&mut self, [x, y]: [usize; 2]) -> &mut T {
        self.get_mut(coords_to_index_2d(x, y)).expect("Index range")
    }

    fn from_fn<F: Sync + Fn([usize; 2]) -> Self::Item>(f: F) -> Self {
        (0..CHUNK_SIZE * CHUNK_SIZE)
            .into_par_iter()
            .map(|i| index_to_coords_2d(i))
            .map(|coords| f(coords))
            .collect::<_>()
    }
}

fn coords_to_index_2d(x: usize, y: usize) -> usize {
    CHUNK_SIZE * x + y
}

fn index_to_coords_2d(i: usize) -> [usize; 2] {
    [i.div_floor(CHUNK_SIZE), i % CHUNK_SIZE]
}

impl<T: Send> SpatiallyMapped<3> for Vec<T> {
    type Item = T;
    fn at_pos(&self, [x, y, z]: [usize; 3]) -> &T {
        self.get(coords_to_index_3d(x, y, z)).expect("Index range")
    }

    fn at_pos_mut(&mut self, [x, y, z]: [usize; 3]) -> &mut T {
        self.get_mut(coords_to_index_3d(x, y, z))
            .expect("Index range")
    }

    fn from_fn<F: Sync + Fn([usize; 3]) -> Self::Item>(f: F) -> Self {
        (0..CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE)
            .into_par_iter()
            .map(|i| index_to_coords_3d(i))
            .map(|coords| f(coords))
            .collect::<_>()
    }
}

fn coords_to_index_3d(x: usize, y: usize, z: usize) -> usize {
    CHUNK_SIZE * CHUNK_SIZE * x + CHUNK_SIZE * z + y
}

fn index_to_coords_3d(i: usize) -> [usize; 3] {
    [
        i.div_floor(CHUNK_SIZE * CHUNK_SIZE),
        i % CHUNK_SIZE,
        i.div_floor(CHUNK_SIZE) % CHUNK_SIZE,
    ]
}

#[macro_export]
macro_rules! define_spatial {
    ($name:ident, $dim:literal, $t:ty) => {
        #[derive(Component, Clone, Debug)]
        pub struct $name(pub Vec<$t>);

        impl SpatiallyMapped<$dim> for $name {
            type Item = $t;

            fn at_pos(&self, pos: [usize; $dim]) -> &Self::Item {
                self.0.at_pos(pos)
            }

            fn at_pos_mut(&mut self, pos: [usize; $dim]) -> &mut Self::Item {
                self.0.at_pos_mut(pos)
            }

            fn from_fn<F: Sync + Fn([usize; $dim]) -> Self::Item>(f: F) -> Self {
                Self(SpatiallyMapped::from_fn(f))
            }
        }
    };
}

#[macro_export]
macro_rules! map_from_noise_2d {
    ($name:ident) => {
        impl<T: NoiseFn<i32, 2> + Sync> From<(T, IVec3)> for $name {
            fn from((noise, chunk_pos): (T, IVec3)) -> Self {
                Self::from_fn(|[x, y]| {
                    noise.get([
                        (x as i32 + chunk_pos.x * CHUNK_SIZE_I32),
                        (y as i32 + chunk_pos.z * CHUNK_SIZE_I32),
                    ]) as f32
                })
            }
        }
    };
}

#[macro_export]
macro_rules! map_from_noise_3d {
    ($name:ident) => {
        impl<T: NoiseFn<i32, 3> + Sync> From<(T, IVec3)> for $name {
            fn from((noise, chunk_pos): (T, IVec3)) -> Self {
                Self::from_fn(|[x, y, z]| {
                    noise.get([
                        (x as i32 + chunk_pos.x * CHUNK_SIZE_I32),
                        (y as i32 + chunk_pos.y * CHUNK_SIZE_I32),
                        (z as i32 + chunk_pos.z * CHUNK_SIZE_I32),
                    ]) as f32
                })
            }
        }
    };
}
