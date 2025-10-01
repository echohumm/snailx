use {
    core::{hint::black_box, time::Duration},
    criterion::{BatchSize, Criterion}
};

// TODO: split into multiple files

fn snail_and_stdlib_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("args_iteration");

    group.bench_function("snail_cstr", |b| {
        b.iter_batched_ref(
            snail::args,
            |args| {
                // measured: iterate and convert each arg
                for arg in black_box(args) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snail_osstr", |b| {
        b.iter_batched_ref(
            snail::osstr_args,
            |args| {
                for arg in black_box(args) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snail_str", |b| {
        b.iter_batched_ref(
            snail::str_args,
            |args| {
                for s in black_box(args) {
                    black_box(s);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snail_cstr_slice", |b| {
        b.iter_batched_ref(
            snail::arg_ptrs,
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

fn snail_and_stdlib_nth(c: &mut Criterion) {
    let mut group = c.benchmark_group("args_nth");

    group.bench_function("snail_cstr", |b| {
        b.iter_batched_ref(
            snail::args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snail_osstr", |b| {
        b.iter_batched_ref(
            snail::osstr_args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snail_str", |b| {
        b.iter_batched_ref(
            snail::str_args,
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
            snail::args,
            |args| {
                let _ = black_box(black_box(args).nth(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

pub fn stdcmp() {
    // Use detailed, accuracy-focused but standard baseline settings.
    // All of these can be overridden via CLI flags (configure_from_args).
    let mut criterion: Criterion<_> = Criterion::default()
        .sample_size(400) // default is 100; use a bit larger for stability
        .measurement_time(Duration::from_secs(8))
        .warm_up_time(Duration::from_secs(4))
        .nresamples(400_000) // default is 100_000; increase for tighter CIs
        .noise_threshold(0.005) // treat changes below 0.5% as noise
        .confidence_level(0.99) // tighter confidence interval
        .configure_from_args();
    snail_and_stdlib_iteration(&mut criterion);
    snail_and_stdlib_nth(&mut criterion);
}

fn main() {
    stdcmp();

    Criterion::default().configure_from_args().final_summary();
}
