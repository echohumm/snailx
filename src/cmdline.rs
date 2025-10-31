// TODO: this module is no longer necessary except for the below helpers, remove cmdline and put
//  the below helpers somewhere else
#[allow(clippy::redundant_pub_crate)]
pub(crate) mod helpers {
    import! {
        {
            mem::transmute,
            option::Option::{self, Some},
            slice
        }
    }
    use crate::ffi::strlen;

    #[inline]
    #[allow(
        clippy::must_use_candidate,
        clippy::not_unsafe_ptr_arg_deref,
        clippy::transmute_bytes_to_str,
        missing_docs
    )]
    pub fn try_to_str(p: *const u8) -> Option<&'static str> {
        // SAFETY: only called internally with valid CStr pointers from argv
        unsafe {
            assume!(!p.is_null());
            let len = strlen(p.cast());
            let bytes = slice::from_raw_parts(p, len + 1);
            assume!(!bytes.is_empty() && bytes[len] == 0, "CStr does not end with null byte");

            let str_bytes = slice::from_raw_parts(p, len);

            #[cfg(not(feature = "assume_valid_str"))]
            if crate::str_checks::is_valid_utf8(str_bytes) {
                Some(transmute::<&'static [u8], &'static str>(str_bytes))
            } else {
                switch!(core::option::Option::None)
            }

            #[cfg(feature = "assume_valid_str")]
            {
                assume!(
                    dbg,
                    crate::str_checks::is_valid_utf8(str_bytes),
                    "invalid UTF-8 in CStr during conversion to str"
                );
                Some(transmute::<&'static [u8], &'static str>(str_bytes))
            }
        }
    }

    #[cfg(feature = "std")]
    #[inline]
    #[allow(
        clippy::unnecessary_wraps,
        clippy::must_use_candidate,
        missing_docs,
        unused_qualifications
    )]
    pub fn to_osstr(p: *const u8) -> Option<&'static ::std::ffi::OsStr> {
        // SAFETY: only called internally with valid CStr pointers from argv
        unsafe {
            assume!(!p.is_null());
            let len = strlen(p.cast());
            assume!(!len == 0);
            Some(&*(switch!(core::ptr::slice_from_raw_parts(p, len)) as *const ::std::ffi::OsStr))
        }
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    #[cfg_attr(not(feature = "no_cold"), cold)]
    pub(crate) fn back(argv: *const *const u8, argc: u32) -> *const *const u8 {
        assume!(!argv.is_null(), "argv is null");
        // SAFETY: argv points to a valid slice of argc count pointers, this is one past the last
        // but always decremented before deref
        unsafe { argv.add(argc as usize) }
    }

    // pub fn front(argv: *const *const u8, argc: u32) -> *const *const u8 {
    //     assume!(!argv.is_null(), "argv is null");
    //
    //     if argc == 0 {
    //         return argv;
    //     } else {
    //         // SAFETY: if argc != 0, argv != null. this is one before the first element, but
    // always         // incremented before deref
    //         unsafe {
    //             argv.sub(1)
    //         }
    //     }
    // }
}
