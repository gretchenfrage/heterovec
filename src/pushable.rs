
use std::{
    mem::{
        size_of,
        align_of,
        transmute,
        transmute_copy,
        ManuallyDrop,
    },
    ptr::{
        drop_in_place,
        slice_from_raw_parts_mut,
    },
    marker::Unsize,
};

/// Types that can be pushed onto a `HeteroSizedVec`.
///
/// Not meant to be implemented outside of the `heterovec` crate.
pub unsafe trait HeteroSizedPush<T: ?Sized> {
    unsafe fn elem_size(&self) -> usize;

    unsafe fn elem_align(&self) -> usize;

    unsafe fn elem_ptr(&self) -> *const T;

    unsafe fn elem_drop_handler(&self) -> fn(*mut u8, usize);

    /// Assume that ownership of the pointee has been taken through unsafe
    /// means, but if there is some destructable wrapper around that
    /// (eg. a `Box`), clean that up, but without dropping the inner element.
    unsafe fn outer_drop(&mut self);
}

/// Used to directly push an element onto a `HeteroSizedVec` from the stack.
pub struct InPlace<E>(pub E);

unsafe impl<T: ?Sized, E: Unsize<T>> HeteroSizedPush<T> for InPlace<E> {
    unsafe fn elem_size(&self) -> usize {
        size_of::<E>()
    }

    unsafe fn elem_align(&self) -> usize {
        align_of::<E>()
    }

    unsafe fn elem_ptr(&self) -> *const T {
        &self.0 as &T as *const T
    }

    unsafe fn elem_drop_handler(&self) -> fn(*mut u8, usize) {
        |data, _| {
            drop_in_place(transmute::<
                *mut u8,
                &mut E,
            >(data));
        }
    }

    unsafe fn outer_drop(&mut self) {}
}

unsafe impl<'a, I: Copy> HeteroSizedPush<[I]> for &'a [I] {
    unsafe fn elem_size(&self) -> usize {
        self.len() * size_of::<I>()
    }

    unsafe fn elem_align(&self) -> usize {
        align_of::<I>()
    }

    unsafe fn elem_ptr(&self) -> *const [I] {
        *self as *const [I]
    }

    unsafe fn elem_drop_handler(&self) -> fn(*mut u8, usize) {
        |_, _| () // we are copy, so no need to drop
    }

    unsafe fn outer_drop(&mut self) {}
}

unsafe impl<I> HeteroSizedPush<[I]> for Vec<I> {
    unsafe fn elem_size(&self) -> usize {
        self.len() * size_of::<I>()
    }

    unsafe fn elem_align(&self) -> usize {
        align_of::<I>()
    }

    unsafe fn elem_ptr(&self) -> *const [I] {
        self.as_slice() as *const [I]
    }

    unsafe fn elem_drop_handler(&self) -> fn(*mut u8, usize) {
        // this relies on the fat pointer meta in an array slice being the length in elements
        |start, len| unsafe {
            // drop each element
            let elems: &mut [I] = &mut *slice_from_raw_parts_mut(
                start as *mut I,
                len,
            );
            for elem in elems {
                drop_in_place(elem);
            }
        }
    }

    unsafe fn outer_drop(&mut self) {
        // this is similar to how we are handling the box

        drop(transmute_copy::<
            Vec<I>,
            Vec<ManuallyDrop<I>>,
        >);
    }
}

unsafe impl<'a> HeteroSizedPush<str> for &'a str {
    unsafe fn elem_size(&self) -> usize {
        (*self).len()
    }

    unsafe fn elem_align(&self) -> usize {
        1
    }

    unsafe fn elem_ptr(&self) -> *const str {
        *self as *const str
    }

    unsafe fn elem_drop_handler(&self) -> fn(*mut u8, usize) {
        |_, _| () // no destructor needed for str
    }

    unsafe fn outer_drop(&mut self) {}
}

unsafe impl<T: ?Sized> HeteroSizedPush<T> for Box<dyn HeteroSizedPush<T>> {
    unsafe fn elem_size(&self) -> usize {
        Box::as_ref(self).elem_size()
    }

    unsafe fn elem_align(&self) -> usize {
        Box::as_ref(self).elem_align()
    }

    unsafe fn elem_ptr(&self) -> *const T {
        Box::as_ref(self).elem_ptr()
    }

    unsafe fn elem_drop_handler(&self) -> fn(*mut u8, usize) {
        Box::as_ref(self).elem_drop_handler()
    }

    unsafe fn outer_drop(&mut self) {
        // this feels deeply disturbing

        drop(transmute_copy::<
            Box<dyn HeteroSizedPush<T>>,
            Box<ManuallyDrop<dyn HeteroSizedPush<T>>>,
        >(self));
    }
}