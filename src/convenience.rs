
use crate::{
    HeteroSizedVec,
    pushable::InPlace,
};

use std::{
    ops::{
        Index,
        IndexMut,
        Range,
    },
    hint::unreachable_unchecked,
    marker::Unsize,
    fmt::{
        self,
        Debug,
        Formatter,
    },
};

impl<T: ?Sized> HeteroSizedVec<T> {
    /// Push some value which unsizes to the element type.
    pub fn push_value<E: Unsize<T>>(&mut self, elem: E) {
        self.push(InPlace(elem));
    }
}

// index operator

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

// iterators

unsafe fn unwrap_unchecked<T>(option: Option<T>) -> T {
    match option {
        Some(elem) => elem,
        None => unreachable_unchecked(),
    }
}

unsafe fn change_lifetime_mut<'a, 'b, T: ?Sized>(r: &'a mut T) -> &'b mut T {
    &mut *(r as *mut T)
}

pub struct Iter<'a, T: ?Sized> {
    vec: &'a HeteroSizedVec<T>,
    index: Range<usize>,
}

impl<'a, T: ?Sized> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.index.next()
            .map(|i| unsafe {
                unwrap_unchecked(self.vec.get(i))
            })
    }
}

pub struct IterMut<'a, T: ?Sized> {
    vec: &'a mut HeteroSizedVec<T>,
    index: Range<usize>,
}

impl<'a, T: ?Sized> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.index.next()
            .map(|i| unsafe {
                change_lifetime_mut(unwrap_unchecked(
                    self.vec.get_mut(i)
                ))
            })
    }
}

impl<'a, T: ?Sized> Iter<'a, T> {
    pub fn new(vec: &'a HeteroSizedVec<T>) -> Self {
        Iter {
            index: 0..vec.len(),
            vec,
        }
    }
}

impl<'a, T: ?Sized> IterMut<'a, T> {
    pub fn new(vec: &'a mut HeteroSizedVec<T>) -> Self {
        IterMut {
            index: 0..vec.len(),
            vec,
        }
    }
}


impl<T: ?Sized> HeteroSizedVec<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut::new(self)
    }
}

impl<'a, T: ?Sized> IntoIterator for &'a HeteroSizedVec<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: ?Sized> IntoIterator for &'a mut HeteroSizedVec<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

// debug

impl<T: ?Sized> Debug for HeteroSizedVec<T>
where
    for<'a> &'a T: Debug
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self)
            .finish()
    }
}