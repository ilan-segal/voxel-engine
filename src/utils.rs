use itertools::{Itertools, Product};
use std::{iter::Step, ops::Range};

pub struct VolumetricRange<T: Step> {
    product: Product<Product<Range<T>, Range<T>>, Range<T>>,
}

impl<T: Step> VolumetricRange<T> {
    pub fn new(x_range: Range<T>, y_range: Range<T>, z_range: Range<T>) -> Self {
        let product = x_range
            .cartesian_product(y_range)
            .cartesian_product(z_range);
        Self { product }
    }
}

impl<T: Step> Iterator for VolumetricRange<T> {
    type Item = (T, T, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.product
            .next()
            .map(|((x, y), z)| (x, y, z))
    }
}

pub fn fast_hash_i32(a: i32) -> u32 {
    fast_hash(u32::from_be_bytes(a.to_be_bytes()))
}

pub fn fast_hash_f32(a: f32) -> u32 {
    fast_hash(u32::from_be_bytes(a.to_be_bytes()))
}

pub fn fast_hash(a: u32) -> u32 {
    const START: u32 = 0x27d4eb2d;
    let mut a = a;
    a ^= START;
    a ^= a.rotate_left(21);
    a ^= a.rotate_right(35);
    a ^= a.rotate_left(4);
    return a;
}
