// TODO: optimize this in general
#![allow(clippy::while_let_on_iterator)]

use {
    crate::{CStr, MappedArgs, cmdline::helpers::try_to_str, iter::helpers::len},
    cmdline::helpers,
    direct
};

import! {
    use core::{
        iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator, Iterator},
        ops::{Fn},
        option::Option::{self, None, Some},
        default::Default
    }
}

// not Copy because that nets a 2-5% performance improvement for some reason
/// An iterator over program arguments as <code>[CStr](CStr)<'static></code>.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Args {
    pub(crate) cur: *const *const u8,
    pub(crate) end: *const *const u8
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

impl Args {
    /// Creates a new `Args` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![cfg(feature = "std")]
    /// # use snailx::Args;
    ///
    /// for arg in Args::new().map(|v| v.to_stdlib()) {
    ///     println!("{}", arg.to_string_lossy());
    /// }
    /// ```
    #[must_use]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    // cold because these are usually called once at startup
    #[cfg_attr(not(feature = "no_cold"), cold)]
    pub fn new() -> Args {
        let (argc, argv) = direct::argc_argv();
        Args { cur: argv, end: helpers::back(argv, argc) }
    }

    /// Gets the remaining arguments in this iterator as a slice.
    #[must_use]
    pub fn as_slice(&self) -> &'static [CStr<'static>] {
        unsafe {
            switch!(core::slice::from_raw_parts(
                self.cur.cast::<CStr<'static>>(),
                len(self.cur, self.end)
            ))
        }
    }

    /// Map this iterator to a different type. Like [`MappedArgs::new`](crate::MappedArgs::new), but
    /// operates on an existing iterator.
    #[must_use]
    pub fn map_ty<Ret, F: Fn(*const u8) -> Option<Ret>>(&self, map: F) -> MappedArgs<Ret, F> {
        MappedArgs {
            cur: self.cur,
            end: self.end,
            map,
            // assume fallible for safety
            #[cfg(feature = "infallible_map")]
            fallible: true
        }
    }

    #[cfg(feature = "infallible_map")]
    /// Map this iterator to a different type. Like
    /// [`MappedArgs::new_infallible`](crate::MappedArgs::new_infallible), but operates on an
    /// existing iterator.
    #[must_use]
    pub fn map_ty_infallible<Ret, F: Fn(*const u8) -> Option<Ret>>(
        &self,
        map: F
    ) -> MappedArgs<Ret, F> {
        MappedArgs { cur: self.cur, end: self.end, map, fallible: false }
    }

    /// Map this iterator to `&'static str`. Like [`MappedArgs::utf8`](crate::MappedArgs::utf8), but
    /// operates on an existing iterator. Non-UTF-8 arguments are skipped.
    #[must_use]
    pub fn map_str(&self) -> MappedArgs<&'static str, fn(*const u8) -> Option<&'static str>> {
        MappedArgs {
            cur: self.cur,
            end: self.end,
            map: try_to_str,
            #[cfg(all(feature = "infallible_map", not(feature = "assume_valid_str")))]
            fallible: true,
            // assume_valid_str makes the map "infallible"
            #[cfg(all(feature = "infallible_map", feature = "assume_valid_str"))]
            fallible: false
        }
    }

    #[cfg(feature = "std")]
    /// Map this iterator to `&'static OsStr`. Like [`MappedArgs::osstr`](crate::MappedArgs::osstr),
    /// but operates on an existing iterator.
    #[must_use]
    pub fn map_os(
        &self
    ) -> MappedArgs<&'static ::std::ffi::OsStr, fn(*const u8) -> Option<&'static ::std::ffi::OsStr>>
    {
        MappedArgs {
            cur: self.cur,
            end: self.end,
            map: helpers::to_osstr,
            #[cfg(feature = "infallible_map")]
            fallible: false
        }
    }
}

// most of these are copied or slightly adapted from slice::Iter
impl Iterator for Args {
    type Item = CStr<'static>;

    // inline(always) nets a 5% performance loss. no inlining nets a 70% loss. normal inlining is
    // good.
    #[inline]
    fn next(&mut self) -> Option<CStr<'static>> {
        if self.cur == self.end {
            return None;
        }

        // SAFETY: we just checked that `self.cur < self.end`
        let p = self.cur;
        self.cur = unsafe { self.cur.add(1) };

        assume!(p < self.end);

        // SAFETY: the pointer is from argv, which always contains valid pointers to cstrs
        Some(unsafe { CStr::from_ptr(p.read()) })
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = unsafe { len(self.cur, self.end) };
        (len, Some(len))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<CStr<'static>> {
        if n >= self.len() {
            self.cur = self.end;
            return None;
        }

        // SAFETY: we just checked that `self.cur + n` is in bounds
        let p = unsafe { self.cur.add(n) };
        self.cur = unsafe { p.add(1) };

        assume!(!self.cur.is_null());
        assume!(!p.is_null() && p < self.end);

        // SAFETY: the pointer is from argv, which always contains valid pointers to cstrs
        Some(unsafe { CStr::from_ptr(p.read()) })
    }
}

impl DoubleEndedIterator for Args {
    #[inline]
    fn next_back(&mut self) -> Option<CStr<'static>> {
        if self.cur == self.end {
            return None;
        }

        // SAFETY: we just checked that `self.cur < self.end`
        self.end = unsafe { self.end.sub(1) };
        assume!(self.end > self.cur);
        
        // SAFETY: the pointer is from argv, which always contains valid pointers to cstrs
        Some(unsafe { CStr::from_ptr(self.end.read()) })
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<CStr<'static>> {
        if n >= self.len() {
            self.end = self.cur;
            return None;
        }

        // SAFETY: we just checked that `self.end - n` is in bounds
        self.end = unsafe { self.end.sub(n + 1) };
        assume!(!self.end.is_null() && self.end > self.cur);

        // SAFETY: the pointer is from argv, which always contains valid pointers to cstrs
        Some(unsafe { CStr::from_ptr(self.end.read()) })
    }
}

impl ExactSizeIterator for Args {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn len(&self) -> usize {
        unsafe { len(self.cur, self.end) }
    }
}
impl FusedIterator for Args {}
