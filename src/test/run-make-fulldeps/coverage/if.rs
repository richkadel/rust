// #![allow(unused_assignments, unused_variables)]

fn notcalled() {
    println!("pub never called");
}

fn main() {
    // let unused = || {
    //     println!("closure never called");
    // };



    // Initialize test constants in a way that cannot be determined at compile time, to ensure
    // rustc and LLVM cannot optimize out statements (or coverage counters) downstream from
    // dependent conditions.
    let
    is_true
    =
        std::env::args().len()
    ==
        1
    ;
    let
        mut
    countdown
    =
        0
    ;
    if
        is_true
    {
        countdown
        =
            10
        ;
    }
}
