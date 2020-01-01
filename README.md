
### Heterovec

A vector that stores dynamically sized types in-place, at the cost of being 
random-access. The element type may be:

- a trait object, eg. `HeteroSizedVec<dyn Fn() -> u32>`
- an array, eg. `HeteroSizedVec<[u32]>`
- a `str`, eg. `HeteroSizedVec<str>`

### Trustworthiness

tl;dr: **I would not recommend production use.**

This targets nightly, opens many feature-gates, possibly relies on some de-facto details
of memory, and pushes unsafe memory management excitingly far. 

Furthermore, I created this quickly, with no peer review, and with inadequate levels of
testing. This may simply break in a future nightly build, or even not work correctly
right now.    

### Examples

Trait objects:

```rust
extern crate heterovec;
use heterovec::HeteroSizedVec;

fn main() {
    let mut functions: HeteroSizedVec<dyn Fn(i32) -> i32> = HeteroSizedVec::new();
    
    fn adder(x: i32) -> impl Fn(i32) -> i32 {
        move |y| x + y
    }
    
    fn multiplier(x: i32) -> impl Fn(i32) -> i32 {
        move |y| x * y
    }
    
    functions.push_value(adder(1));       // functions[0]
    functions.push_value(adder(2));       // functions[1]
    functions.push_value(adder(3));       // functions[2]
    functions.push_value(multiplier(10)); // functions[3]
    functions.push_value(multiplier(16)); // functions[4]
    
    for (i, &(input, output)) in [
        (10, 11),  // 10 + 1  == 11
        (50, 52),  // 50 + 2  == 52
        (0, 3),    // 0 + 3   == 3
        (7, 70),   // 7 * 10  == 70
        (32, 512), // 32 * 16 == 512
    ].iter().enumerate() {
        assert_eq!(functions[i](input), output);
    }
}
```

Arrays:

```rust 
extern crate heterovec;
use heterovec::HeteroSizedVec;

fn main() {
    let mut arrays: HeteroSizedVec<[u32]> = HeteroSizedVec::new();
    
    arrays.push_value([1]);
    arrays.push_value([2, 3]);
    arrays.push_value([4, 5, 6]);
    arrays.push_value([7, 8, 9, 10]);
    
    let elem_5: Vec<u32> = (0_u32..=99).collect::<Vec<u32>>();
    arrays.push(elem_5);
    
    // mutate the elements
    for slice in &mut arrays {
        for number in slice {
            *number += 1;
        }
    }
    
    println!("arrays = {:?}", arrays);
    
    let sums: Vec<u32> = arrays.iter()
        .map(|slice: &[u32]| slice.iter().copied().sum())
        .collect();
        
    assert_eq!(sums, vec![
        2,
        3 + 4,
        5 + 6 + 7,
        8 + 9 + 10 + 11,
        (1..=100).sum()
    ]);
}
```

Strs:

```rust
extern crate heterovec;
use heterovec::HeteroSizedVec;

fn main() {
    let mut strs: HeteroSizedVec<str> = HeteroSizedVec::new();
    
    strs.push("hello");
    strs.push("world");
    strs.push(format!("{}+{}={}", 2, 2, 4).as_ref());
 
    // although the elements are not separate `String` allocations, they are owned.
    // (they are not static string)
    
    {
        let elem: &mut str = &mut strs[1];
        
        // the unsafety comes from mutating a `str`, not from `heterovec`
        unsafe { elem.as_bytes_mut()[4] = b'l' };
    }
    
    assert_eq!(&strs[0], "hello");
    assert_eq!(&strs[1], "worll"); 
    assert_eq!(&strs[2], "2+2=4");
}

```