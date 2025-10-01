// TODO: optimize this. i *might* be able to do better than copying from slice::Iter can since this
//  type is slightly different. also might do better by being more similar (macros instead of
//  .len() call)
#![allow(clippy::while_let_on_iterator, clippy::copy_iterator)]

use {
    super::helpers::{cstr, cstr_nth, dec_get, len, sz_hnt, unchecked_add},
    crate::MappedArgs,
    core::{ffi::CStr, iter::FusedIterator, ops::Index}
};

// not Copy because that nets a 2-5% performance improvement for some reason
/// An iterator over the program's arguments as <code>&'static [CStr]</code>s.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Args {
    pub(crate) cur: *const *const u8,
    pub(crate) end: *const *const u8
}

impl Args {
    // TODO: dedup with cmdline::map_args, etc.
    /// Maps this iterator to a different type. Similar to [`map_args`](crate::map_args), but
    /// operating on an existing iterator.
    #[must_use]
    pub const fn map_ty<Ret, F: Fn(&'static CStr) -> Option<Ret>>(
        &self,
        map: F
    ) -> MappedArgs<Ret, F> {
        MappedArgs { cur: self.cur, end: self.end, map }
    }

    /// Maps this iterator to `&'static str`. Similar to [`str_args`](crate::str_args), but
    /// operating on an existing iterator.
    #[must_use]
    pub const fn map_str(
        &self
    ) -> MappedArgs<&'static str, fn(&'static CStr) -> Option<&'static str>> {
        MappedArgs { cur: self.cur, end: self.end, map: crate::try_to_str }
    }
}

impl Index<usize> for Args {
    type Output = &'static CStr;

    fn index(&self, index: usize) -> &'static &'static CStr {
        let idx = self.cur.wrapping_add(index);

        assume!(
            idx < self.end && idx >= self.cur,
            "index out of bounds: the len is {} but the index is {}",
            self.len(),
            index
        );

        unsafe { &*self.cur.cast::<&'static CStr>() }
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

        // TODO: make this less weird, and more consistent between this, next_back, nth, and the
        //  same in StrArgsIter
        let p = self.cur;
        self.cur = unsafe { self.cur.add(1) };
        Some(cstr(p))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<&'static CStr> {
        let len = self.len();
        if n >= len {
            self.cur = self.end;
            return None;
        }

        let p = unsafe { self.cur.add(n) };
        self.cur = unsafe { p.add(1) };
        Some(cstr_nth(p))
    }

    #[inline]
    fn fold<B, F: FnMut(B, &'static CStr) -> B>(self, init: B, mut f: F) -> B {
        let len = self.len();
        if len == 0 {
            return init;
        }

        let mut acc = init;
        let mut i = 0;
        loop {
            acc = f(acc, unsafe { cstr(self.cur.add(i)) });
            unsafe {
                unchecked_add(&mut i, 1);
            }
            if i == len {
                break;
            }
        }
        acc
    }

    common_iter_methods!(&'static CStr);
}

impl DoubleEndedIterator for Args {
    #[inline]
    fn next_back(&mut self) -> Option<&'static CStr> {
        if self.cur == self.end {
            return None;
        }

        Some(dec_get(&mut self.end))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<&'static CStr> {
        let len = self.len();
        if n >= len {
            self.cur = self.end;
            return None;
        }

        let p = unsafe { self.end.sub(n + 1) };
        self.end = p;
        Some(cstr_nth(p))
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        len(self.cur, self.end)
    }
}
impl FusedIterator for Args {}
