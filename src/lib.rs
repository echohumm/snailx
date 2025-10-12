//! `snailx` provides a simple, zero-allocation interface for iterating over program arguments.
//!
//! This crate exposes lightweight, zero-copy iterators over program arguments:
//! - [`args`] yields <code>[CStr](CStr)<'static></code>
//! - [`str_args`] yields `&'static str`
//!   - if the `assume_valid_str` feature is enabled, all arguments are assumed to be valid UTF-8
//!   - if the `assume_valid_str` feature is disabled, invalid UTF-8 arguments are skipped
//! - [`arg_ptrs`] returns the raw argv as `&'static [*const u8]`
//! - [`map_args`] lets you map each `*const u8` argument pointer into a custom type; `None` values
//!   are skipped
//! - [`osstr_args`] (with the `std` feature) yields `&'static std::ffi::OsStr`
//!
//! `no_std` by default; enable the `std` feature for `OsStr` support.
//! Targets Unix-like systems and macOS.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![allow(clippy::use_self, clippy::similar_names, clippy::cast_lossless, clippy::doc_markdown)]
extern crate core;

// TODO: see how many of these are actually useful
macro_rules! assume {
    // completely unreachable branches
    // assumes expression is false
    (!$e:expr) => {
        if $e {
            // SAFETY: this is unreachable
            #[allow(unused_unsafe)]
            unsafe {
                core::hint::unreachable_unchecked();
            }
        }
    };
    
    // assumes expression is true
    ($e:expr) => {
        if !$e {
            // SAFETY: this is unreachable
            #[allow(unused_unsafe)]
            unsafe {
                core::hint::unreachable_unchecked();
            }
        }
    };
    // potentially-reachable branch with default message
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
                core::hint::unreachable_unchecked();
            }
        }
    };
}

pub mod direct;
mod ffi;

mod cmdline;
mod iter;

#[cfg(any(debug_assertions, not(feature = "assume_valid_str")))] mod str_checks;

pub use {
    cmdline::*,
    ffi::minimal_cstr::CStr,
    iter::{args::*, mapped_args::*}
};

#[cfg(feature = "bench")]
#[allow(missing_docs)]
#[doc(hidden)]
pub mod bench_helpers {
    pub use crate::{cmdline::helpers::try_to_str, iter::helpers::len};
}

// TODO: tests
