# snailx

## General information

- MSRV: 1.48.0
    - Only benchmarked on latest stable and nightly.
    - Tests require `-- --test-threads=1` on earlier rustc versions to run without race conditions causing spurious
      failures.
        - These are spurious as it is a documented safety condition of `snailx::direct::set_argc_argv()` that there is
          no
          concurrent access.
- Licenses: GPL-3.0, MIT

## Overview

`snailx` provides a simple, zero-allocation, zero-dependency API for iterating over program arguments on Unix and MacOS.

### Benefits

- No allocations necessary
    - Theoretically faster than `std::env::args()`, but optimization is in its early stages.
- Small code and binary size

### Downsides

- No Windows support - While Windows support is planned for the future, it is not currently a priority, and use would
  require allocation.
- Unstable API - The API and entire crate is still undergoing heavy changes, and speed, APIs, and other aspects may
  change.

### Example usage

```rust
use snailx::Args;

fn main() {
    // Iterate over arguments, skipping the first one.
    // Because `snailx` uses its own, minimal and intermediary `CStr` type, it must be converted to a `std::ffi::CStr` 
    // before usage. This behavior is planned to be improved in the future.
    let args = Args::new().skip(1).filter_map(|arg| arg.to_stdlib().to_str().ok());
    match args.next() {
        Some("run") => println!("Running"),
        Some("build") => println!("Building"),
        Some(_) => println!("Unknown subcommand"),
        None => println!("No subcommand provided"),
    }
}
```

## Features

- Zero-allocation argument access
- Iterators over multiple string types
    - CStr
    - OsStr
    - str
- `no_std` support <small>technically</small>
- Better performance <small>(WIP)</small>
    - You can, under most circumstances, expect `snailx` iterators to be at least twice as fast as `std::env::args()`,
      but you should benchmark yourself. In certain cases, `snailx` is up to 6x faster than stdlib, but much slower in
      others.

## API

### Functions

- `Args::new() -> Args` - The basic iterator over the program arguments as `snailx::CStr<'static>`
- `MappedArgs::osstr() -> MappedArgs<&'static OsStr, fn(*const u8) -> Option<&'static std::ffi::OsStr>` - Iterator over
  the program arguments as `&'static std::ffi::OsStr`
- `MappedArgs::utf8() -> MappedArgs<&'static str, fn(*const u8) -> Option<&'static str>` - Iterator over the program
  arguments as `&'static str`
- `MappedArgs::new<T, F: Fn(*const u8) -> Option<T>>(map: F)` - Iterator over the program arguments as `T`
- `direct::argc_argv() -> (u32, *const *const u8)` - Raw access to `(argc, argv)`

### Feature flags

- `std` - Enables the `std` feature, which enables all functions relating to `OsStr` and is one way to enable
  `snailx::CStr::to_stdlib`
- `no_cold` - Removes the `#[cold]` attribute from several functions
- `to_core_cstr` (MSRV 1.64.0) - Enables `snailx::CStr::to_stdlib`
- `assume_valid_str` - This massively speeds up the iterator returned by `MappedArgs::utf8()` by disabling validity
  checks, but can cause UB if the program arguments are invalid UTF-8. Use disrecommended unless you can guarantee the 
  returned `&'static str`s will be used safely or invalid UTF-8 will never be used.

### Types

[//]: # (TODO: performance and benchmarks)

- `Args` - Iterator over program arguments as `snailx::CStr<'static>`
- `MappedArgs<T, F>` - Generic iterator that applies a mapping function to each argument
- `CStr<'static>` - Minimal C-style string type for zero-allocation argument access. This exists because this crate is
  `no_std`, but `core_cstr` was stabilized after its MSRV.

## Platform support

- Unix-like systems: Fully supported
    - Linux with GNU: Fully supported and tested
    - Other variants: Fully supported in theory but untested
- macOS: Fully supported but untested
- Windows: Not yet supported (planned for future releases)

### Platform-specific notes

- GNU vs non-GNU: The distinction refers to whether the system uses GNU libc (glibc) or alternative C libraries (musl,
  uClibc, etc.). `snailx` works on both in theory but is tested with glibc.

## Safety

`snailx` uses `unsafe` code to access OS-provided argument storage. The safety guarantees are:

- Arguments are read-only and never modified
- All pointer arithmetic is bounds-checked
- UTF-8 validation is performed unless `assume_valid_str` is enabled
- The `assume_valid_str` feature trades safety for performance and should only be used when you can guarantee valid
  UTF-8 input

## Examples

### Basic argument iteration

```rust
use snailx::Args;

fn main() {
    for (i, arg) in Args::new().enumerate() {
        println!("Argument {}: {:?}", i, arg);
    }
}
```

### String arguments with error handling

```rust
use snailx::MappedArgs;

fn main() {
    for arg in MappedArgs::utf8() {
        match arg {
            "help" => println!("Usage: ..."),
            "version" => println!("Version 0.1.0"),
            other => println!("Unknown argument: {}", other),
        }
    }
}
```

### Custom argument mapping

```rust
use snailx::MappedArgs;

fn main() {
    // alternatively, if `infallible_map` is enabled, you can use `MappedArgs::new_infallible()` if you want 
    // `size_hint` to return an accurate lower bound.
    let lengths: Vec<usize> = MappedArgs::new(|ptr| {
        unsafe {
            // simple strlen implementation
            let mut i = 0;
            while ptr.add(i) != 0 {
                i += 1;
            }
            Some(i)
        }
    }).collect();

    println!("Argument lengths: {:?}", lengths);
}
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## License

`snailx` is dual licensed under GPLv3 and MIT.
