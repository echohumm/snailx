//! `snailx` provides a simple, zero-allocation interface for iterating over program arguments.
//!
//! This crate exposes lightweight, zero-copy iterators over program arguments:
//! - [`Args::new`](Args::new) yields <code>[CStr]<'static></code>
//! - [`MappedArgs::utf8`] yields `&'static str`
//!   - if the `assume_valid_str` feature is enabled, all arguments are assumed to be valid UTF-8
//!   - if the `assume_valid_str` feature is disabled, invalid UTF-8 arguments are skipped
//! - [`direct::argc_argv`] returns the raw `(argc, argv)`
//! - [`MappedArgs::new`] lets you map each `*const u8` argument pointer into a custom type; `None`
//!   values are skipped
//! - [`MappedArgs::osstr`] (with the `std` feature) yields `&'static std::ffi::OsStr`
//!
//! `no_std` by default; enable the `std` feature for `OsStr` support.
//! Targets Unix-like systems and macOS.

// TODO: make sure every iterator method we impl has tests + benches (fold, rfold, count, last)

#![cfg_attr(not(feature = "std"), no_std)]
#![no_implicit_prelude]
#![deny(missing_docs)]
#![allow(clippy::use_self, clippy::similar_names, clippy::cast_lossless, clippy::doc_markdown)]

#[cfg(feature = "alloc")] extern crate alloc;

#[cfg(not(any(unix, target_vendor = "apple")))]
compile_error!("snailx only supports Unix and macOS");

macro_rules! import {
    ($($v:tt)*) => {
        #[cfg(feature = "std")]
        use std::$($v)*;
        #[cfg(not(feature = "std"))]
        use core::$($v)*;
    };
}

#[cfg_attr(feature = "__bench", macro_export)]
/// helper macro to switch between `std` and `core` based on whether `no_std` is on.
macro_rules! switch {
    (core::$($v:tt)*) => {{
        #[cfg(feature = "std")]
        {
            ::std::$($v)*
        }
        #[cfg(not(feature = "std"))]
        {
            ::core::$($v)*
        }
    }};
}

macro_rules! assume {
    // completely unreachable branches
    // assumes expression is false
    (!$e:expr) => {
        if $e {
            // SAFETY: this is unreachable
            #[allow(unused_unsafe)]
            unsafe {
                switch!(core::hint::unreachable_unchecked(););
            }
        }
    };

    // assumes expression is true
    ($e:expr) => {
        if !$e {
            // SAFETY: this is unreachable
            #[allow(unused_unsafe)]
            unsafe {
                switch!(core::hint::unreachable_unchecked(););
            }
        }
    };
    // potentially reachable branch with default message
    (re, $e:expr) => {
        assume!($e, "entered unreachable code");
    };

    // debug-only check with custom message
    (dbg, $e:expr, $($msg:tt)+) => {
        #[cfg(debug_assertions)]
        if !$e {
            panic!($($msg)+);
        }
    };

    // carry out
    (car, $exp:ident, $in_name:ident, $e:expr, $($msg:tt)+) => {
        match $e {
            $exp($in_name) => $in_name,
            #[allow(unused_unsafe)]
            _ => unsafe { switch!(core::hint::unreachable_unchecked();) },
        }
    };

    // custom message for both debug and release
    ($e:expr, $($msg:tt)+) => {
        if !$e {
            #[cfg(debug_assertions)]
            {
                panic!($($msg)+);
            }
            // SAFETY: guarded in debug by the above, UB in release builds if used improperly
            #[cfg(not(debug_assertions))]
            #[allow(unused_unsafe)]
            unsafe {
                switch!(core::hint::unreachable_unchecked(););
            }
        }
    };
}

pub mod direct;
mod ffi;

mod cmdline;
mod iter;

#[cfg(any(feature = "indexing_parser", feature = "non_indexing_parser"))] mod parser;

#[cfg(feature = "indexing_parser")] pub use parser::indexing as indexing_parser;
#[cfg(feature = "non_indexing_parser")] pub use parser::non_indexing as non_indexing_parser;

#[cfg(any(debug_assertions, not(feature = "assume_valid_str")))] mod str_checks;

pub use {
    ffi::minimal_cstr::CStr,
    iter::{args::*, mapped_args::*}
};
#[cfg(any(feature = "to_core_cstr", feature = "std"))]
pub use ffi::minimal_cstr::StdCStr;

#[cfg(feature = "__bench")]
#[allow(missing_docs)]
#[doc(hidden)]
pub mod bench_helpers {
    pub use {cmdline::helpers::*, ffi::strlen, iter::helpers::len};
}
