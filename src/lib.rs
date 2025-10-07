//! `snailx` provides a simple, zero-allocation interface for iterating over the arguments of a
//! program.
//! 
//! This crate exposes lightweight, zero-copy iterators over program arguments:
//! - [`args`] yields `&'static core::ffi::CStr`
//! - [`str_args`] yields `&'static str` (non-UTF-8 arguments are skipped)
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
    // potentially reachable branch with default message
    (re, $e:expr) => {
        assume!($e, "entered unreachable code");
    };
    // potentially reachable branch with message
    ($e:expr, $($msg:tt)+) => {
        let e = $e;
        #[cfg(debug_assertions)]
        {
            assert!(e, $($msg)+);
        }
        if !e {
            #[allow(unused_unsafe)]
            unsafe {
                core::hint::unreachable_unchecked();
            }
        }
    }
}

macro_rules! common_iter_methods {
    ($item:ty) => {
        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            sz_hnt(self.cur, self.end)
        }

        #[inline]
        fn count(self) -> usize {
            self.len()
        }

        #[inline]
        fn last(mut self) -> Option<$item> {
            self.next_back()
        }

        #[inline]
        fn for_each<G: FnMut($item)>(mut self, mut f: G) {
            while let Some(x) = self.next() {
                f(x);
            }
        }

        #[inline]
        fn all<G: FnMut($item) -> bool>(&mut self, mut f: G) -> bool {
            while let Some(x) = self.next() {
                if !f(x) {
                    return false;
                }
            }
            true
        }

        #[inline]
        fn any<G: FnMut($item) -> bool>(&mut self, mut f: G) -> bool {
            while let Some(x) = self.next() {
                if f(x) {
                    return true;
                }
            }
            false
        }

        #[inline]
        fn find<P: FnMut(&$item) -> bool>(&mut self, mut predicate: P) -> Option<$item> {
            while let Some(x) = self.next() {
                if predicate(&x) {
                    return Some(x);
                }
            }
            None
        }

        #[inline]
        fn find_map<B, G: FnMut($item) -> Option<B>>(&mut self, mut f: G) -> Option<B> {
            while let Some(x) = self.next() {
                if let Some(y) = f(x) {
                    return Some(y);
                }
            }
            None
        }

        #[inline]
        fn position<P: FnMut($item) -> bool>(&mut self, mut predicate: P) -> Option<usize> {
            let n = self.len();
            let mut i = 0;
            while let Some(x) = self.next() {
                if predicate(x) {
                    assume!(i < n);
                    return Some(i);
                }
                unsafe {
                    crate::iter::helpers::unchecked_add(&mut i, 1);
                }
            }
            None
        }

        #[inline]
        fn rposition<P: FnMut(<Self as Iterator>::Item) -> bool>(
            &mut self,
            mut predicate: P
        ) -> Option<usize> {
            let n = self.len();
            let mut i = n;
            while let Some(x) = self.next_back() {
                unsafe {
                    crate::iter::helpers::unchecked_sub(&mut i, 1);
                }
                if predicate(x) {
                    assume!(i < n);
                    return Some(i);
                }
            }
            None
        }
    };
}

pub mod direct;
mod ffi;

mod cmdline;
mod iter;

pub use {
    cmdline::*,
    iter::{args::*, mapped_args::*}
};

// TODO: tests
