// TODO: optimize this in general
#![allow(clippy::while_let_on_iterator, clippy::copy_iterator)]

use {
    crate::{
        CStr,
        MappedArgs,
        cmdline::helpers::try_to_str,
        iter::helpers::{len, sz_hnt}
    },
    core::iter::FusedIterator
};

// not Copy because that nets a 2-5% performance improvement for some reason
/// An iterator over program arguments as `&'static CStr`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Args {
    pub(crate) cur: *const *const u8,
    pub(crate) end: *const *const u8
}

impl Args {
    /// Map this iterator to a different type. Like [`crate::map_args`], but operates on an existing
    /// iterator.
    #[must_use]
    pub fn map_ty<Ret, F: Fn(*const u8) -> Option<Ret> + Copy + 'static>(
        &self,
        map: F
    ) -> MappedArgs<Ret, F> {
        MappedArgs { cur: self.cur, end: self.end, map }
    }

    /// Map this iterator to `&'static str`. Like [`crate::str_args`], but operates on an existing
    /// iterator. Non-UTF-8 arguments are skipped.
    #[must_use]
    pub fn map_str(&self) -> MappedArgs<&'static str, fn(*const u8) -> Option<&'static str>> {
        MappedArgs { cur: self.cur, end: self.end, map: try_to_str }
    }
}

// most of these are copied or slightly adapted from slice::Iter
impl Iterator for Args {
    type Item = &'static CStr;

    // inline(always) nets a 5% performance loss. no inlining nets a 70% loss. normal inlining is
    //  good.
    #[inline]
    fn next(&mut self) -> Option<&'static CStr> {
        if self.cur == self.end {
            return None;
        }
        assume!(self.cur < self.end);

        let p = self.cur;
        self.cur = unsafe { self.cur.add(1) };
        Some(unsafe { CStr::from_ptr(p.read()) })
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        sz_hnt(self.cur, self.end)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<&'static CStr> {
        if n >= self.len() {
            self.cur = self.end;
            return None;
        }

        let p = unsafe { self.cur.add(n) };
        self.cur = unsafe { p.add(1) };

        assume!(!p.is_null());

        Some(unsafe { CStr::from_ptr(p.read().cast()) })
    }
}

impl DoubleEndedIterator for Args {
    #[inline]
    fn next_back(&mut self) -> Option<&'static CStr> {
        if self.cur == self.end {
            return None;
        }

        self.end = unsafe { self.end.sub(1) };
        Some(unsafe { CStr::from_ptr(self.end.read()) })
    }
}

impl ExactSizeIterator for Args {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn len(&self) -> usize {
        len(self.cur, self.end)
    }
}
impl FusedIterator for Args {}
