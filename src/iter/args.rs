// TODO: optimize this in general
#![allow(clippy::while_let_on_iterator)]

import! {
    {
        default::Default,
        iter::{ExactSizeIterator, FusedIterator, Iterator},
        ops::{Fn, FnMut},
        option::Option::{self, None, Some},
    }
}

#[cfg(feature = "rev_iter")]
import! {
    iter::DoubleEndedIterator
}

use {
    crate::{CStr, MappedArgs, cmdline::helpers::try_to_str, iter::helpers::len},
    cmdline::helpers,
    direct
};

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
    #[must_use]
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

    /// Map this iterator to a different type. Like [`MappedArgs::new`], but operates on an existing
    /// iterator.
    #[must_use]
    #[cfg_attr(not(feature = "no_cold"), cold)]
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
    /// Map this iterator to a different type. Like [`MappedArgs::new_infallible`], but operates on
    /// an existing iterator.
    #[must_use]
    #[cfg_attr(not(feature = "no_cold"), cold)]
    pub fn map_ty_infallible<Ret, F: Fn(*const u8) -> Option<Ret>>(
        &self,
        map: F
    ) -> MappedArgs<Ret, F> {
        MappedArgs { cur: self.cur, end: self.end, map, fallible: false }
    }

    /// Map this iterator to `&'static str`. Like [`MappedArgs::utf8`], but operates on an existing
    /// iterator. Non-UTF-8 arguments are skipped.
    #[must_use]
    #[cfg_attr(not(feature = "no_cold"), cold)]
    pub fn map_utf8(&self) -> MappedArgs<&'static str, fn(*const u8) -> Option<&'static str>> {
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
    /// Map this iterator to `&'static OsStr`. Like [`MappedArgs::osstr`], but operates on an
    /// existing iterator.
    #[must_use]
    #[allow(unused_qualifications)]
    #[cfg_attr(not(feature = "no_cold"), cold)]
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

    /// Gets the element at index `i`, or `None` if the index is out-of-bounds. This does
    /// _not_ consume elements like `nth`.
    #[must_use]
    pub fn get(&self, i: usize) -> Option<CStr<'static>> {
        if self.len() > i { Some(unsafe { self.get_unchecked(i) }) } else { None }
    }

    /// Gets the element at index `i`. This does _not_ consume elements.
    ///
    /// # Safety
    ///
    /// The caller must ensure the element at index `i` exists and is in bounds.
    #[must_use]
    pub unsafe fn get_unchecked(&self, i: usize) -> CStr<'static> {
        #[allow(clippy::cast_ptr_alignment)]
        self.cur.add(i).cast::<CStr<'static>>().read()
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    unsafe fn next_back_unchecked(&mut self) -> CStr<'static> {
        // SAFETY: we just checked that `self.end - n` is in bounds
        self.end = self.end.sub(1);
        assume!(!self.end.is_null() && self.end > self.cur);

        // SAFETY: the pointer is from argv, which always contains valid pointers to cstrs
        CStr::from_ptr(self.end.read())
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    unsafe fn next_unchecked(&mut self) -> CStr<'static> {
        let p = self.cur;
        self.cur = self.cur.add(1);
        assume!(!p.is_null() && p < self.end);

        CStr::from_ptr(p.read())
    }
}

// most of these are copied or slightly adapted from slice::Iter
impl Iterator for Args {
    type Item = CStr<'static>;

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self) -> Option<CStr<'static>> {
        if self.cur == self.end {
            return None;
        }
        assume!(self.cur < self.end);

        // SAFETY: we just checked that `self.cur < self.end`, the pointer is from argv, which
        // always contains valid pointers to cstrs
        Some(unsafe { self.next_unchecked() })
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = unsafe { len(self.cur, self.end) };
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn last(mut self) -> Option<CStr<'static>> {
        #[cfg(feature = "rev_iter")]
        {
            self.next_back()
        }
        #[cfg(not(feature = "rev_iter"))]
        {
            if self.cur == self.end {
                return None;
            }

            // SAFETY: we just checked that `self.cur < self.end`, the pointer is from argv, which
            //  always contains valid pointers to cstrs
            Some(unsafe { self.next_back_unchecked() })
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<CStr<'static>> {
        if n >= self.len() {
            self.cur = self.end;
            return None;
        }

        // SAFETY: we just checked that `self.cur + n` is in bounds
        self.cur = unsafe { self.cur.add(n) };
        assume!(self.cur < self.end);

        // SAFETY: we just checked that `self.cur < self.end`, the pointer is from argv, which
        // always contains valid pointers to cstrs
        Some(unsafe { self.next_unchecked() })
    }

    #[inline]
    fn fold<B, F: FnMut(B, CStr<'static>) -> B>(mut self, mut acc: B, mut f: F) -> B {
        if self.cur == self.end {
            return acc;
        }

        loop {
            assume!(!self.cur.is_null() && self.cur < self.end);
            acc = f(acc, unsafe { CStr::from_ptr(self.cur.read()) });

            self.cur = unsafe { self.cur.add(1) };
            if self.cur == self.end {
                break;
            }
        }

        acc
    }
}

#[cfg(feature = "rev_iter")]
impl DoubleEndedIterator for Args {
    #[inline]
    fn next_back(&mut self) -> Option<CStr<'static>> {
        if self.cur == self.end {
            return None;
        }

        // SAFETY: we just checked that `self.cur < self.end`, the pointer is from argv, which
        //  always contains valid pointers to cstrs
        Some(unsafe { self.next_back_unchecked() })
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<CStr<'static>> {
        if n >= self.len() {
            self.end = self.cur;
            return None;
        }

        self.end = unsafe { self.end.sub(n) };
        assume!(!self.end.is_null() && self.end > self.cur);

        // SAFETY: we just checked that `self.cur < self.end`, the pointer is from argv, which
        //  always contains valid pointers to cstrs
        Some(unsafe { self.next_back_unchecked() })
    }

    #[inline]
    fn rfold<B, F: FnMut(B, CStr<'static>) -> B>(mut self, mut acc: B, mut f: F) -> B {
        if self.cur == self.end {
            return acc;
        }

        loop {
            // SAFETY: we just checked that `self.end > self.cur` in the last loop
            self.end = unsafe { self.end.sub(1) };
            assume!(!self.end.is_null() && self.end > self.cur);

            // SAFETY: the pointer is from argv, which always contains valid pointers to cstrs
            acc = f(acc, unsafe { CStr::from_ptr(self.end.read()) });

            // SAFETY: next deref is guarded by the if and break below
            if self.cur == self.end {
                break;
            }
        }
        acc
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
