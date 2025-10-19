#![allow(clippy::while_let_on_iterator, unused_qualifications)]

import! {
    {
        iter::{DoubleEndedIterator, Iterator},
        ops::{Fn, FnMut},
        option::Option::{self, None, Some}
    }
}
use {super::helpers::len, CStr, cmdline::helpers, direct};

// TODO: count() impl

// /// This enum is used to determine what to do when an argument fails to parse.
// #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
// #[repr(u8)]
// pub enum Recovery {
//     /// Return the contained string.
//     Yield(&'static str),
//     /// Skip the argument, returning the next valid argument or `None` if there are no more.
//     Skip,
//     /// Just return `None`.
//     YieldNone
// }

// not Copy for consistency with Args
/// An iterator that maps each argument using a user-provided function. If the mapping returns
/// `None`, that argument is skipped.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MappedArgs<Ret, F: Fn(*const u8) -> Option<Ret> = fn(*const u8) -> Option<Ret>> {
    pub(crate) cur: *const *const u8,
    pub(crate) end: *const *const u8,
    pub(crate) map: F,
    #[cfg(feature = "infallible_map")]
    pub(crate) fallible: bool
}

impl MappedArgs<&'static str, fn(*const u8) -> Option<&'static str>> {
    /// Returns an iterator over the program's arguments as `&'static str`. Non-UTF-8 arguments are
    /// skipped.
    #[must_use]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    #[cfg_attr(not(feature = "no_cold"), cold)]
    pub fn utf8() -> MappedArgs<&'static str, fn(*const u8) -> Option<&'static str>> {
        MappedArgs::new(helpers::try_to_str)
    }
}

#[cfg(feature = "std")]
impl MappedArgs<&'static ::std::ffi::OsStr, fn(*const u8) -> Option<&'static ::std::ffi::OsStr>> {
    /// Returns an iterator over the program's arguments as `&'static std::ffi::OsStr`. Requires the
    /// `std` feature.
    #[must_use]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    #[cfg_attr(not(feature = "no_cold"), cold)]
    pub fn osstr()
    -> MappedArgs<&'static ::std::ffi::OsStr, fn(*const u8) -> Option<&'static ::std::ffi::OsStr>>
    {
        #[cfg(not(feature = "infallible_map"))]
        {
            MappedArgs::new(helpers::to_osstr)
        }
        #[cfg(feature = "infallible_map")]
        {
            MappedArgs::new_infallible(helpers::to_osstr)
        }
    }
}

#[allow(clippy::len_without_is_empty)]
impl<Ret, F: Fn(*const u8) -> Option<Ret>> MappedArgs<Ret, F> {
    /// Returns an iterator that applies `map` to each argument (`*const u8`). If `map` returns
    /// `None`, that argument is skipped.
    ///
    /// The mapping function is assumed to be fallible, so `size_hint()` will return
    /// `(0, Some(len))`.
    #[must_use]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    #[cfg_attr(not(feature = "no_cold"), cold)]
    pub fn new(map: F) -> MappedArgs<Ret, F> {
        let (argc, argv) = direct::argc_argv();
        MappedArgs {
            cur: argv,
            end: helpers::back(argv, argc),
            map,
            #[cfg(feature = "infallible_map")]
            fallible: true
        }
    }

    #[cfg(feature = "infallible_map")]
    /// Returns an iterator that applies `map` to each argument (`*const u8`).
    ///
    /// The mapping function is assumed to be infallible, so `size_hint()` will return
    /// `(len, Some(len))`.
    ///
    /// `map` should never return `None`, but in the case that it does, it will be skipped.
    #[must_use]
    #[allow(clippy::inline_always)]
    #[inline(always)]
    #[cfg_attr(not(feature = "no_cold"), cold)]
    pub fn new_infallible(map: F) -> MappedArgs<Ret, F> {
        let (argc, argv) = direct::argc_argv();
        MappedArgs { cur: argv, end: helpers::back(argv, argc), map, fallible: false }
    }

    /// Gets the remaining arguments in this iterator as a slice.
    #[must_use]
    #[inline]
    pub fn as_slice(&self) -> &'static [CStr<'static>] {
        unsafe {
            switch!(core::slice::from_raw_parts(
                self.cur.cast::<CStr<'static>>(),
                len(self.cur, self.end)
            ))
        }
    }

    /// Gets the remaining length of items in this iterator.
    ///
    /// Returns `None` if `infallible_map` is disabled or this iterator's mapping function is marked
    /// as fallible. If `infallible_map` is enabled and this iterator is marked as infallible,
    /// returns `Some(len)`.
    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub fn len(&self) -> Option<usize> {
        #[cfg(not(feature = "infallible_map"))]
        {
            None
        }
        #[cfg(feature = "infallible_map")]
        {
            if self.fallible { None } else { Some(unsafe { len(self.cur, self.end) }) }
        }
    }
}

impl<Ret, F: Fn(*const u8) -> Option<Ret>> Iterator for MappedArgs<Ret, F> {
    type Item = Ret;

    // TODO: try rewriting these to be faster

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn next(&mut self) -> Option<Ret> {
        let mut ret = None;

        while self.cur != self.end {
            // SAFETY: we just checked that `self.cur + n` is in bounds
            let p = self.cur;
            self.cur = unsafe { self.cur.add(1) };
            assume!(!p.is_null() && p < self.end);

            // SAFETY: the pointer is from argv, which always contains valid pointers to cstrs
            if let Some(v) = (self.map)(unsafe { p.read() }) {
                ret = Some(v);
                break;
            }
        }

        ret
    }

    /// Returns the bounds on the remaining length of the iterator.
    ///
    /// Specifically, `size_hint()` returns a tuple where the first element
    /// is the lower bound, and the second element is the upper bound.
    ///
    /// The upper bound will always be `Some(len)`, where `len` is the number of elements remaining
    /// in the iterator if the mapping function returns `Some` for every element.
    ///
    /// If `infallible_map` is disabled or this iterator's mapping function has been marked as
    /// fallible, the lower bound will be 0. If `infallible_map` is enabled and this iterator is
    /// marked as infallible, the lower bound will also be `len`.
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        #[cfg(not(feature = "infallible_map"))]
        // 0 lower bound because all args may be skipped, len upper bound because all may be fine
        {
            (0, Some(unsafe { len(self.cur, self.end) }))
        }
        #[cfg(feature = "infallible_map")]
        {
            let len = unsafe { len(self.cur, self.end) };
            if self.fallible { (0, Some(len)) } else { (len, Some(len)) }
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Ret> {
        if n >= unsafe { len(self.cur, self.end) } {
            self.cur = self.end;
            return None;
        }

        self.cur = unsafe { self.cur.add(n) };
        assume!(!self.cur.is_null() && self.cur < self.end);

        self.next()
    }

    #[inline]
    fn fold<B, G: FnMut(B, Ret) -> B>(self, init: B, f: G) -> B {
        map_helpers::fold(self, init, f, <*const *const u8>::add, usize::checked_add)
    }
}

impl<Ret, F: Fn(*const u8) -> Option<Ret>> DoubleEndedIterator for MappedArgs<Ret, F> {
    #[inline]
    fn next_back(&mut self) -> Option<Ret> {
        let mut ret = None;

        while self.cur != self.end {
            // SAFETY: we just checked that `self.cur < self.end`
            self.end = unsafe { self.end.sub(1) };

            assume!(!self.end.is_null() && self.end > self.cur);

            if let Some(v) = (self.map)(unsafe { self.end.read() }) {
                ret = Some(v);
                break;
            }
        }

        ret
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Ret> {
        if n >= unsafe { len(self.cur, self.end) } {
            self.cur = self.end;
            return None;
        }

        self.end = unsafe { self.end.sub(n) };
        assume!(!self.end.is_null() && self.end > self.cur);

        self.next_back()
    }

    #[inline]
    fn rfold<B, G: FnMut(B, Ret) -> B>(self, init: B, f: G) -> B {
        map_helpers::fold(self, init, f, <*const *const u8>::sub, usize::checked_sub)
    }
}

mod map_helpers {
    import! {
        {
            ops::{Fn, FnMut},
            option::Option::{self, Some}
        }
    }
    use crate::{MappedArgs, iter::helpers::len};

    #[allow(clippy::inline_always)]
    #[inline(always)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn fold<Ret, F: Fn(*const u8) -> Option<Ret>, B, G: FnMut(B, Ret) -> B>(
        slf: MappedArgs<Ret, F>,
        mut acc: B,
        mut f: G,
        modfun: unsafe fn(*const *const u8, usize) -> *const *const u8,
        check: fn(usize, usize) -> Option<usize>
    ) -> B {
        let len = unsafe { len(slf.cur, slf.end) };
        if len == 0 {
            return acc;
        }

        let mut i = 0;

        loop {
            // SAFETY: we just checked that `self.cur + i` is in bounds, pointer is from argv which
            // always contains valid pointers to cstrs
            if let Some(v) = (slf.map)(unsafe { modfun(slf.cur, i).read() }) {
                acc = f(acc, v);
            }

            assume!(check(i, 1).is_some(), "integer overflow");
            i += 1;

            if i == len {
                break;
            }
        }
        acc
    }
}

// removed as i realized neither of these fit the functionality of MappedArgs
//
// impl<Ret, F: Fn(*const u8) -> Option<Ret>> ExactSizeIterator
//     for MappedArgs<Ret, F>
// {
//     #[allow(clippy::inline_always)]
//     #[inline(always)]
//     fn len(&self) -> usize {
//         len(self.cur, self.end)
//     }
// }
// impl<Ret, F: Fn(*const u8) -> Option<Ret>> FusedIterator for MappedArgs<Ret, F>
// {}
