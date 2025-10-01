//! `snail` provides a simple, zero-allocation interface for iterating over the arguments of a
//! program.
//!
//! Use [`Args`] to iterate over the program's arguments. If you wish to iterate over the arguments
//! as <code>&'static [CStr](core::ffi::CStr)</code>s, use either the [`IntoIterator`]
//! implementation or the [`Args::iter`] method.
//!
//! If you wish to iterate over the arguments as `&'static str`s, use [`Args::iter_str`].

// yes, this crate is *technically* no_std.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::use_self, clippy::similar_names, clippy::cast_lossless)]
#![deny(missing_docs)]

macro_rules! assume {
    (!$e:expr) => {
        if $e {
            #[allow(unused_unsafe)]
            unsafe {
                core::hint::unreachable_unchecked();
            }
        }
    };
    // completely unreachable branch
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

mod cmdline;
mod iter;


pub use {
    cmdline::*,
    iter::{args::*, mapped_args::*},
};

// TODO: tests
