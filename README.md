# snailx

## General information

- MSRV: 1.48.0
    - Only benchmarked on latest stable and nightly.
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
use snailx::args;

fn main() {
    // Iterate over arguments, skipping the first one.
    // Because `snailx` uses its own, minimal and intermediary `CStr` type, it must be converted to a `std::ffi::CStr` 
    // before usage. This behavior is planned to be improved in the future.
    let args = args().skip(1).filter_map(|arg| arg.to_stdlib().to_str().ok());
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

## API

### Functions

- `args() -> Args` - The basic iterator over the program arguments as `snailx::CStr<'static>`
- `args_os() -> MappedArgs<&'static OsStr, fn(*const u8) -> Option<&'static std::ffi::OsStr>` - Iterator over the
  program arguments as `&'static std::ffi::OsStr`
- `args_str() -> MappedArgs<&'static str, fn(*const u8) -> Option<&'static str>` - Iterator over the program arguments
  as `&'static str`
- `map_args<T, F: Fn(*const u8) -> Option<T> + Copy + 'static>(map: F)` - Iterator over the program arguments as `T`
- `arg_ptrs() -> &'static [*const u8]` - Slice of pointers to the program arguments. Returns a direct slice of argv
- `args_slice() -> &'static [CStr<'static>]` - Slice of the program arguments. This function's safety is a primary
  reason for the existence of `snailx::CStr`

### Feature flags

- `std` - Enables the `std` feature, which enables all functions relating to `OsStr` and is one way to enable
  `snailx::CStr::to_stdlib`
- `no_cold` - Removes the `#[cold]` attribute from several functions
- `to_core_cstr` (MSRV 1.64.0) - Enables `snailx::CStr::to_stdlib`
- `assume_valid_str` - This massively speeds up the iterator returned by `args_str()` by disabling validity checks, but
  can cause UB if the program arguments are invalid UTF-8. Use disrecommended unless you can guarantee the returned
  `&'static str`s will be used safely or invalid UTF-8 will never be used.

### Types

[//]: # (TODO: performance and benchmarks)

- `Args` - Iterator over program arguments as `snailx::CStr<'static>`
- `MappedArgs<T, F>` - Generic iterator that applies a mapping function to each argument
- `CStr<'static>` - Minimal C-style string type for zero-allocation argument access. This exists to make `args_slice()`
  safe and because this crate is `no_std`, but `core_cstr` was stabilized after its MSRV.

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
use snailx::args;

fn main() {
    for (i, arg) in args().enumerate() {
        println!("Argument {}: {:?}", i, arg);
    }
}
```

### String arguments with error handling

```rust
use snailx::args_str;

fn main() {
    for arg in args_str() {
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
use snailx::map_args;

fn main() {
    let lengths: Vec<usize> = map_args(|ptr| {
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
