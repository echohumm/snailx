use {
    core::time::Duration,
    criterion::{BatchSize, Criterion}
};

#[rustversion::before(1.66)]
fn black_box<T>(dummy: T) -> T {
    dummy
}

#[rustversion::since(1.66)]
fn black_box<T>(dummy: T) -> T {
    core::hint::black_box(dummy)
}

// TODO: split into multiple files

const PRESET_ARGV_SMALL: [*const u8; 1] = [b"bash\0".as_ptr()];

fn snailx_iter_small(c: &mut Criterion) {
    unsafe {
        snailx::direct::set_argc_argv(PRESET_ARGV_SMALL.len() as u32, PRESET_ARGV_SMALL.as_ptr())
    };

    let mut group = c.benchmark_group("snailx_iter_small");

    group.bench_function("cstr", |b| {
        b.iter_batched_ref(
            snailx::args,
            |args| {
                for arg in black_box(args) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("osstr", |b| {
        b.iter_batched_ref(
            snailx::osstr_args,
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
            snailx::str_args,
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

fn snailx_nth_small(c: &mut Criterion) {
    unsafe {
        snailx::direct::set_argc_argv(PRESET_ARGV_SMALL.len() as u32, PRESET_ARGV_SMALL.as_ptr())
    };

    let mut group = c.benchmark_group("snailx_nth_small");

    group.bench_function("cstr", |b| {
        b.iter_batched_ref(
            snailx::args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("osstr", |b| {
        b.iter_batched_ref(
            snailx::osstr_args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("str", |b| {
        b.iter_batched_ref(
            snailx::str_args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

fn snailx_and_stdlib_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("args_iteration");
    group.bench_function("snailx_cstr", |b| {
        b.iter_batched_ref(
            snailx::args,
            |args| {
                for arg in black_box(args) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snailx_osstr", |b| {
        b.iter_batched_ref(
            snailx::osstr_args,
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
            snailx::str_args,
            |args| {
                for s in black_box(args) {
                    black_box(s);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("stdlib_osstring", |b| {
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

    group.bench_function("stdlib_string", |b| {
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

fn snailx_and_stdlib_nth(c: &mut Criterion) {
    let mut group = c.benchmark_group("args_nth");

    group.bench_function("snailx_cstr", |b| {
        b.iter_batched_ref(
            snailx::args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snailx_osstr", |b| {
        b.iter_batched_ref(
            snailx::osstr_args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snailx_str", |b| {
        b.iter_batched_ref(
            snailx::str_args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("stdlib_osstring", |b| {
        b.iter_batched_ref(
            std::env::args_os,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("stdlib_string", |b| {
        b.iter_batched_ref(
            snailx::args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

const PRESET_ARGV: [*const u8; 10] = [
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

fn snailx_preset_iter(c: &mut Criterion) {
    unsafe { snailx::direct::set_argc_argv(PRESET_ARGV.len() as u32, PRESET_ARGV.as_ptr()) };

    let mut group = c.benchmark_group("snailx_iter_preset");

    group.bench_function("cstr", |b| {
        b.iter_batched_ref(
            snailx::args,
            |args| {
                for arg in black_box(args) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("osstr", |b| {
        b.iter_batched_ref(
            snailx::osstr_args,
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
            snailx::str_args,
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

fn snailx_preset_args_nth(c: &mut Criterion) {
    unsafe { snailx::direct::set_argc_argv(PRESET_ARGV.len() as u32, PRESET_ARGV.as_ptr()) };

    let mut group = c.benchmark_group("snailx_nth_preset");

    group.bench_function("cstr", |b| {
        b.iter_batched_ref(
            snailx::args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("osstr", |b| {
        b.iter_batched_ref(
            snailx::osstr_args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("str", |b| {
        b.iter_batched_ref(
            snailx::str_args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

fn snailx_general(c: &mut Criterion) {
    let mut group = c.benchmark_group("snailx_general");

    group.bench_function("try_to_str", |b| {
        b.iter(|| {
            black_box(snailx::try_to_str(black_box(
                black_box(const { b"afairlylongtypicalargcstr\0" }).as_ptr()
            )))
        });
    });

    group.finish();
}

pub fn bench(c: &mut Criterion) {
    snailx_iter_small(c);
    snailx_nth_small(c);

    snailx_and_stdlib_iteration(c);
    snailx_and_stdlib_nth(c);

    snailx_preset_iter(c);
    snailx_preset_args_nth(c);

    snailx_general(c);
}

fn main() {
    let mut criterion: Criterion<_> = Criterion::default()
        .sample_size(800) // default is 100; use a bit larger for stability
        .measurement_time(Duration::from_secs(16))
        .warm_up_time(Duration::from_secs(4))
        .nresamples(800_000) // default is 100_000; increase for tighter CIs
        .noise_threshold(0.001) // treat changes below 0.1% as noise
        .confidence_level(0.999) // tighter confidence interval
        .configure_from_args();

    bench(&mut criterion);

    criterion.final_summary();
}
