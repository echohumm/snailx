//! `snailx` provides a simple, zero-allocation interface for iterating over the arguments of a
//! program.
//! 
//! This crate exposes lightweight, zero-copy iterators over program arguments:
//! - [`args`] yields `&'static core::ffi::CStr`
//! - [`str_args`] yields `&'static str`
//!   - if the `assume_valid_str` feature is enabled, all arguments are assumed to be valid UTF-8
//!   - if the `assume_valid_str` feature is disabled, invalid UTF-8 arguments are skipped
//! - [`arg_ptrs`] returns the raw argv as `&'static [*const u8]`
//! - [`map_args`] lets you map each `&'static CStr` into a custom type; `None` values are skipped
//! - [`osstr_args`] (with the `std` feature) yields `&'static std::ffi::OsStr`
//!
//! `no_std` by default; enable the `std` feature for `OsStr` support.
//! Targets Unix-like systems and macOS.

// yes, this crate is *technically* no_std.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::use_self, clippy::similar_names, clippy::cast_lossless, clippy::doc_markdown)]
#![deny(missing_docs)]
extern crate core;

// TODO: cleanup attrs, docs

macro_rules! assume {
    // completely unreachable branches
    (!$e:expr) => {
        if $e {
            #[allow(unused_unsafe)]
            unsafe {
                core::hint::unreachable_unchecked();
            }
        }
    };
    ($e:expr) => {
        if !$e {
            #[allow(unused_unsafe)]
            unsafe {
                core::hint::unreachable_unchecked();
            }
        }
    };
    (const $e:expr) => {
        const {
            if !$e {
                #[allow(unused_unsafe)]
                unsafe {
                    core::hint::unreachable_unchecked();
                }
            }
        }
    };
    // potentially reachable branch with default message
    (re, $e:expr) => {
        assume!($e, "entered unreachable code");
    };
    // potentially reachable with message, only check in debug
    (dbg, $e:expr, $($msg:tt)+) => {
        #[cfg(debug_assertions)]
        if !$e {
            panic!($($msg)+);
        }
    };
    // potentially reachable branch with message
    ($e:expr, $($msg:tt)+) => {
        if !$e {
            #[cfg(debug_assertions)]
            {
                panic!($($msg)+);
            }
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

#[cfg(any(debug_assertions, not(feature = "assume_valid_str")))]
mod str_checks;

pub use {
    cmdline::*,
    iter::{args::*, mapped_args::*}
};

// TODO: tests
