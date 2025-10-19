#![allow(clippy::cast_possible_truncation, clippy::incompatible_msrv)]

extern crate core;
extern crate criterion;

use {
    core::{hint::black_box, ptr, time::Duration},
    criterion::{BatchSize, Criterion}
};

const ARGV_MINIMAL: [*const u8; 1] = [b"bash\0".as_ptr()];

fn bench_snailx_iter_minimal(c: &mut Criterion) {
    unsafe { snailx::direct::set_argc_argv(ARGV_MINIMAL.len() as u32, ARGV_MINIMAL.as_ptr()) };

    let mut group = c.benchmark_group("snailx/minimal/iterate");

    group.bench_function("cstr", |b| {
        b.iter_batched_ref(
            snailx::Args::new,
            |args| {
                for arg in black_box(args) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("osstr", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::osstr,
            |args| {
                for arg in black_box(args) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("str", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::utf8,
            |args| {
                for s in black_box(args) {
                    black_box(s);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

fn bench_snailx_nth_minimal(c: &mut Criterion) {
    unsafe { snailx::direct::set_argc_argv(ARGV_MINIMAL.len() as u32, ARGV_MINIMAL.as_ptr()) };

    let mut group = c.benchmark_group("snailx/minimal/nth");

    group.bench_function("cstr", |b| {
        b.iter_batched_ref(
            snailx::Args::new,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("osstr", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::osstr,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("str", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::utf8,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

#[cfg(feature = "std")]
fn bench_iter_snailx_vs_std(c: &mut Criterion) {
    let mut group = c.benchmark_group("args/iterate/snailx_vs_std");
    group.bench_function("snailx_cstr", |b| {
        b.iter_batched_ref(
            snailx::Args::new,
            |args| {
                for arg in black_box(args) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("snailx_osstr", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::osstr,
            |args| {
                for arg in black_box(args) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snailx_str", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::utf8,
            |args| {
                for s in black_box(args) {
                    black_box(s);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("std_osstring", |b| {
        b.iter_batched_ref(
            std::env::args_os,
            |args_os| {
                for arg in black_box(args_os) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("std_string", |b| {
        b.iter_batched_ref(
            std::env::args,
            |args_s| {
                for arg in black_box(args_s) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

#[cfg(feature = "std")]
fn bench_nth_snailx_vs_std(c: &mut Criterion) {
    let mut group = c.benchmark_group("args/nth/snailx_vs_std");

    group.bench_function("snailx_cstr", |b| {
        b.iter_batched_ref(
            snailx::Args::new,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("snailx_osstr", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::osstr,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snailx_str", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::utf8,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("std_osstring", |b| {
        b.iter_batched_ref(
            std::env::args_os,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("std_string", |b| {
        b.iter_batched_ref(
            std::env::args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

const ARGV_PRESET_CMDLINE: [*const u8; 10] = [
    b"bash\0".as_ptr(),
    b"-c\0".as_ptr(),
    // not a mistake. strings like in "bash -c "some cmd"" are (usually) interpreted as one
    // argument
    b"pacman -Syu\0".as_ptr(),
    b"||\0".as_ptr(),
    b"echo\0".as_ptr(),
    b"\"failed\"".as_ptr(),
    b"&&\0".as_ptr(),
    b"journalctl\0".as_ptr(),
    b"-x\0".as_ptr(),
    b"-e\0".as_ptr()
];

fn bench_snailx_iter_preset(c: &mut Criterion) {
    unsafe {
        snailx::direct::set_argc_argv(
            ARGV_PRESET_CMDLINE.len() as u32,
            ARGV_PRESET_CMDLINE.as_ptr()
        )
    };

    let mut group = c.benchmark_group("snailx/preset/iterate");

    group.bench_function("cstr", |b| {
        b.iter_batched_ref(
            snailx::Args::new,
            |args| {
                for arg in black_box(args) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("osstr", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::osstr,
            |args| {
                for arg in black_box(args) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("str", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::utf8,
            |args| {
                for s in black_box(args) {
                    black_box(s);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

fn bench_snailx_nth_preset(c: &mut Criterion) {
    unsafe {
        snailx::direct::set_argc_argv(
            ARGV_PRESET_CMDLINE.len() as u32,
            ARGV_PRESET_CMDLINE.as_ptr()
        )
    };

    let mut group = c.benchmark_group("snailx/preset/nth");

    group.bench_function("cstr", |b| {
        b.iter_batched_ref(
            snailx::Args::new,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(5)));
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("osstr", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::osstr,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(5)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("str", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::utf8,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(5)));
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

fn bench_snailx_helpers(c: &mut Criterion) {
    let mut group = c.benchmark_group("snailx/helpers");

    group.bench_function("c_ptr_try_to_str", |b| {
        b.iter(|| {
            black_box(snailx::bench_helpers::try_to_str(black_box(black_box(
                const { b"afairlylongtypicalargcstr\0".as_ptr() }
            ))))
        });
    });

    group.bench_function("iter_len", |b| {
        b.iter_batched_ref(
            || {
                let mem = Vec::from(&[ptr::null(); 1024]);
                let p: *const *const u8 = mem.as_ptr();
                (p, unsafe { p.add(1023) })
            },
            |(start, end)| {
                black_box(unsafe {
                    snailx::bench_helpers::len(
                        black_box(*black_box(start)),
                        black_box(*black_box(end))
                    )
                })
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("cstr/to_stdlib", |b| {
        b.iter(|| {
            black_box(
                black_box(unsafe {
                    snailx::CStr::from_ptr(black_box(
                        const { b"fairlylongcstrargeventhoughlengthdoesntmatter\0".as_ptr() }
                    ))
                })
                .to_stdlib()
            );
        });
    });

    group.finish();
}

pub fn bench(c: &mut Criterion) {
    bench_snailx_iter_minimal(c);
    bench_snailx_nth_minimal(c);

    #[cfg(feature = "std")]
    {
        bench_iter_snailx_vs_std(c);
        bench_nth_snailx_vs_std(c);
    }

    bench_snailx_iter_preset(c);
    bench_snailx_nth_preset(c);

    bench_snailx_helpers(c);
}

fn main() {
    let mut crit: Criterion<_> = Criterion::default()
        .sample_size(800) // default is 100; use a bit larger for stability
        .measurement_time(Duration::from_secs(16))
        .warm_up_time(Duration::from_secs(4))
        .nresamples(800_000) // default is 100_000; increase for tighter CIs
        .noise_threshold(0.001) // treat changes below 0.1% as noise
        .confidence_level(0.999) // tighter confidence interval
        .configure_from_args();

    bench(&mut crit);

    crit.final_summary();
}
