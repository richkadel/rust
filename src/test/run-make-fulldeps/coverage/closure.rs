#![allow(unused_assignments, unused_variables)]

fn main() {
    // Initialize test constants in a way that cannot be determined at compile time, to ensure
    // rustc and LLVM cannot optimize out statements (or coverage counters) downstream from
    // dependent conditions.
    let is_true = std::env::args().len() == 1;
    let is_false = ! is_true;

    let mut some_string = Some(String::from("the string content"));
    println!(
        "The string or alt: {}"
        ,
        some_string
            .
            unwrap_or_else
        (
            ||
            {
                let mut countdown = 0;
                if is_false {
                    countdown = 10;
                }
                "alt string 1".to_owned()
            }
        )
    );

    some_string = Some(String::from("the string content"));
    let
        a
    =
        ||
    {
        let mut countdown = 0;
        if is_false {
            countdown = 10;
        }
        "alt string 2".to_owned()
    };
    println!(
        "The string or alt: {}"
        ,
        some_string
            .
            unwrap_or_else
        (
            a
        )
    );

    some_string = None;
    println!(
        "The string or alt: {}"
        ,
        some_string
            .
            unwrap_or_else
        (
            ||
            {
                let mut countdown = 0;
                if is_false {
                    countdown = 10;
                }
                "alt string 3".to_owned()
            }
        )
    );

    some_string = None;
    let
        a
    =
        ||
    {
        let mut countdown = 0;
        if is_false {
            countdown = 10;
        }
        "alt string 4".to_owned()
    };
    println!(
        "The string or alt: {}"
        ,
        some_string
            .
            unwrap_or_else
        (
            a
        )
    );

    let
        quote_closure
    =
        |val|
    {
        let mut countdown = 0;
        if is_false {
            countdown = 10;
        }
        format!("'{}'", val)
    };
    println!(
        "Repeated, quoted string: {:?}"
        ,
        std::iter::repeat("repeat me")
            .take(5)
            .map
        (
            quote_closure
        )
            .collect::<Vec<_>>()
    );

    let
        _unused_closure
    =
        |
            mut countdown
        |
    {
        if is_false {
            countdown = 10;
        }
        "closure should be unused".to_owned()
    };

    // Note that unused closures are not codegenned, so coverage counters are not generated for the
    // body of any unused closure. To ensure closures do appear in coverage analysis, the enclosing
    // function adds a "Gap Region" for the closure body. Gap Regions show zero coverage on _lines_
    // that have no other coverage. By using a "Gap Region" (instead of the normal "Code Region")
    // coverage results are less confusing for closures that _are_ used and executed, and unused
    // closures spanning at least 1 line not including opening and closing tokens (`|` and `}`)
    // show a coverage count of zero, as desired.
    //
    // However, an unused closure defined on a single line will likely not appear uncovered because
    // the enclosing function will typically have a code span (related to the closure's definition)
    // that touches the closure's line -- and as a result, the Gap Region is ignored for that line.
    //
    // Since Gap Regions deconflict with Code Regions at the line-level only, this seems to be the
    // best we can do.
    //
    // The following examples show some of these variations.

    let _short_unused_closure = | _unused_arg: u8 | println!("not called");

    let _shortish_unused_closure = | _unused_arg: u8 | {
        println!("not called")
    };

    let _as_short_unused_closure = |
        _unused_arg: u8
    | { println!("not called") };

    let _almost_as_short_unused_closure = |
        _unused_arg: u8
    | { println!("not called") }
    ;
}
