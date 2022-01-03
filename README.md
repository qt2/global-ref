# global-ref

[![Crates.io](https://img.shields.io/crates/v/global-ref)](https://crates.io/crates/global-ref)
[![API reference](https://docs.rs/global-ref/badge.svg)](https://docs.rs/global-ref/)

## Overview
This crate is used to share references between functions through statics.

**Because the implementation internally converts raw pointers to usize and shares them between threads, fetching references is essentially unsafe. If you use this crate, please verify its safety before using it.**

## Examples
```rust
use std::thread;
use global_ref::GlobalMut;

fn main() {
    static GLOBAL: GlobalMut<i32> = GlobalMut::new();

    let mut content = 0;

    GLOBAL.with(&mut content, || {
        fn add_one() {
            *GLOBAL.get_mut() += 1;
        }

        let handle = thread::spawn(add_one);
        handle.join().unwrap();
        assert_eq!(*GLOBAL.get(), 1);
    });

    assert!(GLOBAL.try_get().is_none());
}
```

## License
MIT