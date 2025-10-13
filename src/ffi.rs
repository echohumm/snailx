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
    pub fn strlen(s: *const c_char) -> size_t;
}

pub mod minimal_cstr {
    extern crate core;

    use super::c_char;
    import! {
        use core::marker::PhantomData
    }
    // TODO: no cfgs on imports
    #[cfg(any(feature = "std", feature = "to_core_cstr", debug_assertions))] use super::strlen;
    #[cfg(any(feature = "std", feature = "to_core_cstr", debug_assertions))]
    import! {
        use core::slice
    }

    /// A minimal CStr implementation for use in place of `core::ffi::CStr` (unstable before 1.64)
    /// and `std::ffi::CStr` (requires `std`).
    ///
    /// To do anything meaningful with this, you must convert it to the standard library's fully
    /// implemented version via [`to_stdlib`](CStr::to_stdlib).
    ///
    /// Do not call `to_stdlib` more than once, as every call runs `strlen` to determine the length
    /// of the `CStr`.
    #[repr(transparent)]
    pub struct CStr<'a> {
        _inner: *const c_char,
        _marker: PhantomData<&'a [c_char]>
    }

    #[allow(clippy::inline_always)]
    impl<'a> CStr<'a> {
        #[cfg(all(feature = "std", not(feature = "to_core_cstr")))]
        /// Converts this value into the `std` equivalent.
        #[must_use]
        #[inline(always)]
        pub fn to_stdlib(&self) -> &'a ::std::ffi::CStr {
            // SAFETY: from_ptr requires that the pointer is a valid CStr
            unsafe {
                assume!(!self._inner.is_null());
                let bytes = slice::from_raw_parts(self._inner, strlen(self._inner.cast()) + 1);
                assume!(
                    !bytes.is_empty() && bytes[bytes.len() - 1] == 0,
                    "CStr does not end with null byte"
                );

                &*(bytes as *const [u8] as *const ::std::ffi::CStr)
            }
        }

        #[cfg(feature = "to_core_cstr")]
        /// Converts this value into the `core` equivalent.
        #[must_use]
        #[inline(always)]
        pub fn to_stdlib(&self) -> &'a core::ffi::CStr {
            // SAFETY: from_ptr requires that the pointer is a valid CStr
            unsafe {
                assume!(!self._inner.is_null());
                let bytes = slice::from_raw_parts(self._inner, strlen(self._inner.cast()) + 1);
                assume!(
                    !bytes.is_empty() && bytes[bytes.len() - 1] == 0,
                    "CStr does not end with null byte"
                );

                &*(bytes as *const [u8] as *const core::ffi::CStr)
            }
        }

        /// Creates a `CStr` from a pointer to its first byte.
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
                    let bytes = slice::from_raw_parts(p, len + 1);
                    !bytes.is_empty() && bytes[len] == 0
                },
                "CStr does not end with null byte"
            );

            CStr { _inner: p, _marker: PhantomData }
        }
    }
}
