#![allow(non_camel_case_types)]

type size_t = usize;
type c_char = u8;

#[cfg(any(target_arch = "avr", target_arch = "msp430"))]
pub type c_int = i16;
#[cfg(any(target_arch = "avr", target_arch = "msp430"))]
pub type c_uint = u16;

#[cfg(not(any(target_arch = "avr", target_arch = "msp430")))]
pub type c_int = i32;
#[cfg(not(any(target_arch = "avr", target_arch = "msp430")))]
pub type c_uint = u32;

extern "C" {
    /// Gets the length of a C-style string by finding the first nul byte after the given pointer.
    pub fn strlen(s: *const c_char) -> size_t;
}

pub mod minimal_cstr {
    extern crate core;

    use super::{c_char, strlen};
    import! {
        {cmp::PartialEq, marker::PhantomData}
    }

    // TODO: make this a full-fledged CStr implementation so no need for to_stdlib

    /// A minimal CStr implementation for use in place of `core::ffi::CStr` (unstable before 1.64)
    /// and `std::ffi::CStr` (requires `std`).
    ///
    /// To do anything meaningful with this, you must convert it to the standard library's fully
    /// implemented version via [`to_stdlib`](CStr::to_stdlib).
    ///
    /// Do not call `to_stdlib` more than once, as every call runs `strlen` to determine the length
    /// of the `CStr`.
    #[repr(transparent)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CStr<'a> {
        inner: *const c_char,
        _marker: PhantomData<&'a [c_char]>
    }

    impl PartialEq<*const u8> for CStr<'_> {
        #[allow(clippy::inline_always)]
        #[inline(always)]
        fn eq(&self, other: &*const u8) -> bool {
            self.inner == *other
        }
    }
    impl<'a> PartialEq<CStr<'a>> for *const u8 {
        #[allow(clippy::inline_always)]
        #[inline(always)]
        fn eq(&self, other: &CStr<'a>) -> bool {
            *self == other.inner
        }
    }

    #[allow(clippy::inline_always)]
    #[allow(clippy::len_without_is_empty)]
    impl<'a> CStr<'a> {
        /// Gets a pointer to the start of this `CStr`.
        ///
        /// # Examples
        ///
        /// ```
        /// // this is necessary for the test to not cause UB with miri; this shouldn't be necessary
        /// //  in normal code.
        /// const ARGS: [*const u8; 1] = ["argument\0".as_ptr()];
        /// unsafe { snailx::direct::set_argc_argv(ARGS.len() as u32, ARGS.as_ptr()) };
        ///
        /// let first = snailx::Args::new().as_slice().get(0).copied();
        /// if let Some(c) = first {
        ///     let p = c.as_ptr();
        ///     assert!(!p.is_null());
        /// }
        /// ```
        #[must_use]
        #[inline(always)]
        pub const fn as_ptr(&self) -> *const u8 {
            self.inner.cast()
        }

        /// Gets the length of this `CStr`.
        ///
        /// Avoid calling this function more than once.
        ///
        /// # Examples
        ///
        /// ```
        /// // this is necessary for the test to not cause UB with miri; this shouldn't be necessary
        /// //  in normal code.
        /// const ARGS: [*const u8; 1] = ["argument\0".as_ptr()];
        /// unsafe { snailx::direct::set_argc_argv(ARGS.len() as u32, ARGS.as_ptr()) };
        ///
        /// let first = snailx::Args::new().as_slice().get(0).copied();
        /// if let Some(c) = first {
        ///     let _l = c.len();
        /// }
        /// ```
        #[must_use]
        #[inline(always)]
        pub fn len(&self) -> usize {
            unsafe { strlen(self.inner) }
        }

        #[cfg(all(feature = "std", not(feature = "to_core_cstr")))]
        /// Converts this value into the `std` equivalent.
        ///
        /// # Examples
        ///
        /// ```
        /// // this is necessary for the test to not cause UB with miri; this shouldn't be necessary
        /// //  in normal code.
        /// const ARGS: [*const u8; 1] = ["argument\0".as_ptr()];
        /// unsafe { snailx::direct::set_argc_argv(ARGS.len() as u32, ARGS.as_ptr()) };
        ///
        /// let first = snailx::Args::new().as_slice().get(0).copied();
        /// if let Some(c) = first {
        ///     let s = c.to_stdlib().to_string_lossy();
        ///     let _ = s.len();
        /// }
        /// ```
        #[must_use]
        #[inline(always)]
        pub fn to_stdlib(&self) -> &'a ::std::ffi::CStr {
            // SAFETY: from_ptr requires that the pointer is a valid CStr
            unsafe {
                assume!(!self.inner.is_null());
                let bytes =
                    switch!(core::slice::from_raw_parts(self.inner, strlen(self.inner.cast()) + 1));
                assume!(
                    !bytes.is_empty() && bytes[bytes.len() - 1] == 0,
                    "CStr does not end with null byte"
                );

                &*(bytes as *const [u8] as *const ::std::ffi::CStr)
            }
        }

        #[cfg(feature = "to_core_cstr")]
        /// Converts this value into the `core` equivalent.
        ///
        /// # Examples
        ///
        /// ```
        /// // this is necessary for the test to not cause UB with miri; this shouldn't be necessary
        /// //  in normal code.
        /// const ARGS: [*const u8; 1] = ["argument\0".as_ptr()];
        /// unsafe { snailx::direct::set_argc_argv(ARGS.len() as u32, ARGS.as_ptr()) };
        ///
        /// let first = snailx::Args::new().as_slice().get(0).copied();
        /// if let Some(c) = first {
        ///     let core_cstr = c.to_stdlib();
        ///     // Access the raw bytes (without the terminating nul):
        ///     let _ = core_cstr.to_bytes().len();
        /// }
        /// ```
        #[must_use]
        #[inline(always)]
        pub fn to_stdlib(&self) -> &'a core::ffi::CStr {
            // SAFETY: from_ptr requires that the pointer is a valid CStr
            unsafe {
                assume!(!self.inner.is_null());
                let bytes =
                    switch!(core::slice::from_raw_parts(self.inner, strlen(self.inner.cast()) + 1));
                assume!(
                    !bytes.is_empty() && bytes[bytes.len() - 1] == 0,
                    "CStr does not end with null byte"
                );

                &*(bytes as *const [u8] as *const core::ffi::CStr)
            }
        }

        /// Creates a `CStr` from a pointer to its first byte.
        ///
        /// # Examples
        ///
        /// ```
        /// // this is necessary for the test to not cause UB with miri; this shouldn't be necessary
        /// //  in normal code.
        /// const ARGS: [*const u8; 1] = ["argument\0".as_ptr()];
        /// unsafe { snailx::direct::set_argc_argv(ARGS.len() as u32, ARGS.as_ptr()) };
        ///
        /// let first = snailx::Args::new().as_slice().get(0).copied();
        /// if let Some(c) = first {
        ///     let p = c.as_ptr();
        ///     // SAFETY: Using a pointer provided by the OS for argv is valid here.
        ///     // Note that this operation is redundant and only an example; `first` was already a
        ///     //  CStr.
        ///     let rebuilt = unsafe { snailx::CStr::from_ptr(p) };
        ///     assert_eq!(rebuilt.as_ptr(), p);
        /// }
        /// ```
        ///
        /// # Safety
        ///
        /// - The memory pointed to by `ptr` must contain a valid nul terminator at the end of the
        ///   string.
        /// - `ptr` must be valid for reads of bytes up to and including the nul terminator. This
        ///   means in particular:
        ///     - The entire memory range of this `CStr` must be contained within a single
        ///       allocation!
        ///     - `ptr` must be non-null even for a zero-length cstr.
        /// - The memory referenced by the returned `CStr` must not be mutated for the duration of
        ///   lifetime `'a`.
        /// - The nul terminator must be within `isize::MAX` bytes from `ptr`
        #[must_use]
        #[inline(always)]
        pub unsafe fn from_ptr(p: *const u8) -> CStr<'a> {
            assume!(
                dbg,
                {
                    let len = strlen(p.cast());
                    let bytes = switch!(core::slice::from_raw_parts(p, len + 1));
                    !bytes.is_empty() && bytes[len] == 0
                },
                "CStr does not end with null byte"
            );

            CStr { inner: p, _marker: PhantomData }
        }
    }
}
