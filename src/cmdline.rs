use {
    crate::{
        direct,
        ffi::minimal_cstr::CStr,
        iter::{args::Args, mapped_args::MappedArgs}
    },
    core::slice
};
// TODO: maybe make types non-send/sync like stdlib does. will require adding a field bc !bounds
//  aren't stable

/// Returns an iterator over the program's arguments as `&'static CStr`.
#[must_use]
#[inline]
// cold because these are usually called once at startup
#[cfg_attr(not(feature = "no_cold"), cold)]
pub fn args() -> Args {
    let (argc, argv) = direct::argc_argv();
    Args { cur: argv, end: back(argv, argc) }
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
    MappedArgs { cur: argv, end: back(argv, argc), map }
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) mod helpers {
    use {
        crate::ffi::strlen,
        core::{mem::transmute, slice}
    };

    #[allow(clippy::must_use_candidate, clippy::not_unsafe_ptr_arg_deref, missing_docs)]
    pub fn try_to_str(p: *const u8) -> Option<&'static str> {
        unsafe {
            assume!(!p.is_null());
            let len = strlen(p.cast());
            let bytes = slice::from_raw_parts(p, len + 1);
            assume!(!bytes.is_empty() && bytes[len] == 0, "CStr does not end with null byte");

            let str_bytes = slice::from_raw_parts(p, len);

            #[cfg(not(feature = "assume_valid_str"))]
            if crate::str_checks::is_valid_utf8(str_bytes) {
                #[allow(clippy::transmute_bytes_to_str)]
                Some(transmute::<&'static [u8], &'static str>(str_bytes))
            } else {
                None
            }

            #[cfg(feature = "assume_valid_str")]
            {
                assume!(
                    dbg,
                    crate::str_checks::is_valid_utf8(str_bytes),
                    "invalid UTF-8 in CStr during conversion to str"
                );
                #[allow(clippy::transmute_bytes_to_str)]
                Some(transmute::<&'static [u8], &'static str>(str_bytes))
            }
        }
    }

    #[cfg(feature = "std")]
    #[allow(clippy::unnecessary_wraps)]
    #[inline]
    pub fn to_osstr(p: *const u8) -> Option<&'static std::ffi::OsStr> {
        unsafe {
            assume!(!p.is_null());
            let len = strlen(p.cast());
            assume!(!len == 0);
            Some(&*(core::ptr::slice_from_raw_parts(p, len) as *const std::ffi::OsStr))
        }
    }
}

/// Returns an iterator over the program's arguments as `&'static str`. Non-UTF-8 arguments are
/// skipped.
#[allow(clippy::inline_always)]
#[inline(always)]
#[cfg_attr(not(feature = "no_cold"), cold)]
pub fn str_args() -> MappedArgs<&'static str, fn(*const u8) -> Option<&'static str>> {
    map_args(helpers::try_to_str)
}

#[cfg(feature = "std")]
/// Returns an iterator over the program's arguments as`&'static std::ffi::OsStr`. Requires the
/// `std` feature.
#[allow(clippy::inline_always)]
#[inline(always)]
#[cfg_attr(not(feature = "no_cold"), cold)]
pub fn osstr_args()
-> MappedArgs<&'static std::ffi::OsStr, fn(*const u8) -> Option<&'static std::ffi::OsStr>> {
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
pub fn arg_ptrs() -> &'static [*const u8] {
    let (argc, argv) = direct::argc_argv();
    assume!(argv as usize != 0 || argc == 0, "argc is nonzero but argv is null");

    if argc == 0 {
        return &[];
    }

    unsafe { slice::from_raw_parts(argv, argc as usize) }
}

/// Returns a slice of `&'static CStr`.
///
/// Prefer [`arg_ptrs`] or one of the other iterators. The references in this slice carry
/// incorrect pointer metadata, which makes many `CStr` methods unsound.
///
/// # Safety
///
/// - Do not call any `CStr` method that relies on pointer metadata (length/bytes), such as
///   `to_bytes`, `to_bytes_with_nul`, `to_str`, `bytes`, `count_bytes`, formatting
///   (`Display`/`Debug`), indexing, or comparison methods.
/// - Treat the returned `&'static CStr` values as opaque handles; do not inspect their contents.
///
/// If you're not absolutely sure you need this API, avoid it. Please just use [`arg_ptrs`].
#[must_use]
#[inline]
#[cfg_attr(not(feature = "no_cold"), cold)]
pub unsafe fn args_slice() -> &'static [&'static CStr] {
    let (argc, argv) = direct::argc_argv();
    assume!(argv as usize != 0 || argc == 0, "argc is nonzero but argv is null");

    if argc == 0 {
        return &[];
    }

    slice::from_raw_parts(argv.cast::<&'static CStr>(), argc as usize)
}

#[cfg_attr(not(feature = "no_cold"), cold)]
fn back(argv: *const *const u8, argc: u32) -> *const *const u8 {
    assume!(re, argv as usize != 0 || argc == 0);
    // point to one-past-the-last element to follow standard exclusive-end iteration semantics
    unsafe { argv.add(argc as usize) }
}
