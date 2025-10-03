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

fn snailx_and_stdlib_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("args_iteration");

    group.bench_function("snailx_cstr", |b| {
        b.iter_batched_ref(
            snailx::args,
            |args| {
                // measured: iterate and convert each arg
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

    group.bench_function("snailx_cstr_slice", |b| {
        b.iter_batched_ref(
            snailx::arg_ptrs,
            |args| {
                for arg in black_box(black_box(args).iter()) {
                    black_box(arg);
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

pub fn stdcmp(c: &mut Criterion) {
    // Use detailed, accuracy-focused but standard baseline settings.
    // All of these can be overridden via CLI flags (configure_from_args).
    snailx_and_stdlib_iteration(c);
    snailx_and_stdlib_nth(c);
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

    stdcmp(&mut criterion);

    criterion.final_summary();
}
