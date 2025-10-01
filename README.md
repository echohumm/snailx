# snail

Zero-allocation, low-overhead access to program arguments (`argv`) with iterators over `&'static CStr`, `&'static str`, and (with the `std` feature) `&'static OsStr`. Works in `no_std` (optional `std` feature) and targets Unix and macOS.

---

## Status

* **MSRV:** 1.64.0
* **Platforms:** Unix-like (including macOS). Windows not yet supported.
  * If Windows is ever supported, it will not be zero-allocation.
* **Allocation:** None — iteration is over the process `argv` memory exposed by the platform.

### Future plans

* Lower MSRV.
* Two argument parsers:
  * a non-indexing, zero-allocation parser
  * an indexing, allocating parser
* Possibly: Windows and other platform support

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
snail = "0.5"
```

Enable `std` adapters (e.g. `OsStr`) with the `std` feature:

```toml
[dependencies]
snail = { version = "0.5", features = ["std"] }
```

The crate is `#![no_std]` when `std` is not enabled. On Linux and macOS, `snail` can read arguments automatically from 
the runtime. On other platforms (or unusual runtimes) you may need to arrange for the C `main` to call into Rust so the 
raw `argc, argv` are handed to `snail::direct::set_argc_argv`.

---

## Quick start (examples)

Iterate as `&'static CStr`:

```rust
for c in snail::args() {
    println!("{:?}", c);
}
```

Iterate as `&'static std::ffi::OsStr` (requires `std` feature):

```rust
for o in snail::osargs() {
    println!("{:?}", o);
}
```

Iterate as `&'static str` (skips invalid UTF-8):

```rust
for s in snail::str_args() {
    println!("{}", s);
}
```

Map/filter arguments with custom converter:

```rust
# use core::ffi::CStr;

let iter = snail::map_args(|c: &'static CStr| {
    c.to_str().ok().filter(|s| s.starts_with("--"))
});

for flag in iter {
    println!("flag: {}", flag);
}
```

Get raw argv pointers:

```rust
let ptrs: &'static [*const u8] = snail::arg_ptrs();
println!("argc = {}", ptrs.len());
```

---

## Design notes

* Exposes lightweight iterators (no allocation, no copying).
* Unsafe internals are encapsulated; public API is safe on supported targets.
* Iterators are double-ended and implement `ExactSizeIterator` and `FusedIterator` where applicable.

## Feature flags

* `std` — adapters that return `&'static OsStr` and other conveniences; crate remains usable without it.

## License

Dual-licensed:

* Apache License, Version 2.0
* MIT License

## Contribution

By contributing you agree to dual-license your contributions as above (unless you explicitly state otherwise).

---
