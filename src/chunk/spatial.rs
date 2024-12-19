use super::CHUNK_SIZE;

pub trait SpatiallyMapped<T, const DIM: usize> {
    fn at_pos(&self, pos: [usize; DIM]) -> &T;
    fn at_pos_mut(&mut self, pos: [usize; DIM]) -> &mut T;
}

impl<T> SpatiallyMapped<T, 2> for Vec<T> {
    fn at_pos(&self, [x, y]: [usize; 2]) -> &T {
        self.get(CHUNK_SIZE * x + y)
            .expect("Index range")
    }

    fn at_pos_mut(&mut self, [x, y]: [usize; 2]) -> &mut T {
        self.get_mut(CHUNK_SIZE * x + y)
            .expect("Index range")
    }
}

impl<T> SpatiallyMapped<T, 3> for Vec<T> {
    fn at_pos(&self, [x, y, z]: [usize; 3]) -> &T {
        self.get(CHUNK_SIZE * CHUNK_SIZE * x + CHUNK_SIZE * z + y)
            .expect("Index range")
    }

    fn at_pos_mut(&mut self, [x, y, z]: [usize; 3]) -> &mut T {
        self.get_mut(CHUNK_SIZE * CHUNK_SIZE * x + CHUNK_SIZE * z + y)
            .expect("Index range")
    }
}
