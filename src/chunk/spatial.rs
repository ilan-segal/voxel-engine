use super::CHUNK_SIZE;

pub trait SpatiallyMapped<const DIM: usize> {
    type Item;

    fn at_pos(&self, pos: [usize; DIM]) -> &Self::Item;
    fn at_pos_mut(&mut self, pos: [usize; DIM]) -> &mut Self::Item;
}

impl<T> SpatiallyMapped<2> for Vec<T> {
    type Item = T;
    fn at_pos(&self, [x, y]: [usize; 2]) -> &T {
        self.get(coords_to_index_2d(x, y))
            .expect("Index range")
    }

    fn at_pos_mut(&mut self, [x, y]: [usize; 2]) -> &mut T {
        self.get_mut(coords_to_index_2d(x, y))
            .expect("Index range")
    }
}

fn coords_to_index_2d(x: usize, y: usize) -> usize {
    CHUNK_SIZE * x + y
}

impl<T> SpatiallyMapped<3> for Vec<T> {
    type Item = T;
    fn at_pos(&self, [x, y, z]: [usize; 3]) -> &T {
        self.get(coords_to_index_3d(x, y, z))
            .expect("Index range")
    }

    fn at_pos_mut(&mut self, [x, y, z]: [usize; 3]) -> &mut T {
        self.get_mut(coords_to_index_3d(x, y, z))
            .expect("Index range")
    }
}

fn coords_to_index_3d(x: usize, y: usize, z: usize) -> usize {
    CHUNK_SIZE * CHUNK_SIZE * x + CHUNK_SIZE * z + y
}
