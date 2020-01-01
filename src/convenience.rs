
use crate::HeteroSizedVec;

use std::ops::{
    Index,
    IndexMut,
};

impl<T: ?Sized> Index<usize> for HeteroSizedVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.get(index).unwrap()
    }
}

impl<T: ?Sized> IndexMut<usize> for HeteroSizedVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.get_mut(index).unwrap()
    }
}

