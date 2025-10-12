#![allow(clippy::while_let_on_iterator, clippy::copy_iterator)]

use {
    super::helpers::{len, sz_hnt},
    core::iter::FusedIterator
};

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
pub struct MappedArgs<
    Ret,
    F: Fn(*const u8) -> Option<Ret> + Copy + 'static = fn(*const u8) -> Option<Ret>
> {
    pub(crate) cur: *const *const u8,
    pub(crate) end: *const *const u8,
    pub(crate) map: F 
    // MAYBEDO: below bc sometimes nth has a faster method
    // pub(crate) nth_map: F
}

impl<Ret, F: Fn(*const u8) -> Option<Ret> + Copy + 'static> Iterator for MappedArgs<Ret, F> {
    type Item = Ret;

    // TODO: try rewriting these to be faster

    #[inline]
    fn next(&mut self) -> Option<Ret> {
        let mut ret = None;

        while self.cur != self.end {
            let p = self.cur;
            // SAFETY: we just checked that `p < self.end`
            self.cur = unsafe { self.cur.add(1) };

            // SAFETY: the pointer is from argv, which always contains valid pointers to cstrs
            if let Some(v) = (self.map)(unsafe { p.read() }) {
                ret = Some(v);
                break;
            }
        }

        ret
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        sz_hnt(self.cur, self.end)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Ret> {
        if n >= self.len() {
            self.cur = self.end;
            return None;
        }

        let mut ret = None;

        while self.cur != self.end {
            // SAFETY: we just checked that `self.cur + n` is in bounds
            let p = unsafe { self.cur.add(n) };
            self.cur = unsafe { p.add(1) };

            // SAFETY: the pointer is from argv, which always contains valid pointers to cstrs
            if let Some(v) = (self.map)(unsafe { p.read() }) {
                ret = Some(v);
                break;
            }
        }

        ret
    }

    #[inline]
    fn fold<B, G: FnMut(B, Ret) -> B>(self, init: B, mut f: G) -> B {
        let len = self.len();
        if len == 0 {
            return init;
        }

        let mut acc = init;

        let mut i = 0;
        loop {
            // SAFETY: we just checked that `self.cur + i` is in bounds, pointer is from argv which
            // always contains valid pointers to cstrs
            if let Some(v) = (self.map)(unsafe { self.cur.add(i).read() }) {
                acc = f(acc, v);
            }

            assume!(i.checked_add(1).is_some(), "integer overflow");
            i += 1;

            if i == len {
                break;
            }
        }
        acc
    }
}

impl<Ret, F: Fn(*const u8) -> Option<Ret> + Copy + 'static> DoubleEndedIterator
    for MappedArgs<Ret, F>
{
    // TODO: skip like next and nth do
    #[inline]
    fn next_back(&mut self) -> Option<Ret> {
        if self.cur == self.end {
            return None;
        }

        // SAFETY: we just checked that `self.cur < self.end`
        unsafe {
            self.end = self.end.sub(1);
            (self.map)(self.end.read())
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Ret> {
        let len = self.len();
        if n >= len {
            self.cur = self.end;
            return None;
        }

        // Move end backward by n+1 (exclusive-end semantics) and read the element
        unsafe {
            self.end = self.end.sub(n + 1);
            (self.map)(self.end.read())
        }
    }
}

impl<Ret, F: Fn(*const u8) -> Option<Ret> + Copy + 'static> ExactSizeIterator
    for MappedArgs<Ret, F>
{
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn len(&self) -> usize {
        len(self.cur, self.end)
    }
}
impl<Ret, F: Fn(*const u8) -> Option<Ret> + Copy + 'static> FusedIterator for MappedArgs<Ret, F> {}
