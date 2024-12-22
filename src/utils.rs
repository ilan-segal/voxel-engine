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
