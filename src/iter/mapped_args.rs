#![allow(clippy::while_let_on_iterator, clippy::copy_iterator)]

use {
    super::helpers::{cstr, cstr_nth, dec_get, len, sz_hnt, unchecked_add},
    core::{ffi::CStr, iter::FusedIterator, ops::Index}
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
pub struct MappedArgs<Ret, F: Fn(&'static CStr) -> Option<Ret> + Copy + 'static = fn(&'static CStr) -> Option<Ret>> {
    pub(crate) cur: *const *const u8,
    pub(crate) end: *const *const u8,
    pub(crate) map: F
}

impl<F: Fn(&'static CStr) -> Option<&'static str> + Copy + 'static> Index<usize> for MappedArgs<&'static str, F> {
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

        (self.map)(cstr(idx)).expect("cstr contains invalid UTF-8")
    }
}

#[cfg(feature = "std")]
impl<F: Fn(&'static CStr) -> Option<&'static std::ffi::OsStr> + Copy + 'static> Index<usize>
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

        unsafe { (self.map)(cstr(idx)).unwrap_unchecked() }
    }
}

// TODO: dedup with Args

impl<Ret, F: Fn(&'static CStr) -> Option<Ret> + Copy + 'static> Iterator for MappedArgs<Ret, F> {
    type Item = Ret;

    #[inline]
    fn next(&mut self) -> Option<Ret> {
        let mut ret = None;

        while self.cur != self.end {
            let s = cstr(self.cur);
            self.cur = unsafe { self.cur.add(1) };

            if let Some(v) = (self.map)(s) {
                ret = Some(v);
                break;
            }
        }

        ret
    }

    // TODO: make these skip as well, like next()

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Ret> {
        let len = self.len();
        if n >= len {
            self.cur = self.end;
            return None;
        }

        let p = unsafe { self.cur.add(n) };
        self.cur = unsafe { p.add(1) };
        let s = cstr_nth(p);
        (self.map)(s)
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
            let s = unsafe { cstr(self.cur.add(i)) };
            if let Some(v) = (self.map)(s) {
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

    common_iter_methods! { Ret }
}

impl<Ret, F: Fn(&'static CStr) -> Option<Ret> + Copy + 'static> DoubleEndedIterator for MappedArgs<Ret, F> {
    #[inline]
    fn next_back(&mut self) -> Option<Ret> {
        if self.cur == self.end {
            return None;
        }

        let s = dec_get(&mut self.end);
        (self.map)(s)
    }

    fn nth_back(&mut self, n: usize) -> Option<Ret> {
        let len = self.len();
        if n >= len {
            self.cur = self.end;
            return None;
        }

        // Move end backward by n+1 (exclusive-end semantics) and read the element
        let p = unsafe { self.end.sub(n + 1) };
        self.end = p;
        let s = cstr_nth(p);
        (self.map)(s)
    }
}

impl<Ret, F: Fn(&'static CStr) -> Option<Ret> + Copy + 'static> ExactSizeIterator for MappedArgs<Ret, F> {
    #[inline(always)]
    fn len(&self) -> usize {
        len(self.cur, self.end)
    }
}
impl<Ret, F: Fn(&'static CStr) -> Option<Ret> + Copy + 'static> FusedIterator for MappedArgs<Ret, F> {}
