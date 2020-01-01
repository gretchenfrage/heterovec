
use crate::HeteroSizedVec;

#[test]
fn closure_basic() {
    #[inline(never)]
    fn closure(n: usize) -> impl Fn() -> usize {
        move || n
    }

    let mut vec: HeteroSizedVec<dyn Fn() -> usize> = HeteroSizedVec::new();
    for n in 0..10 {
        vec.push_value(closure(n));
    }
    for (i, func) in vec.iter().enumerate() {
        assert_eq!(i, func());
    }
}

#[test]
fn array_basic() {
    let mut vec: HeteroSizedVec<[usize]> = HeteroSizedVec::new();

    vec.push_value([0]);
    vec.push_value([1, 2]);
    vec.push_value([3, 4, 5]);
    vec.push_value([6, 7, 8, 9]);
    vec.push_value([10, 11, 12, 13, 14]);

    assert_eq!(&vec[0], &[0]);
    assert_eq!(&vec[1], &[1, 2]);
    assert_eq!(&vec[2], &[3, 4, 5]);
    assert_eq!(&vec[3], &[6, 7, 8, 9]);
    assert_eq!(&vec[4], &[10, 11, 12, 13, 14]);
}

#[test]
fn array_copy_slice_basic() {
    let mut vec: HeteroSizedVec<[usize]> = HeteroSizedVec::new();

    vec.push(&[0_usize] as &[_]);
    vec.push(&[1_usize, 2] as &[_]);
    vec.push(&[3_usize, 4, 5] as &[_]);
    vec.push(&[6_usize, 7, 8, 9] as &[_]);
    vec.push(&[10_usize, 11, 12, 13, 14] as &[_]);

    assert_eq!(&vec[0], &[0]);
    assert_eq!(&vec[1], &[1, 2]);
    assert_eq!(&vec[2], &[3, 4, 5]);
    assert_eq!(&vec[3], &[6, 7, 8, 9]);
    assert_eq!(&vec[4], &[10, 11, 12, 13, 14]);
}

#[test]
fn array_from_vec_basic() {
    let mut vec: HeteroSizedVec<[usize]> = HeteroSizedVec::new();

    vec.push(vec![0_usize]);
    vec.push(vec![1_usize, 2]);
    vec.push(vec![3_usize, 4, 5]);
    vec.push(vec![6_usize, 7, 8, 9]);
    vec.push(vec![10_usize, 11, 12, 13, 14]);

    assert_eq!(&vec[0], &[0]);
    assert_eq!(&vec[1], &[1, 2]);
    assert_eq!(&vec[2], &[3, 4, 5]);
    assert_eq!(&vec[3], &[6, 7, 8, 9]);
    assert_eq!(&vec[4], &[10, 11, 12, 13, 14]);
}

#[test]
fn str_basic() {
    let mut vec: HeteroSizedVec<str> = HeteroSizedVec::new();

    let strings = &["hello", "world", "foo", "bar", "baz"];

    for s in strings.iter().copied() {
        vec.push(s);
    }

    for (i, s) in strings.iter().copied().enumerate() {
        assert_eq!(&vec[i], s);
    }
}

#[test]
fn mutate_arrays() {
    let mut vec: HeteroSizedVec<[u8]> = HeteroSizedVec::new();

    vec.push_value([0]);
    vec.push_value([1, 2]);
    vec.push_value([3, 4, 5]);
    vec.push_value([6, 7, 8, 9]);

    let mut i = 0;
    for array in &mut vec {
        for elem in array {
            *elem = i;
            i += 1;
        }
    }

    for array in &mut vec {
        for elem in array {
            *elem *= 2;
        }
    }

    let mut i = 0;
    for array in &vec {
        for &elem in array {
            assert_eq!(elem, i * 2);
            i += 1;
        }
    }
}

#[test]
fn mutate_arrays_copy_slice() {
    let mut vec: HeteroSizedVec<[u8]> = HeteroSizedVec::new();

    vec.push(&[0_u8] as &[_]);
    vec.push(&[1_u8, 2] as &[_]);
    vec.push(&[3_u8, 4, 5] as &[_]);
    vec.push(&[6_u8, 7, 8, 9] as &[_]);

    let mut i = 0;
    for array in &mut vec {
        for elem in array {
            *elem = i;
            i += 1;
        }
    }

    for array in &mut vec {
        for elem in array {
            *elem *= 2;
        }
    }

    let mut i = 0;
    for array in &vec {
        for &elem in array {
            assert_eq!(elem, i * 2);
            i += 1;
        }
    }
}

#[test]
fn mutate_arrays_from_vec() {
    let mut vec: HeteroSizedVec<[u8]> = HeteroSizedVec::new();

    vec.push(vec![0]);
    vec.push(vec![1, 2]);
    vec.push(vec![3, 4, 5]);
    vec.push(vec![6, 7, 8, 9]);

    let mut i = 0;
    for array in &mut vec {
        for elem in array {
            *elem = i;
            i += 1;
        }
    }

    for array in &mut vec {
        for elem in array {
            *elem *= 2;
        }
    }

    let mut i = 0;
    for array in &vec {
        for &elem in array {
            assert_eq!(elem, i * 2);
            i += 1;
        }
    }
}

#[test]
fn closure_boxing() {
    #[inline(never)]
    fn closure(n: usize) -> impl Fn() -> usize {
        move || n
    }

    let mut vec: HeteroSizedVec<dyn Fn() -> usize> = HeteroSizedVec::new();
    for n in 0..10 {
        vec.push_value(closure(n));
    }

    let vec2: Vec<Box<dyn Fn() -> usize>> = vec.into_box_vec();

    for (i, func) in vec2.iter().enumerate() {
        assert_eq!(i, func());
    }
}

#[test]
#[should_panic]
fn index_out_of_bounds() {
    let mut vec: HeteroSizedVec<str> = HeteroSizedVec::new();

    for _ in 0..3 {
        vec.push("hello world");
    }

    for i in 0..4 {
        let _ = &vec[i];
    }
}

pub mod drop_test {
    #[test]
    #[should_panic]
    fn dangling_pointer_should_panic() {
        use std::mem::forget;

        let counter = DropTestCounter::new();
        let token = counter.token();
        forget(token);
        counter.check();
    }

    #[test]
    #[should_panic]
    fn double_free_should_panic() {
        use std::ptr::drop_in_place;
        use std::mem::drop;

        let counter = DropTestCounter::new();
        let mut token = counter.token();

        unsafe {
            drop_in_place(&mut token);
            drop(token);
        }

        counter.check();
    }

    #[test]
    fn drop_test_sanity_check() {
        let counter = DropTestCounter::new();

        let mut tokens = Vec::new();
        for _ in 0..100 {
            tokens.push(counter.token());
        }
        drop(tokens);

        counter.check();
    }

    use std::sync::{
        atomic::{Ordering, AtomicI64},
        Arc,
    };

    pub use std::mem::drop;

    #[derive(Clone)]
    pub struct DropTestCounter { alive_count: Arc<AtomicI64> }

    pub struct DropTestToken {
        alive_count: Arc<AtomicI64>,
        already_dropped: bool,
    }

    impl DropTestCounter {
        pub fn new() -> Self {
            DropTestCounter {
                alive_count: Arc::new(AtomicI64::new(0))
            }
        }

        pub fn token(&self) -> DropTestToken {
            self.alive_count.fetch_add(1, Ordering::Relaxed);
            DropTestToken {
                alive_count: self.alive_count.clone(),
                already_dropped: false,
            }
        }

        pub fn check(&self) {
            assert_eq!(
                self.alive_count.load(Ordering::Relaxed),
                0,
                "dangling pointer detected",
            );
        }
    }

    impl Drop for DropTestToken {
        fn drop(&mut self) {
            if self.already_dropped {
                panic!("double free detected");
            }

            self.already_dropped = true;
            self.alive_count.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

#[test]
fn closure_drop_test() {
    use drop_test::*;

    #[inline(never)]
    fn closure(n: usize, token: DropTestToken) -> impl Fn() -> usize {
        move || {
            let token = &token;
            n
        }
    }

    let counter = DropTestCounter::new();

    let mut vec: HeteroSizedVec<dyn Fn() -> usize> = HeteroSizedVec::new();
    for n in 0..10 {
        vec.push_value(closure(n, counter.token()));
    }

    for (i, func) in vec.iter().enumerate() {
        assert_eq!(i, func());
    }

    drop(vec);
    counter.check();
}

#[test]
fn array_drop_test() {
    use drop_test::*;

    let counter = DropTestCounter::new();

    let mut vec: HeteroSizedVec<[DropTestToken]> = HeteroSizedVec::new();

    for i in 0..100 {
        let mut elem_vec: Vec<DropTestToken> = Vec::new();
        for _ in 0..i {
            elem_vec.push(counter.token());
        }
        vec.push(elem_vec);
    }

    drop(vec);

    counter.check();
}

#[test]
fn test_alignment() {
    pub trait Align {
        fn addr(&self) -> usize;

        fn align(&self) -> usize;
    }

    macro_rules! align_impl {
        ($align:expr)=>{{
            #[repr(align($align))]
            struct SpecialAlign(u8);

            impl Align for SpecialAlign {
                fn addr(&self) -> usize {
                    self as *const Self as usize
                }

                fn align(&self) -> usize { $align }
            }

            SpecialAlign(0)
        }};
    }

    let mut vec: HeteroSizedVec<dyn Align> = HeteroSizedVec::new();

    macro_rules! align_push_each {
        ($vec:expr, [$($align:expr),* $(,)?])=>{
            $(
            $vec.push_value(align_impl!($align));
            )*
        };
    }

    align_push_each!(vec, [2, 16, 32, 2, 2, 1024, 16, 16, 256, 32, 4]);

    for elem in &vec {
        assert!(elem.addr() % elem.align() == 0);
    }

}