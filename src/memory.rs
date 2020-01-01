
use crate::{
    HeteroSizedVec,
    pushable::HeteroSizedPush,
};

use std::{
    mem::{
        size_of,
        transmute_copy,
        ManuallyDrop,
        MaybeUninit,
    },
    ptr,
};

/// Size of a thin pointer.
const THIN_PTR_SIZE: usize = size_of::<usize>();

/// Size of a fat pointer.
const FAT_PTR_SIZE: usize = size_of::<usize>() * 2;

/// Whether a pointer to a type is a fat pointer.
fn pointer_is_fat<T: ?Sized>() -> bool {
    match size_of::<&T>() {
        FAT_PTR_SIZE => true,
        THIN_PTR_SIZE => false,
        _ => panic!("unexpected pointer size: {}", size_of::<&T>())
    }
}

/// Decomposed memory representation of a fat pointer.
#[repr(C)]
#[derive(Copy, Clone)]
struct FatPtr<DataPtr> {
    data: DataPtr,
    meta: usize,
}

type FatPtrConst = FatPtr<*const u8>;
type FatPtrMut = FatPtr<*mut u8>;

impl FatPtrConst {
    #[inline(always)]
    unsafe fn transmute_to_ref<'a, T: ?Sized>(self) -> &'a T {
        transmute_copy::<FatPtrConst, &T>(&self)
    }

    #[inline(always)]
    unsafe fn transmute_from_ptr<T: ?Sized>(p: *const T) -> Self {
        transmute_copy::<*const T, FatPtrConst>(&p)
    }
}

impl FatPtrMut {
    #[inline(always)]
    unsafe fn transmute_to_mut<'a, T: ?Sized>(self) -> &'a mut T {
        transmute_copy::<FatPtrMut, &mut T>(&self)
    }

    #[inline(always)]
    unsafe fn transmute_to_box<T: ?Sized>(self) -> Box<T> {
        transmute_copy::<FatPtrMut, Box<T>>(&self)
    }
}

impl<T: ?Sized> HeteroSizedVec<T> {
    /// Push an element onto the vector.
    pub fn push<E: HeteroSizedPush<T>>(&mut self, elem: E) {
        unsafe {
            // prevent double-free in panic
            let mut elem = ManuallyDrop::new(elem);

            let elem_size:  usize  = elem.elem_size();
            let elem_align: usize  = elem.elem_align();
            let elem_ptr: *const T = elem.elem_ptr();
            let elem_drop_handler: fn(*mut u8, usize) = elem.elem_drop_handler();

            // push the fat pointer meta,
            // handle the case that the pointer isn't actually fat
            let elem_data_ptr: *const u8 = match pointer_is_fat::<T>() {
                true => {
                    let parts = FatPtrConst::transmute_from_ptr::<T>(elem_ptr);
                    self.ptr_meta.push(parts.meta);
                    parts.data
                },
                false => transmute_copy::<*const T, *const u8>(&elem_ptr),
            };

            // determine the start position in the elements storage
            // and the amount of padding bytes to place before-hand
            let mut to_reserve: usize = elem_size;
            let mut offset: usize = self.storage.len();

            if self.storage.len() % elem_align != 0 {
                let padding: usize = (
                    elem_align - (self.storage.len() % elem_align)
                );

                offset += padding;
                to_reserve += padding;
            }

            // add mem index and elem len
            self.mem_indices.push(offset);
            self.elems_size.push(elem_size);

            // write element to storage memory
            // 1. this is really unsafe
            // 2. it leaves the padding bytes as uninitialized
            // 3. but i don't think that will actually trigged UB
            // 4. because of... vector implementation details... :/
            self.storage.reserve(to_reserve);
            ptr::copy_nonoverlapping::<u8>(
                // src:
                elem_data_ptr,
                // dst:
                self.storage.as_mut_ptr().offset(offset as isize),
                // len:
                elem_size,
            );
            self.storage.set_len(
                self.storage.len() + to_reserve
            );

            // once we've finished reading from the elem, we can call
            // `outer_drop` on the wrapper, which will invalidate
            // `elem_data_ptr`.
            elem.outer_drop();

            // add drop handler, now that all other state is properly created
            self.drop_handlers.push(elem_drop_handler);
        }
    }

    /// Get element by index as reference.
    pub fn get(&self, index: usize) -> Option<&T> {
        unsafe {
            let offset: usize = match self.mem_indices.get(index) {
                Some(&offset) => offset,
                None => return None,
            };
            let raw_ptr: *const u8 = self.storage.as_ptr()
                .offset(offset as isize);

            if pointer_is_fat::<T>() {
                Some(FatPtrConst {
                    data: raw_ptr,
                    meta: *self.ptr_meta.get_unchecked(index),
                }.transmute_to_ref::<T>())
            } else {
                Some(transmute_copy::<*const u8, &T>(&raw_ptr))
            }
        }
    }

    /// Get element by index as mutable reference.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        unsafe {
            let offset: usize = match self.mem_indices.get(index) {
                Some(&offset) => offset,
                None => return None,
            };
            let raw_ptr: *mut u8 = self.storage.as_mut_ptr()
                .offset(offset as isize);

            if pointer_is_fat::<T>() {
                Some(FatPtrMut {
                    data: raw_ptr,
                    meta: *self.ptr_meta.get_unchecked(index),
                }.transmute_to_mut::<T>())
            } else {
                Some(transmute_copy::<*mut u8, &mut T>(&raw_ptr))
            }
        }
    }

    /// Convert into a vector of boxes.
    pub fn into_boxed_vec(self) -> Vec<Box<T>> {
        unsafe {
            if pointer_is_fat::<T>() {
                self.into_boxed_vec_fat()
            } else {
                self.into_boxed_vec_thin()
            }
        }
    }

    // `into_boxed_vec` case where `&T` is a fat pointer.
    unsafe fn into_boxed_vec_fat(mut self) -> Vec<Box<T>> {
        // clear the drop handlers now to prevent double-free in panic
        self.drop_handlers.clear();

        let mut boxed_vec: Vec<Box<T>> =
            Vec::with_capacity(self.mem_indices.len());

        // iterate through elements
        for (offset, (size, meta)) in Iterator::zip(
            self.mem_indices.iter().copied(),
            Iterator::zip(
                self.elems_size.iter().copied(),
                self.ptr_meta.iter().copied(),
            ),
        ) {

            // make the heap allocation
            let heap_alloc: Box<[MaybeUninit<u8>]> =
                Box::new_uninit_slice(size);

            // lower it into a raw heap data pointer
            let heap_ptr: *mut u8 =
                Box::into_raw(heap_alloc) as *mut u8;

            // copy the element onto the heap
            ptr::copy_nonoverlapping::<u8>(
                // src:
                self.storage.as_mut_ptr().offset(offset as isize),
                // dst:
                heap_ptr,
                // len:
                size,
            );

            // produce the fat heap pointer as a box
            let fat_box: Box<T> = FatPtrMut {
                data: heap_ptr,
                meta,
            }.transmute_to_box::<T>();

            // push to the vec
            boxed_vec.push(fat_box);
        }

        // done
        boxed_vec
    }

    // `into_boxed_vec` case where `&T` is a thin pointer.
    unsafe fn into_boxed_vec_thin(mut self) -> Vec<Box<T>> {
        // clear the drop handlers now to prevent double-free in panic
        self.drop_handlers.clear();

        let mut boxed_vec: Vec<Box<T>> =
            Vec::with_capacity(self.mem_indices.len());

        // iterate through elements
        for (offset, size) in Iterator::zip(
            self.mem_indices.iter().copied(),
            self.elems_size.iter().copied(),
        ) {
            // make the heap allocation
            let heap_alloc: Box<[MaybeUninit<u8>]> =
                Box::new_uninit_slice(size);

            // lower it into a raw heap data pointer
            let heap_ptr: *mut u8 =
                Box::into_raw(heap_alloc) as *mut u8;

            // copy the element onto the heap
            ptr::copy_nonoverlapping::<u8>(
                // src:
                self.storage.as_mut_ptr().offset(offset as isize),
                // dst:
                heap_ptr,
                // len:
                size,
            );

            // make the heap data pointer typed
            let thin_box: Box<T> = transmute_copy::<*mut u8, Box<T>>(&heap_ptr);

            // push to the vec
            boxed_vec.push(thin_box);
        }

        boxed_vec
    }
}

impl<T: ?Sized> Drop for HeteroSizedVec<T> {
    fn drop(&mut self) {
        // drop elements
        unsafe {
            if pointer_is_fat::<T>() {
                for (offset, (destructor, meta)) in Iterator::zip(
                    self.mem_indices.iter().copied(),
                    Iterator::zip(
                        self.drop_handlers.iter().copied(),
                        self.ptr_meta.iter().copied(),
                    ),
                ) {
                    let ptr: *mut u8 = self.storage.as_mut_ptr()
                        .offset(offset as isize);

                    destructor(ptr, meta);
                }
            } else {
                for (offset, destructor) in Iterator::zip(
                    self.mem_indices.iter().copied(),
                    self.drop_handlers.iter().copied(),
                ) {
                    let ptr: *mut u8 = self.storage.as_mut_ptr()
                        .offset(offset as isize);

                    destructor(ptr, 0);
                }
            }
        }
    }
}