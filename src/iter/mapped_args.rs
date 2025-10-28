#![allow(clippy::while_let_on_iterator, unused_qualifications)]

import! {
    {
        iter::{Iterator, FusedIterator},
        ops::{Fn, FnMut},
        option::Option::{self, None, Some}
    }
}

#[cfg(feature = "rev_iter")]
import! {
    iter::DoubleEndedIterator
}

use {
    super::{args::Args, helpers::len},
    cmdline::helpers,
    direct
};
// TODO: may be better to not implement certain things manually and just delegate to fold

macro_rules! fallible_q {
    ($self:ident, $f:expr, $i:expr) => {
        #[cfg(not(feature = "infallible_map"))]
        $f
        #[cfg(feature = "infallible_map")]
        if $self.fallible {
            $f
        } else {
            $i
        }
    };
}

macro_rules! next_back {
    ($self:ident) => {{
        while $self.cur != $self.end {
            // SAFETY: we just checked that `$self.cur < $self.end`
            $self.end = unsafe { $self.end.sub(1) };

            assume!(!$self.end.is_null() && $self.end > $self.cur);

            if let Some(v) = ($self.map)(unsafe { $self.end.read() }) {
                return Some(v);
            }
        }

        None
    }};
}

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
            // SAFETY: to_osstr only returns Some
            unsafe { MappedArgs::new_infallible(helpers::to_osstr) }
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
    /// # Safety
    ///
    /// `map` must never return `None`.
    #[must_use]
    #[cfg_attr(not(feature = "no_cold"), cold)]
    pub unsafe fn new_infallible(map: F) -> MappedArgs<Ret, F> {
        let (argc, argv) = direct::argc_argv();
        MappedArgs { cur: argv, end: helpers::back(argv, argc), map, fallible: false }
    }

    /// Converts this mapped iterator to an [`Args`] instance. Like [`Args::new`], but operates on
    /// an existing mapped iterator.
    #[must_use]
    #[cfg_attr(not(feature = "no_cold"), cold)]
    pub fn unmap(self) -> Args {
        Args { cur: self.cur, end: self.end }
    }

    // as_slice removed as it was pretty useless

    /// Gets the remaining length of items in this iterator.
    ///
    /// Returns `None` if `infallible_map` is disabled or this iterator's mapping function is marked
    /// as fallible. If `infallible_map` is enabled and this iterator is marked as infallible,
    /// returns `Some(len)`.
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
        while self.cur != self.end {
            // SAFETY: we just checked that `self.cur + n` is in bounds
            let p = self.cur;
            self.cur = unsafe { self.cur.add(1) };
            assume!(!p.is_null() && p < self.end);

            // SAFETY: the pointer is from argv, which always contains valid pointers to cstrs
            if let Some(v) = (self.map)(unsafe { p.read() }) {
                return Some(v);
            }
        }

        None
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

    #[cfg(feature = "infallible_map")]
    #[inline]
    fn count(self) -> usize {
        if self.fallible {
            self.fold(0, |count, _| count + 1)
        } else {
            // SAFETY: the pointers are guaranteed to be valid for len() as they are from argv
            unsafe { len(self.cur, self.end) }
        }
    }

    #[inline]
    fn last(mut self) -> Option<Ret> {
        #[cfg(feature = "rev_iter")]
        {
            self.next_back()
        }
        #[cfg(not(feature = "rev_iter"))]
        {
            next_back!(self)
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Ret> {
        // SAFETY: the pointers are guaranteed to be valid for len() as they are from argv
        if n >= unsafe { len(self.cur, self.end) } {
            self.cur = self.end;
            return None;
        }

        fallible_q!(
            self,
            {
                let mut i = 0;
                while self.cur != self.end {
                    let p = self.cur;
                    // SAFETY: we just checked that `self.cur < self.end`
                    self.cur = unsafe { self.cur.add(1) };
                    assume!(!p.is_null() && p < self.end);

                    // SAFETY: the pointer is from argv, which always contains valid pointers to
                    // cstrs
                    if let Some(v) = (self.map)(unsafe { p.read() }) {
                        if i == n {
                            return Some(v);
                        }
                        i += 1;
                    }
                }
            },
            {
                // SAFETY: we just checked that `self.cur + n` is in bounds
                self.cur = unsafe { self.cur.add(n) };
                assume!(!self.cur.is_null() && self.cur < self.end);

                return self.next();
            }
        );

        None
    }

    #[inline]
    fn fold<B, G: FnMut(B, Ret) -> B>(mut self, mut acc: B, mut f: G) -> B {
        if self.cur == self.end {
            return acc;
        }

        loop {
            assume!(!self.cur.is_null() && self.cur < self.end);
            fallible_q!(
                self,
                {
                    if let Some(v) = (self.map)(unsafe { self.cur.read() }) {
                        acc = f(acc, v);
                    }
                },
                {
                    // SAFETY: caller guarantees that the map is infallible
                    acc = f(
                        acc,
                        assume!(
                            car,
                            Some,
                            e,
                            unsafe { (self.map)(self.cur.read()) },
                            "map is infallible, but returned None"
                        )
                    );
                }
            );

            // SAFETY: we just checked that `self.cur` is in bounds
            self.cur = unsafe { self.cur.add(1) };
            if self.cur == self.end {
                break;
            }
        }
        acc
    }
}

#[cfg(feature = "rev_iter")]
impl<Ret, F: Fn(*const u8) -> Option<Ret>> DoubleEndedIterator for MappedArgs<Ret, F> {
    #[inline]
    fn next_back(&mut self) -> Option<Ret> {
        next_back!(self)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Ret> {
        if n >= unsafe { len(self.cur, self.end) } {
            self.cur = self.end;
            return None;
        }

        fallible_q!(
            self,
            {
                let mut i = 0;
                while self.cur != self.end {
                    self.end = unsafe { self.end.sub(1) };
                    assume!(!self.end.is_null() && self.end > self.cur);

                    if let Some(v) = (self.map)(unsafe { self.end.read() }) {
                        if i == n {
                            return Some(v);
                        }
                        i += 1;
                    }
                }
            },
            {
                self.end = unsafe { self.end.sub(n) };
                assume!(!self.end.is_null() && self.end > self.cur);

                return self.next_back();
            }
        );

        None
    }

    #[inline]
    fn rfold<B, G: FnMut(B, Ret) -> B>(mut self, mut acc: B, mut f: G) -> B {
        if self.cur == self.end {
            return acc;
        }

        loop {
            // SAFETY: we just checked that `self.cur < self.end` in the last loop
            self.end = unsafe { self.end.sub(1) };
            assume!(!self.end.is_null() && self.end > self.cur);

            #[cfg(not(feature = "infallible_map"))]
            {
                // SAFETY: the pointer is from argv, which always contains valid pointers to cstrs
                if let Some(v) = (self.map)(unsafe { self.end.read() }) {
                    acc = f(acc, v);
                }
            }
            #[cfg(feature = "infallible_map")]
            {
                if self.fallible {
                    // SAFETY: the pointer is from argv, which always contains valid pointers to
                    // cstrs
                    if let Some(v) = (self.map)(unsafe { self.end.read() }) {
                        acc = f(acc, v);
                    }
                } else {
                    // SAFETY: caller guarantees that the map is infallible
                    acc = f(
                        acc,
                        assume!(
                            car,
                            Some,
                            e,
                            unsafe { (self.map)(self.end.read()) },
                            "map is infallible, but returned None"
                        )
                    );
                }
            }

            if self.cur == self.end {
                break;
            }
        }
        acc
    }
}

impl<Ret, F: Fn(*const u8) -> Option<Ret>> FusedIterator for MappedArgs<Ret, F> {}

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
