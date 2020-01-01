
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

