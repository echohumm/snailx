use crate::{
    direct,
    ffi::minimal_cstr::CStr,
    iter::{args::Args, mapped_args::MappedArgs}
};

import! {
    use core::{slice, ops::Fn, option::Option, marker::Copy}
}

/// Returns an iterator over the program's arguments as <code>[CStr](CStr)<'static></code>.
#[must_use]
#[inline]
// cold because these are usually called once at startup
#[cfg_attr(not(feature = "no_cold"), cold)]
pub fn args() -> Args {
    let (argc, argv) = direct::argc_argv();
    Args { cur: argv, end: helpers::back(argv, argc) }
}

/// Returns an iterator that applies `map` to each argument (`*const u8`). If `map` returns
/// `None`, that argument is skipped.
#[must_use]
#[inline]
#[cfg_attr(not(feature = "no_cold"), cold)]
pub fn map_args<Ret, F: Fn(*const u8) -> Option<Ret> + Copy + 'static>(
    map: F
) -> MappedArgs<Ret, F> {
    let (argc, argv) = direct::argc_argv();
    MappedArgs { cur: argv, end: helpers::back(argv, argc), map }
}

/// Returns an iterator over the program's arguments as `&'static str`. Non-UTF-8 arguments are
/// skipped.
#[must_use]
#[allow(clippy::inline_always)]
#[inline(always)]
#[cfg_attr(not(feature = "no_cold"), cold)]
// TODO: better name for this than current/orig(str_args)
pub fn args_utf8() -> MappedArgs<&'static str, fn(*const u8) -> Option<&'static str>> {
    map_args(helpers::try_to_str)
}

/// Returns an iterator over the program's arguments as `&'static std::ffi::OsStr`. Requires the
/// `std` feature.
#[cfg(feature = "std")]
#[must_use]
#[allow(clippy::inline_always)]
#[inline(always)]
#[cfg_attr(not(feature = "no_cold"), cold)]
pub fn args_os()
-> MappedArgs<&'static ::std::ffi::OsStr, fn(*const u8) -> Option<&'static ::std::ffi::OsStr>> {
    map_args(helpers::to_osstr)
}

/// Returns the raw argv as a slice of pointers: `&'static [*const u8]`.
///
/// The slice references the OS-provided storage and should usually not be mutated.
///
/// This is a simpler way to iterate over the elements, if preferred.
#[must_use]
#[inline]
#[cfg_attr(not(feature = "no_cold"), cold)]
pub fn argv_ptrs() -> &'static [*const u8] {
    let (argc, argv) = direct::argc_argv();
    assume!(!argv.is_null() || argc == 0, "argc is nonzero but argv is null");

    if argc == 0 {
        return &[];
    }

    // SAFETY: argv points to a valid slice of argc count pointers
    unsafe { slice::from_raw_parts(argv, argc as usize) }
}

/// Returns a slice of <code>[CStr](CStr)<'static></code>.
#[must_use]
#[inline]
#[cfg_attr(not(feature = "no_cold"), cold)]
pub fn args_slice() -> &'static [CStr<'static>] {
    let (argc, argv) = direct::argc_argv();
    assume!(!argv.is_null() || argc == 0, "argc is nonzero but argv is null");

    if argc == 0 {
        return &[];
    }

    // SAFETY: argv points to a valid slice of argc count pointers, CStr is repr(transparent) around
    // a pointer
    unsafe { slice::from_raw_parts(argv.cast::<CStr<'static>>(), argc as usize) }
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) mod helpers {
    use crate::ffi::strlen;

    import! {
        use core::{mem::transmute, slice, option::Option::{self, Some}}
    }

    #[allow(
        clippy::must_use_candidate,
        clippy::not_unsafe_ptr_arg_deref,
        clippy::transmute_bytes_to_str,
        missing_docs
    )]
    #[inline]
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
    #[allow(clippy::unnecessary_wraps)]
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
    pub fn back(argv: *const *const u8, argc: u32) -> *const *const u8 {
        assume!(!argv.is_null() || argc == 0, "argc is nonzero but argv is null");
        // SAFETY: argv points to a valid slice of argc count pointers, this is one past the last
        // but always decremented before deref
        unsafe { argv.add(argc as usize) }
    }
    
    // pub fn front(argv: *const *const u8, argc: u32) -> *const *const u8 {
    //     assume!(!argv.is_null() || argc == 0, "argc is nonzero but argv is null");
    //
    //     if argc == 0 {
    //         return argv;
    //     } else {
    //         // SAFETY: if argc != 0, argv != null. this is one before the first element, but always
    //         // incremented before deref
    //         unsafe {
    //             argv.sub(1)
    //         }
    //     }
    // }
}
