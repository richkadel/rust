// run-rustfix

#![feature(range_is_empty)]
#![warn(clippy::len_zero)]
#![allow(unused)]
#![allow(stable_features)] // TODO: https://github.com/rust-lang/rust-clippy/issues/5956

mod issue_3807 {
    // With the feature enabled, `is_empty` should be suggested
    fn suggestion_is_fine() {
        let _ = (0..42).len() == 0;
    }
}

fn main() {}
