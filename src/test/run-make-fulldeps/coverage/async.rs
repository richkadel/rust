#![allow(unused_assignments)]

// require-rust-edition-2018

async fn c(x: u8) -> u8 {
    if x == 8 {
        1
    } else {
        0
    }
}

async fn d() -> u8 { 1 }

// Reports `0` coverage *only* with -Clink-dead-code (on by default except under Windows)
async fn e() -> u8 { 1 }

async fn f() -> u8 { 1 }

// Reports `0` coverage *only* with -Clink-dead-code (on by default except under Windows)
async fn foo() -> [bool; 10] { [false; 10] }

pub async fn g(x: u8) {
    match x {
        y if e().await == y => (),
        y if f().await == y => (),
        _ => (),
    }
}

async fn h(x: usize) {
    match x {
        y if foo().await[y] => (),
        _ => (),
    }
}

async fn i(x: u8) {
    match x {
        y if c(x).await == y + 1 => { d().await; },
        y if f().await == y + 1 => (),
        _ => (),
    } // shows `0` coverage on this line (see discussion below)
}

fn j(x: u8) {
    let c = 1;
    let f = 2;
    match x {
        y if c == y + 1 => (),
        y if f == y + 1 => (),
        _ => (),
    } // non-async function should show no coverage on this line (compared with `i()`)
}

// TODO: Compare with `l()`, unused `k()` is allowed to compile, and shows `0` coverage, as desired
// (but not expected).
fn k(x: u8) {
    match x {
        1 => (),
        2 => (),
        _ => (),
    }
}

fn l(x: u8) {
    match x {
        1 => (),
        2 => (),
        _ => (),
    } // TODO(richkadel): Compare with `j()`, why does this line show coverage of `1`?
}

fn main() {
    let _ = g(10);
    let _ = h(9);
    let mut future = Box::pin(i(8));
    j(7);
    l(6);
    executor::block_on(future.as_mut());
}

mod executor {
    use core::{
        future::Future,
        pin::Pin,
        task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    };

    pub fn block_on<F: Future>(mut future: F) -> F::Output {
        let mut future = unsafe { Pin::new_unchecked(&mut future) };

        static VTABLE: RawWakerVTable = RawWakerVTable::new(
            |_| unimplemented!("clone"),
            |_| unimplemented!("wake"),
            |_| unimplemented!("wake_by_ref"),
            |_| (),
        );
        let waker = unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) };
        let mut context = Context::from_waker(&waker);

        loop {
            if let Poll::Ready(val) = future.as_mut().poll(&mut context) {
                break val;
            }
        }
    }
}

// Async function bodies are executed in (hidden) closures that are "unused" if not "awaited"
// with an executor. If unused, they have no coverage, so the enclosing MIR (same function,
// without the closure body) submits a "Gap Region" to the Coverage Map, giving the function
// a coverage count (of `0`) *only* if the closure doesn't have it's own coverage. If the
// closure *is* used, the "Gap Region" is ignored *only* on the lines that also have coverage
// regions from the closure. That means lines that may otherwise have _no_ closure, in
// non-async functions, will have a coverage count of 0 in async functions.
//
// There is a tradeoff between highlighting too little uncovered code versus highlighting
// too much, and the conservative choice is the latter.
//
// TODO(richkadel): replace this TODO with a FIXME and issue once I'm sure this is how
// I'll leave it:
// FIXME(?????): Possible workarounds (with some effort) include:
// * Change how the Gap Region is injected, in the enclosing MIR. Currently,
//   `...::closure::spans::CoverageSpans::to_refined_spans()` adds a
//   "CoverageSpan::unreachable()" to the coverage spans for each enclosed closure. With some
//   effort, we could generate the actual coverage spans for the closure's MIR, and then add them
//   as unreachable GapRegion's.
// * *Or* we could check at codegen time if the closure's MIR *was* added to the coverage map,
//   and only add the Gap Region (under the enclosing function's coverage map) if not.
