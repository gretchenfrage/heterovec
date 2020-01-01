#![feature(unsize)]
#![feature(arbitrary_self_types)]
#![feature(coerce_unsized)]
#![feature(new_uninit)]
#![feature(slice_from_raw_parts)]
#![allow(unused_parens)]

use std::marker::PhantomData;

/// `HeteroSizedPush` and its implementations.
mod pushable;

/// Very unsafe memory management.
mod memory;

/// Ease of use functions and implementations.
mod convenience;

#[cfg(test)]
pub mod tests;

// public-facing re-exports

#[doc(inline)]
pub use self::pushable::{
    HeteroSizedPush,
    InPlace,
};
/// Iterators.
pub mod iter {
    #[doc(inline)]
    pub use crate::convenience::{
        Iter,
        IterMut,
    };
}

/// Dense vector of an unsized type.
///
/// This collection supports few operations. Important ones are:
///
/// - Pushing an element
/// - Indexing
/// - Conversion into a `Vec` of boxes
///
/// This supports elements such as trait objects, `str`, and `[T]`.
pub struct HeteroSizedVec<T: ?Sized> {
    // densely packed elements
    // respects alignment rules
    storage: Vec<u8>,
    // fat-pointer metadata for each element
    ptr_meta: Vec<usize>,
    // start-indices of each element within storage
    mem_indices: Vec<usize>,
    // handlers for dropping each element
    //
    // the given pointer is to the start address of the element
    // and the second element is fat pointer metadata
    // unless the pointers are not fat, in which case it will be zero
    drop_handlers: Vec<fn(*mut u8, usize)>,
    // the runtime size of each element
    // this is used for moving them to the heap
    elems_size: Vec<usize>,

    p: PhantomData<T>,
}

impl<T: ?Sized> HeteroSizedVec<T> {
    /// New, empty vector.
    pub fn new() -> Self {
        HeteroSizedVec {
            storage: Vec::new(),
            ptr_meta: Vec::new(),
            mem_indices: Vec::new(),
            drop_handlers: Vec::new(),
            elems_size: Vec::new(),

            p: PhantomData,
        }
    }

    /// Length in elements.
    pub fn len(&self) -> usize {
        self.mem_indices.len()
    }
}

