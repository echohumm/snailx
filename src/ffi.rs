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
    use {
        crate::ffi::strlen,
        core::{slice},
        super::c_char
    };

    /// A very minimal CStr implementation, meant for use in place of `core::ffi::CStr`, which was
    /// unstable before 1.64, and `std::ffi::CStr` which requires `std`.
    pub struct CStr {
        // TODO: just hold a *const c_char so that args_slice can be safe
        _inner: [c_char]
    }

    #[allow(clippy::inline_always)]
    impl CStr {
        #[cfg(all(feature = "std", not(feature = "to_core_cstr")))]
        /// Converts this value into the `std` equivalent.
        #[must_use]
        #[inline(always)]
        pub fn to_stdlib(&self) -> &std::ffi::CStr {
            unsafe { &*(self as *const CStr as *const std::ffi::CStr) }
        }

        #[cfg(feature = "to_core_cstr")]
        /// Converts this value into the `core` equivalent.
        #[must_use]
        #[inline(always)]
        pub const fn to_stdlib(&self) -> &core::ffi::CStr {
            unsafe { &*(self as *const CStr as *const core::ffi::CStr) }
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
        #[inline]
        pub unsafe fn from_ptr<'a>(p: *const u8) -> &'a CStr {
            let len = strlen(p.cast());

            assume!(!p.is_null());
            let bytes = slice::from_raw_parts(p, len + 1);
            assume!(!bytes.is_empty() && bytes[len] == 0, "CStr does not end with null byte");

            &*(bytes as *const [u8] as *const CStr)
        }
    }
}
