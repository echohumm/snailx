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

// TODO: make sure every iterator and parser method we impl has tests + benches
// TODO: break up big files, types, and modules

#![cfg_attr(not(feature = "std"), no_std)]
#![no_implicit_prelude]
#![deny(missing_docs)]
#![allow(clippy::use_self, clippy::similar_names, clippy::cast_lossless, clippy::doc_markdown)]

#[cfg(feature = "alloc")] extern crate alloc;
extern crate core;

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
    // assumes expression is absolutely false
    (!$e:expr) => {
        if $e {
            // SAFETY: this is unreachable
            #[allow(unused_unsafe)]
            unsafe {
                switch!(core::hint::unreachable_unchecked(););
            }
        }
    };

    // assumes expression is absolutely true
    ($e:expr) => {
        if !$e {
            // SAFETY: this is unreachable
            #[allow(unused_unsafe)]
            unsafe {
                switch!(core::hint::unreachable_unchecked(););
            }
        }
    };

    // debug-only check with custom message
    (dbg, $e:expr, $($msg:tt)+) => {
        #[cfg(debug_assertions)]
        if !$e {
            panic!($($msg)+);
        }
    };

    // carry out (same as [Option,Result]::unwrap_unchecked())
    (car, $exp:ident, $in_name:ident, $e:expr, $($msg:tt)+) => {
        match $e {
            $exp($in_name) => $in_name,
            #[allow(unused_unsafe)]
            _ => unsafe { switch!(core::hint::unreachable_unchecked();) },
        }
    };

    // custom message for both debug and release (similar to debug_assert)
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

mod iter;

#[cfg(any(feature = "indexing_parser", feature = "non_indexing_parser"))] mod parser;

#[cfg(feature = "indexing_parser")] pub use parser::indexing as indexing_parser;
#[cfg(feature = "non_indexing_parser")] pub use parser::non_indexing as non_indexing_parser;

#[cfg(any(debug_assertions, not(feature = "assume_valid_str")))] mod str_checks;

#[cfg(any(feature = "to_core_cstr", feature = "std"))] pub use ffi::minimal_cstr::StdCStr;
pub use {
    ffi::minimal_cstr::CStr,
    iter::{args::*, mapped_args::*}
};

#[cfg(feature = "__bench")]
#[allow(missing_docs)]
#[doc(hidden)]
pub mod bench_helpers {
    pub use {ffi::strlen, helpers::*, iter::len};
}

mod helpers {
    import! {
        {
            mem::transmute,
            option::Option::{self, Some},
            slice
        }
    }
    use crate::ffi::strlen;

    #[inline]
    #[allow(
        clippy::must_use_candidate,
        clippy::not_unsafe_ptr_arg_deref,
        clippy::transmute_bytes_to_str,
        missing_docs
    )]
    pub fn try_to_str(p: *const u8) -> Option<&'static str> {
        // SAFETY: only called internally with valid CStr pointers from argv
        unsafe {
            assume!(!p.is_null());
            let len = strlen(p.cast());
            let bytes = slice::from_raw_parts(p, len + 1);
            assume!(
                !bytes.is_empty() && bytes[len] == 0,
                "`try_to_str`: CStr does not end with null byte"
            );

            let str_bytes = slice::from_raw_parts(p, len);

            #[cfg(not(feature = "assume_valid_str"))]
            if crate::str_checks::is_valid_utf8(str_bytes) {
                Some(transmute::<&'static [u8], &'static str>(str_bytes))
            } else {
                switch!(core::option::Option::None)
            }

            #[cfg(feature = "assume_valid_str")]
            {
                assume!(
                    dbg,
                    crate::str_checks::is_valid_utf8(str_bytes),
                    "invalid UTF-8 in CStr during conversion to str"
                );
                Some(transmute::<&'static [u8], &'static str>(str_bytes))
            }
        }
    }

    #[cfg(feature = "std")]
    #[inline]
    #[allow(
        clippy::unnecessary_wraps,
        clippy::must_use_candidate,
        missing_docs,
        unused_qualifications
    )]
    pub fn to_osstr(p: *const u8) -> Option<&'static ::std::ffi::OsStr> {
        // SAFETY: only called internally with valid CStr pointers from argv
        unsafe {
            assume!(!p.is_null());
            let len = strlen(p.cast());
            assume!(!len == 0);
            Some(&*(switch!(core::ptr::slice_from_raw_parts(p, len)) as *const ::std::ffi::OsStr))
        }
    }

    #[cfg(any(feature = "std", feature = "to_core_cstr"))]
    #[inline]
    #[allow(clippy::must_use_candidate, clippy::not_unsafe_ptr_arg_deref, missing_docs)]
    pub fn to_stdcstr(p: *const u8) -> Option<&'static crate::StdCStr> {
        Some(unsafe { crate::CStr::from_ptr(p).to_stdlib() })
    }

    #[allow(
        clippy::inline_always,
        clippy::must_use_candidate,
        clippy::not_unsafe_ptr_arg_deref,
        missing_docs
    )]
    #[inline(always)]
    #[cfg_attr(not(feature = "no_cold"), cold)]
    pub fn back(argv: *const *const u8, argc: u32) -> *const *const u8 {
        assume!(!argv.is_null(), "`back`: argv is null");
        // SAFETY: argv points to a valid slice of argc count pointers, this is one past the last
        // but always decremented before deref
        unsafe { argv.add(argc as usize) }
    }
}
