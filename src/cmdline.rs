use {
    crate::{
        direct,
        iter::{args::Args, mapped_args::MappedArgs}
    },
    core::{ffi::CStr, slice}
};
// TODO: maybe make types non-send/sync like stdlib does. will require adding a field bc !bounds 
//  aren't stable

/// Returns an iterator over the program's arguments as `&'static core::ffi::CStr`.
#[must_use]
#[inline]
// cold because these are usually called once at startup
#[cfg_attr(not(feature = "bench"), cold)]
pub fn args() -> Args {
    let (argc, argv) = direct::argc_argv();
    Args { cur: argv, end: back(argv, argc) }
}

/// Returns an iterator that applies `map` to each argument (`&'static CStr`). If `map` returns 
/// `None`, that argument is skipped.
#[must_use]
#[inline]
#[cfg_attr(not(feature = "bench"), cold)]
pub fn map_args<Ret, F: Fn(&'static CStr) -> Option<Ret>>(map: F) -> MappedArgs<Ret, F> {
    let (argc, argv) = direct::argc_argv();
    MappedArgs { cur: argv, end: back(argv, argc), map }
}

pub(crate) fn try_to_str(s: &'static CStr) -> Option<&'static str> {
    s.to_str().ok()
}

/// Returns an iterator over the program's arguments as `&'static str`. Non-UTF-8 arguments are 
/// skipped.
#[inline]
#[cfg_attr(not(feature = "bench"), cold)]
pub fn str_args() -> MappedArgs<&'static str, fn(&'static CStr) -> Option<&'static str>> {
    map_args(try_to_str)
}

#[cfg(feature = "std")]
#[allow(clippy::unnecessary_wraps)]
#[inline]
fn to_osstr(s: &'static CStr) -> Option<&'static std::ffi::OsStr> {
    unsafe {
        Some(
            &*(core::ptr::slice_from_raw_parts(s.as_ptr(), crate::ffi::strlen(s.as_ptr()))
                as *const std::ffi::OsStr)
        )
    }
}

#[cfg(feature = "std")]
/// Returns an iterator over the program's arguments as`&'static std::ffi::OsStr`. Requires the 
/// `std` feature.
#[inline]
#[cfg_attr(not(feature = "bench"), cold)]
pub fn osstr_args()
-> MappedArgs<&'static std::ffi::OsStr, fn(&'static CStr) -> Option<&'static std::ffi::OsStr>> {
    map_args(to_osstr)
}

/// Returns the raw argv as a slice of pointers: `&'static [*const u8]`.
///
/// The slice references the OS-provided storage and should usually not be mutated.
#[must_use]
#[inline]
#[cfg_attr(not(feature = "bench"), cold)]
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
/// If you're not absolutely sure you need this API, avoid it.
#[must_use]
#[inline]
#[cfg_attr(not(feature = "bench"), cold)]
pub unsafe fn args_slice() -> &'static [&'static CStr] {
    let (argc, argv) = direct::argc_argv();
    assume!(argv as usize != 0 || argc == 0, "argc is nonzero but argv is null");

    if argc == 0 {
        return &[];
    }

    unsafe { slice::from_raw_parts(argv.cast::<&'static CStr>(), argc as usize) }
}

#[cfg_attr(not(feature = "bench"), cold)]
fn back(argv: *const *const u8, argc: u32) -> *const *const u8 {
    assume!(re, argv as usize != 0 || argc == 0);
    // point to one-past-the-last element to follow standard exclusive-end iteration semantics
    unsafe { argv.add(argc as usize) }
}
