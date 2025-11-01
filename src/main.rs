#![allow(
    clippy::incompatible_msrv,
    clippy::uninlined_format_args,
    clippy::similar_names,
    clippy::cast_precision_loss,
    dead_code,
    unused_imports
)]

extern crate core;
extern crate snailx;

#[cfg(not(feature = "__testing_bl_166"))] use core::hint::black_box;
#[cfg(feature = "__testing_bl_166")]
fn black_box<T>(x: T) -> T {
    x
}
use {
    snailx::{Args, MappedArgs, direct::set_argc_argv},
    std::{env, time::Instant}
};

/// helper that runs `setup()` then runs `op(&data)` and times it, repeating `reps` times.
/// prints total and average.
fn measure<T, S, F>(name: &str, setup: S, op: F, reps: usize)
where
    S: Fn() -> T,
    F: Fn(&mut T)
{
    let mut total_ns: u128 = 0;
    for _ in 0..reps {
        let mut data = setup();
        let start = Instant::now();
        op(black_box(&mut data));
        let elapsed = start.elapsed();
        total_ns += elapsed.as_nanos();
    }

    let total_s = (total_ns as f64) / 1_000_000_000.0;
    let avg_s = total_s / (reps as f64);

    let avg_ns = (total_ns as f64) / (reps as f64);

    println!(
        "{:<20} — total: {:>8.19} s, avg: {:>8.14} s/ {} ns ({} reps)",
        name, total_s, avg_s, avg_ns, reps
    );
}

const ARGS: [*const u8; 18] = [
    "target/debug/snailx\0".as_ptr(),
    "hi\0".as_ptr(),
    "1\0".as_ptr(),
    "2\0".as_ptr(),
    "3\0".as_ptr(),
    "4\0".as_ptr(),
    "5\0".as_ptr(),
    "6\0".as_ptr(),
    "7\0".as_ptr(),
    "8\0".as_ptr(),
    "9\0".as_ptr(),
    "10\0".as_ptr(),
    "11\0".as_ptr(),
    "12\0".as_ptr(),
    "13\0".as_ptr(),
    "14\0".as_ptr(),
    "15\0".as_ptr(),
    "-g\0".as_ptr()
];

#[cfg(any(feature = "std", feature = "to_core_cstr"))]
#[allow(clippy::too_many_lines, clippy::iter_nth_zero)]
fn main() {
    unsafe {
        #[allow(clippy::cast_possible_truncation)]
        set_argc_argv(ARGS.len() as u32, ARGS.as_ptr());
    }

    let mut args = Args::new();
    let mut str_args = MappedArgs::utf8();

    let len = args.len();
    // no str_args.len()
    let len2 = ARGS.len();

    println!("len: {}, len2: {}", len, len2);

    let a = args.nth(0);
    let b = str_args.nth(0);

    let c = args.nth(2);
    let d = str_args.nth(2);

    println!(
        "a: \"{}\", b: \"{}\", c: \"{}\" d: \"{}\"",
        a.unwrap().to_stdlib().to_string_lossy(),
        b.unwrap(),
        c.unwrap().to_stdlib().to_string_lossy(),
        d.unwrap()
    );

    #[cfg(feature = "rev_iter")]
    {
        let mut args2 = Args::new();
        let mut str_args2 = MappedArgs::utf8();

        let a = args2.nth_back(0);
        let b = str_args2.nth_back(0);

        let c = args2.nth_back(2);
        let d = str_args2.nth_back(2);

        println!(
            "a: \"{}\", b: \"{}\", c: \"{}\" d: \"{}\"",
            a.unwrap().to_stdlib().to_string_lossy(),
            b.unwrap(),
            c.unwrap().to_stdlib().to_string_lossy(),
            d.unwrap()
        );
    }

    // print args for testing
    for arg in Args::new() {
        println!("cstr arg: {}", arg.to_stdlib().to_string_lossy());
    }

    println!();

    #[cfg(feature = "std")]
    for arg in MappedArgs::osstr() {
        println!("osstr arg: {}", arg.to_string_lossy());
    }

    println!();

    for arg in MappedArgs::utf8() {
        println!("str arg: {}", arg);
    }

    #[cfg(feature = "rev_iter")]
    {
        println!("\nReversed:");

        for arg in Args::new().rev() {
            println!("rev cstr arg: {}", arg.to_stdlib().to_string_lossy());
        }
    }

    println!();

    #[cfg(all(feature = "std", feature = "rev_iter"))]
    for arg in MappedArgs::osstr().rev() {
        println!("rev osstr arg: {}", arg.to_string_lossy());
    }

    println!();

    #[cfg(feature = "rev_iter")]
    for arg in MappedArgs::utf8().rev() {
        println!("rev str arg: {}", arg);
    }

    println!();

    #[cfg(feature = "indexing_parser")]
    {
        use snailx::indexing_parser::{IndexingParser, OptRule};

        let rules = &[
            OptRule::new("greet").set_long("greet").set_short('g'),
            OptRule::new("number").set_long("number").set_short('n').set_val_count(1)
        ];

        let mut args = IndexingParser::new();
        println!("Unparsed: {:?}\n", args);
        args.parse(rules, |_| true).expect("failed to parse");
        println!("Parsed: {:?}\n", args);
        println!("Parsed pretty: {:#?}\n", args);

        const NUM: [*const u8; 1] = ["-n\0".as_ptr()];

        args.reset();
        unsafe {
            set_argc_argv(1, NUM.as_ptr());
        }
        args.parse(rules, |_| false).expect("failed to parse");

        println!("Parsed (incomplete n): {:?}\n", args);
        println!("Parsed pretty (incomplete n): {:#?}\n", args);

        const NUM_FULL: [*const u8; 2] = ["-n\0".as_ptr(), "10\0".as_ptr()];

        args.reset();
        unsafe {
            set_argc_argv(2, NUM_FULL.as_ptr());
        }
        args.parse(rules, |_| false).expect("failed to parse");

        println!("Parsed (full n): {:?}\n", args);
        println!("Parsed pretty (full n): {:#?}\n", args);
    }

    // CLI: [reps] [arg_count]
    let args: Vec<String> = env::args().collect();
    let reps: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100_000);

    println!("Running {} reps\n", reps);

    // snail_cstr
    measure(
        "snail_cstr",
        Args::new,
        |args| {
            for arg in black_box(args) {
                black_box(arg);
            }
        },
        reps
    );

    #[cfg(feature = "std")]
    // snail_osstr
    measure(
        "snail_osstr",
        MappedArgs::osstr,
        |args| {
            for arg in black_box(args) {
                black_box(arg);
            }
        },
        reps
    );

    // snail_str
    measure(
        "snail_str",
        MappedArgs::utf8,
        |args| {
            for s in black_box(args) {
                black_box(s);
            }
        },
        reps
    );

    measure(
        "stdlib_osstring",
        env::args_os,
        |args_os| {
            for arg in black_box(args_os) {
                black_box(arg);
            }
        },
        reps
    );

    measure(
        "stdlib_string",
        env::args,
        |args_s| {
            for arg in black_box(args_s) {
                black_box(arg);
            }
        },
        reps
    );

    println!("\nDone.");
}

#[cfg(not(any(feature = "std", feature = "to_core_cstr")))]
fn main() {
    eprintln!("main test requires to_stdlib");
    std::process::exit(0);
}
