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
            snailx::MappedArgs::os,
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
            snailx::MappedArgs::os,
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

#[cfg(feature = "rev_iter")]
fn bench_snailx_iter_back_minimal(c: &mut Criterion) {
    unsafe { snailx::direct::set_argc_argv(ARGV_MINIMAL.len() as u32, ARGV_MINIMAL.as_ptr()) };

    let mut group = c.benchmark_group("snailx/minimal/iterate_back");

    group.bench_function("cstr", |b| {
        b.iter_batched_ref(
            snailx::Args::new,
            |args| {
                while let Some(arg) = black_box(black_box(&mut *args).next_back()) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("osstr", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::os,
            |args| {
                while let Some(arg) = black_box(black_box(&mut *args).next_back()) {
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
                while let Some(s) = black_box(black_box(&mut *args).next_back()) {
                    black_box(s);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

#[cfg(feature = "rev_iter")]
fn bench_snailx_nth_back_minimal(c: &mut Criterion) {
    unsafe { snailx::direct::set_argc_argv(ARGV_MINIMAL.len() as u32, ARGV_MINIMAL.as_ptr()) };

    let mut group = c.benchmark_group("snailx/minimal/nth_back");

    group.bench_function("cstr", |b| {
        b.iter_batched_ref(
            snailx::Args::new,
            |args| {
                let _ = black_box(black_box(args).nth_back(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("osstr", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::os,
            |args| {
                let _ = black_box(black_box(args).nth_back(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("str", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::utf8,
            |args| {
                let _ = black_box(black_box(args).nth_back(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

fn bench_snailx_fold_minimal(c: &mut Criterion) {
    unsafe { snailx::direct::set_argc_argv(ARGV_MINIMAL.len() as u32, ARGV_MINIMAL.as_ptr()) };

    let mut group = c.benchmark_group("snailx/minimal/fold");

    group.bench_function("cstr", |b| {
        b.iter_batched(
            snailx::Args::new,
            |args| {
                let _ = black_box(black_box(args).fold(0usize, |acc, _| acc + 1));
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("osstr", |b| {
        b.iter_batched(
            snailx::MappedArgs::os,
            |args| {
                let _ = black_box(black_box(args).fold(0usize, |acc, _| acc + 1));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("str", |b| {
        b.iter_batched(
            snailx::MappedArgs::utf8,
            |args| {
                let _ = black_box(black_box(args).fold(0usize, |acc, _| acc + 1));
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

#[cfg(feature = "rev_iter")]
fn bench_snailx_rfold_minimal(c: &mut Criterion) {
    unsafe { snailx::direct::set_argc_argv(ARGV_MINIMAL.len() as u32, ARGV_MINIMAL.as_ptr()) };

    let mut group = c.benchmark_group("snailx/minimal/rfold");

    group.bench_function("cstr", |b| {
        b.iter_batched(
            snailx::Args::new,
            |args| {
                let _ = black_box(black_box(args).rfold(0usize, |acc, _| acc + 1));
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("osstr", |b| {
        b.iter_batched(
            snailx::MappedArgs::os,
            |args| {
                let _ = black_box(black_box(args).rfold(0usize, |acc, _| acc + 1));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("str", |b| {
        b.iter_batched(
            snailx::MappedArgs::utf8,
            |args| {
                let _ = black_box(black_box(args).rfold(0usize, |acc, _| acc + 1));
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
            snailx::MappedArgs::os,
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

#[cfg(all(feature = "std", feature = "rev_iter"))]
fn bench_iter_back_snailx_vs_std(c: &mut Criterion) {
    let mut group = c.benchmark_group("args/iterate_back/snailx_vs_std");
    group.bench_function("snailx_cstr_back", |b| {
        b.iter_batched_ref(
            snailx::Args::new,
            |args| {
                while let Some(arg) = black_box(black_box(&mut *args).next_back()) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("snailx_osstr_back", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::os,
            |args| {
                while let Some(arg) = black_box(black_box(&mut *args).next_back()) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snailx_str_back", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::utf8,
            |args| {
                while let Some(s) = black_box(black_box(&mut *args).next_back()) {
                    black_box(s);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("std_osstring_back", |b| {
        b.iter_batched_ref(
            std::env::args_os,
            |args_os| {
                while let Some(arg) = black_box(black_box(&mut *args_os).next_back()) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("std_string_back", |b| {
        b.iter_batched_ref(
            std::env::args,
            |args_s| {
                while let Some(arg) = black_box(black_box(&mut *args_s).next_back()) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

#[cfg(all(feature = "std", feature = "rev_iter"))]
fn bench_nth_back_snailx_vs_std(c: &mut Criterion) {
    let mut group = c.benchmark_group("args/nth_back/snailx_vs_std");

    group.bench_function("snailx_cstr_back", |b| {
        b.iter_batched_ref(
            snailx::Args::new,
            |args| {
                let _ = black_box(black_box(args).nth_back(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("snailx_osstr_back", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::os,
            |args| {
                let _ = black_box(black_box(args).nth_back(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snailx_str_back", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::utf8,
            |args| {
                let _ = black_box(black_box(args).nth_back(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("std_osstring_back", |b| {
        b.iter_batched_ref(
            std::env::args_os,
            |args| {
                let _ = black_box(black_box(args).nth_back(black_box(0)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("std_string_back", |b| {
        b.iter_batched_ref(
            std::env::args,
            |args| {
                let _ = black_box(black_box(args).nth_back(black_box(0)));
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
            snailx::MappedArgs::os,
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

#[cfg(feature = "std")]
fn bench_fold_snailx_vs_std(c: &mut Criterion) {
    let mut group = c.benchmark_group("args/fold/snailx_vs_std");

    group.bench_function("snailx_cstr", |b| {
        b.iter_batched_ref(
            snailx::Args::new,
            |args| {
                black_box(black_box(args).fold(black_box(0usize), black_box(|acc, _| acc + 1)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snailx_osstr", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::os,
            |args| {
                black_box(black_box(args).fold(black_box(0usize), black_box(|acc, _| acc + 1)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("snailx_str", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::utf8,
            |args| {
                black_box(black_box(args).fold(black_box(0usize), black_box(|acc, _| acc + 1)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("std_osstring", |b| {
        b.iter_batched_ref(
            std::env::args_os,
            |args_os| {
                black_box(black_box(args_os).fold(black_box(0usize), black_box(|acc, _| acc + 1)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("std_string", |b| {
        b.iter_batched_ref(
            std::env::args,
            |args_s| {
                black_box(black_box(args_s).fold(black_box(0usize), black_box(|acc, _| acc + 1)));
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
            snailx::MappedArgs::os,
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
            snailx::MappedArgs::os,
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

#[cfg(feature = "rev_iter")]
fn bench_snailx_iter_back_preset(c: &mut Criterion) {
    unsafe {
        snailx::direct::set_argc_argv(
            ARGV_PRESET_CMDLINE.len() as u32,
            ARGV_PRESET_CMDLINE.as_ptr()
        )
    };

    let mut group = c.benchmark_group("snailx/preset/iterate_back");

    group.bench_function("cstr", |b| {
        b.iter_batched_ref(
            snailx::Args::new,
            |args| {
                while let Some(arg) = black_box(black_box(&mut *args).next_back()) {
                    black_box(arg);
                }
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("osstr", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::os,
            |args| {
                while let Some(arg) = black_box(black_box(&mut *args).next_back()) {
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
                while let Some(s) = black_box(black_box(&mut *args).next_back()) {
                    black_box(s);
                }
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

#[cfg(feature = "rev_iter")]
fn bench_snailx_nth_back_preset(c: &mut Criterion) {
    unsafe {
        snailx::direct::set_argc_argv(
            ARGV_PRESET_CMDLINE.len() as u32,
            ARGV_PRESET_CMDLINE.as_ptr()
        )
    };

    let mut group = c.benchmark_group("snailx/preset/nth_back");

    group.bench_function("cstr", |b| {
        b.iter_batched_ref(
            snailx::Args::new,
            |args| {
                let _ = black_box(black_box(args).nth_back(black_box(5)));
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("osstr", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::os,
            |args| {
                let _ = black_box(black_box(args).nth_back(black_box(5)));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("str", |b| {
        b.iter_batched_ref(
            snailx::MappedArgs::utf8,
            |args| {
                let _ = black_box(black_box(args).nth_back(black_box(5)));
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

fn bench_snailx_fold_preset(c: &mut Criterion) {
    unsafe {
        snailx::direct::set_argc_argv(
            ARGV_PRESET_CMDLINE.len() as u32,
            ARGV_PRESET_CMDLINE.as_ptr()
        )
    };

    let mut group = c.benchmark_group("snailx/preset/fold");

    group.bench_function("cstr", |b| {
        b.iter_batched(
            snailx::Args::new,
            |args| {
                let _ = black_box(black_box(args).fold(0usize, |acc, _| acc + 1));
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("osstr", |b| {
        b.iter_batched(
            snailx::MappedArgs::os,
            |args| {
                let _ = black_box(black_box(args).fold(0usize, |acc, _| acc + 1));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("str", |b| {
        b.iter_batched(
            snailx::MappedArgs::utf8,
            |args| {
                let _ = black_box(black_box(args).fold(0usize, |acc, _| acc + 1));
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

#[cfg(feature = "rev_iter")]
fn bench_snailx_rfold_preset(c: &mut Criterion) {
    unsafe {
        snailx::direct::set_argc_argv(
            ARGV_PRESET_CMDLINE.len() as u32,
            ARGV_PRESET_CMDLINE.as_ptr()
        )
    };

    let mut group = c.benchmark_group("snailx/preset/rfold");

    group.bench_function("cstr", |b| {
        b.iter_batched(
            snailx::Args::new,
            |args| {
                let _ = black_box(black_box(args).rfold(0usize, |acc, _| acc + 1));
            },
            BatchSize::SmallInput
        );
    });

    #[cfg(feature = "std")]
    group.bench_function("osstr", |b| {
        b.iter_batched(
            snailx::MappedArgs::os,
            |args| {
                let _ = black_box(black_box(args).rfold(0usize, |acc, _| acc + 1));
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("str", |b| {
        b.iter_batched(
            snailx::MappedArgs::utf8,
            |args| {
                let _ = black_box(black_box(args).rfold(0usize, |acc, _| acc + 1));
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

// Additional argv preset with long options to exercise IndexingParser long handling
#[cfg(feature = "indexing_parser")]
const ARGV_PRESET_LONG: [*const u8; 59] = [
    b"prog\0".as_ptr(),
    b"--num\0".as_ptr(),
    b"10\0".as_ptr(),
    b"--alpha\0".as_ptr(),
    b"--beta\0".as_ptr(),
    b"--gamma\0".as_ptr(),
    b"1\0".as_ptr(),
    b"2\0".as_ptr(),
    b"--delta\0".as_ptr(),
    b"3\0".as_ptr(),
    b"--epsilon\0".as_ptr(),
    b"--zeta\0".as_ptr(),
    b"--eta\0".as_ptr(),
    b"--theta\0".as_ptr(),
    b"--iota\0".as_ptr(),
    b"--kappa\0".as_ptr(),
    b"--lambda\0".as_ptr(),
    b"--mu\0".as_ptr(),
    b"--nu\0".as_ptr(),
    b"--xi\0".as_ptr(),
    b"--omicron\0".as_ptr(),
    b"--pi\0".as_ptr(),
    b"--rho\0".as_ptr(),
    b"--sigma\0".as_ptr(),
    b"--tau\0".as_ptr(),
    b"--limit\0".as_ptr(),
    b"42\0".as_ptr(),
    b"--path\0".as_ptr(),
    b"/tmp\0".as_ptr(),
    b"file1\0".as_ptr(),
    b"file2\0".as_ptr(),
    b"file3\0".as_ptr(),
    b"pos1\0".as_ptr(),
    b"pos2\0".as_ptr(),
    b"pos3\0".as_ptr(),
    b"--user\0".as_ptr(),
    b"alice\0".as_ptr(),
    b"--group\0".as_ptr(),
    b"staff\0".as_ptr(),
    b"--level\0".as_ptr(),
    b"7\0".as_ptr(),
    b"--dry-run\0".as_ptr(),
    b"--verbose\0".as_ptr(),
    b"--mode\0".as_ptr(),
    b"fast\0".as_ptr(),
    b"--retry\0".as_ptr(),
    b"3\0".as_ptr(),
    b"--timeout\0".as_ptr(),
    b"1000\0".as_ptr(),
    b"--output\0".as_ptr(),
    b"out.txt\0".as_ptr(),
    b"--config\0".as_ptr(),
    b"conf.toml\0".as_ptr(),
    b"--threads\0".as_ptr(),
    b"8\0".as_ptr(),
    b"--seed\0".as_ptr(),
    b"12345\0".as_ptr(),
    b"pos4\0".as_ptr(),
    b"pos5\0".as_ptr()
];

#[cfg(feature = "indexing_parser")]
const RULES: &[snailx::indexing_parser::OptRule; 20] = &[
    snailx::indexing_parser::OptRule::new_auto_long("num").set_val_count(1),
    snailx::indexing_parser::OptRule::new_auto_long("alpha"),
    snailx::indexing_parser::OptRule::new_auto_long("beta"),
    snailx::indexing_parser::OptRule::new_auto_long("gamma").set_val_count(2),
    snailx::indexing_parser::OptRule::new_auto_long("delta").set_val_count(1),
    snailx::indexing_parser::OptRule::new_auto_long("epsilon"),
    snailx::indexing_parser::OptRule::new_auto_long("zeta"),
    snailx::indexing_parser::OptRule::new_auto_long("eta"),
    snailx::indexing_parser::OptRule::new_auto_long("theta"),
    snailx::indexing_parser::OptRule::new_auto_long("iota"),
    snailx::indexing_parser::OptRule::new_auto_long("kappa"),
    snailx::indexing_parser::OptRule::new_auto_long("lambda"),
    snailx::indexing_parser::OptRule::new_auto_long("mu"),
    snailx::indexing_parser::OptRule::new_auto_long("nu"),
    snailx::indexing_parser::OptRule::new_auto_long("xi"),
    snailx::indexing_parser::OptRule::new_auto_long("omicron"),
    snailx::indexing_parser::OptRule::new_auto_long("pi"),
    snailx::indexing_parser::OptRule::new_auto_long("rho"),
    snailx::indexing_parser::OptRule::new_auto_long("sigma"),
    snailx::indexing_parser::OptRule::new_auto_long("tau")
];

#[cfg(feature = "indexing_parser")]
fn bench_indexing_parser_minimal(c: &mut Criterion) {
    unsafe { snailx::direct::set_argc_argv(ARGV_MINIMAL.len() as u32, ARGV_MINIMAL.as_ptr()) };

    let mut group = c.benchmark_group("indexing_parser/minimal");

    group.bench_function("parse_only", |b| {
        b.iter_batched(
            snailx::indexing_parser::IndexingParser::new,
            |mut p| {
                // TODO: bench with named positionals and different positional ranges too
                p.parse(black_box(RULES), ..usize::MAX, &[], black_box(|_| true), false).unwrap();
                black_box(p)
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("parse_and_query", |b| {
        b.iter_batched(
            snailx::indexing_parser::IndexingParser::new,
            |mut p| {
                p.parse(black_box(RULES), ..usize::MAX, &[], black_box(|_| true), false).unwrap();
                black_box(p.positional(0));
                black_box(p.flag("alpha"));
                black_box(p.flag("beta"));
                if let Ok(it) = p.option("num") {
                    let _ = black_box(black_box(it).next());
                }
                if let Ok(it2) = p.option("gamma") {
                    let _ = black_box(black_box(it2).nth(1));
                }
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

#[cfg(feature = "indexing_parser")]
fn bench_indexing_parser_preset_cmdline(c: &mut Criterion) {
    unsafe {
        snailx::direct::set_argc_argv(
            ARGV_PRESET_CMDLINE.len() as u32,
            ARGV_PRESET_CMDLINE.as_ptr()
        )
    };

    let mut group = c.benchmark_group("indexing_parser/preset_cmdline");

    group.bench_function("parse_only", |b| {
        b.iter_batched(
            snailx::indexing_parser::IndexingParser::new,
            |mut p| {
                p.parse(black_box(RULES), ..usize::MAX, &[], black_box(|_| true), false).unwrap();
                black_box(p)
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("parse_and_query", |b| {
        b.iter_batched(
            snailx::indexing_parser::IndexingParser::new,
            |mut p| {
                p.parse(black_box(RULES), ..usize::MAX, &[], black_box(|_| true), false).unwrap();
                black_box(p.positional(0));
                black_box(p.flag("alpha"));
                black_box(p.flag("beta"));
                if let Ok(it) = p.option("num") {
                    let _ = black_box(black_box(it).next());
                }
                if let Ok(it2) = p.option("gamma") {
                    let _ = black_box(black_box(it2).nth(1));
                }
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

#[cfg(feature = "indexing_parser")]
fn bench_indexing_parser_long(c: &mut Criterion) {
    unsafe {
        snailx::direct::set_argc_argv(ARGV_PRESET_LONG.len() as u32, ARGV_PRESET_LONG.as_ptr())
    };

    let mut group = c.benchmark_group("indexing_parser/long");

    group.bench_function("parse_only", |b| {
        b.iter_batched(
            snailx::indexing_parser::IndexingParser::new,
            |mut p| {
                p.parse(black_box(RULES), ..usize::MAX, &[], black_box(|_| true), false).unwrap();
                black_box(p)
            },
            BatchSize::SmallInput
        );
    });

    group.bench_function("parse_and_query", |b| {
        b.iter_batched(
            snailx::indexing_parser::IndexingParser::new,
            |mut p| {
                p.parse(black_box(RULES), ..usize::MAX, &[], black_box(|_| true), false).unwrap();
                black_box(p.positional(0));
                black_box(p.flag("alpha"));
                black_box(p.flag("beta"));
                if let Ok(it) = p.option("num") {
                    let _ = black_box(black_box(it).next());
                }
                if let Ok(it2) = p.option("gamma") {
                    let _ = black_box(black_box(it2).nth(1));
                }
            },
            BatchSize::SmallInput
        );
    });

    group.finish();
}

pub fn bench(c: &mut Criterion) {
    bench_snailx_iter_minimal(c);
    #[cfg(feature = "rev_iter")]
    bench_snailx_iter_back_minimal(c);

    bench_snailx_nth_minimal(c);
    #[cfg(feature = "rev_iter")]
    bench_snailx_nth_back_minimal(c);

    bench_snailx_fold_minimal(c);
    #[cfg(feature = "rev_iter")]
    bench_snailx_rfold_minimal(c);

    #[cfg(feature = "std")]
    {
        bench_iter_snailx_vs_std(c);
        #[cfg(feature = "rev_iter")]
        bench_iter_back_snailx_vs_std(c);

        bench_nth_snailx_vs_std(c);
        #[cfg(feature = "rev_iter")]
        bench_nth_back_snailx_vs_std(c);

        bench_fold_snailx_vs_std(c);
    }

    bench_snailx_iter_preset(c);
    #[cfg(feature = "rev_iter")]
    bench_snailx_iter_back_preset(c);

    bench_snailx_nth_preset(c);
    #[cfg(feature = "rev_iter")]
    bench_snailx_nth_back_preset(c);

    bench_snailx_fold_preset(c);
    #[cfg(feature = "rev_iter")]
    bench_snailx_rfold_preset(c);

    #[cfg(feature = "indexing_parser")]
    {
        bench_indexing_parser_minimal(c);
        bench_indexing_parser_preset_cmdline(c);
        bench_indexing_parser_long(c);
    }

    bench_snailx_helpers(c);
}

fn main() {
    let mut crit: Criterion<_> = Criterion::default()
        .sample_size(800) // default is 100; use a bit larger for stability
        .measurement_time(Duration::from_secs(16))
        .warm_up_time(Duration::from_secs(4))
        .nresamples(800_000) // default is 100_000; increase for tighter CIs
        .noise_threshold(0.0001) // treat changes below 0.01% as noise
        .confidence_level(0.9999) // tighter confidence interval
        .configure_from_args();

    bench(&mut crit);

    crit.final_summary();
}
