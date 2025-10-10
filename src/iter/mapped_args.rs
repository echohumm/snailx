#![allow(clippy::while_let_on_iterator, clippy::copy_iterator)]

use {
    super::helpers::{len, sz_hnt, unchecked_add},
    core::{iter::FusedIterator, ops::Index},
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
    F: Fn(*const u8) -> Option<Ret> + Copy + 'static = fn(*const u8) -> Option<Ret>,
> {
    pub(crate) cur: *const *const u8,
    pub(crate) end: *const *const u8,
    pub(crate) map: F,
    // MAYBEDO: below bc sometimes nth has a faster method
    // pub(crate) nth_map: F
}

impl<F: Fn(*const u8) -> Option<&'static str> + Copy + 'static> Index<usize>
    for MappedArgs<&'static str, F>
{
    type Output = str;

    fn index(&self, index: usize) -> &'static str {
        // TODO: make this a helper
        let idx = self.cur.wrapping_add(index);

        assume!(
            idx < self.end && idx >= self.cur,
            "index out of bounds: the len is {} but the index is {}",
            self.len(),
            index
        );

        (self.map)(unsafe { idx.read() }).expect("cstr contains invalid UTF-8")
    }
}

#[cfg(feature = "std")]
impl<F: Fn(*const u8) -> Option<&'static std::ffi::OsStr> + Copy + 'static> Index<usize>
    for MappedArgs<&'static std::ffi::OsStr, F>
{
    type Output = std::ffi::OsStr;

    fn index(&self, index: usize) -> &'static std::ffi::OsStr {
        let idx = self.cur.wrapping_add(index);

        assume!(
            idx < self.end && idx >= self.cur,
            "index out of bounds: the len is {} but the index is {}",
            self.len(),
            index
        );

        unsafe { (self.map)(idx.read()).unwrap_unchecked() }
    }
}

// TODO: dedup with Args

impl<Ret, F: Fn(*const u8) -> Option<Ret> + Copy + 'static> Iterator for MappedArgs<Ret, F> {
    type Item = Ret;

    #[inline]
    fn next(&mut self) -> Option<Ret> {
        let mut ret = None;

        while self.cur != self.end {
            let p = self.cur;
            self.cur = unsafe { self.cur.add(1) };

            if let Some(v) = (self.map)(unsafe { p.read() }) {
                ret = Some(v);
                break;
            }
        }

        ret
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Ret> {
        if n >= self.len() {
            self.cur = self.end;
            return None;
        }

        let mut ret = None;

        // TODO: try rewriting this to be faster
        while self.cur != self.end {
            let p = unsafe { self.cur.add(n) };
            self.cur = unsafe { p.add(1) };

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
            if let Some(v) = (self.map)(unsafe { self.cur.add(i).read() }) {
                acc = f(acc, v);
            }

            unsafe {
                unchecked_add(&mut i, 1);
            }
            if i == len {
                break;
            }
        }
        acc
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        sz_hnt(self.cur, self.end)
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
    #[inline(always)]
    fn len(&self) -> usize {
        len(self.cur, self.end)
    }
}
impl<Ret, F: Fn(*const u8) -> Option<Ret> + Copy + 'static> FusedIterator for MappedArgs<Ret, F> {}
