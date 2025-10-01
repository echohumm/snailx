use {
    crate::{
        direct,
        iter::{args::Args, mapped_args::MappedArgs}
    },
    core::{ffi::CStr, slice}
};
// TODO: maybe make types non-send/sync like stdlib does

/// Returns an iterator over the program's arguments as <code>&'static [CStr]</code>s.
#[must_use]
#[inline]
// cold because these are usually called once at startup
#[cold]
pub fn args() -> Args {
    let (argc, argv) = direct::argc_argv();
    Args { cur: argv, end: back(argv, argc) }
}

/// Returns an iterator over the program's arguments as the result of applying `map` to each
/// argument.
#[must_use]
#[inline]
#[cold]
pub fn map_args<Ret, F: Fn(&'static CStr) -> Option<Ret>>(map: F) -> MappedArgs<Ret, F> {
    let (argc, argv) = direct::argc_argv();
    MappedArgs { cur: argv, end: back(argv, argc), map }
}

pub(crate) fn try_to_str(s: &'static CStr) -> Option<&'static str> {
    // if let Ok(s) = s.to_str() {
    // 		Some(s)
    // 	} else {
    // 		// TODO: determine the behavior when a CStr contains invalid UTF-8
    // 		None
    // 	}
    s.to_str().ok()
}

/// Returns an iterator over the program's arguments as <code>&'static [str](core::str)</code>s.
#[inline]
#[cold]
pub fn str_args() -> MappedArgs<&'static str, fn(&'static CStr) -> Option<&'static str>> {
    map_args(try_to_str)
}

#[cfg(feature = "std")]
#[allow(clippy::unnecessary_wraps)]
#[inline]
fn to_osstr(s: &'static CStr) -> Option<&'static std::ffi::OsStr> {
    unsafe {
        #[allow(clippy::strlen_on_c_strings)]
        Some(
            &*(core::ptr::slice_from_raw_parts(s.as_ptr(), libc::strlen(s.as_ptr()))
                as *const std::ffi::OsStr)
        )
    }
}

#[cfg(feature = "std")]
/// Returns an iterator over the program's arguments as
/// <code>&'static [OsStr](std::ffi::OsStr)</code>s.
#[inline]
#[cold]
pub fn osstr_args()
-> MappedArgs<&'static std::ffi::OsStr, fn(&'static CStr) -> Option<&'static std::ffi::OsStr>> {
    map_args(to_osstr)
}

/// Returns a slice of pointers to the program's arguments.
#[must_use]
#[inline]
#[cold]
pub fn arg_ptrs() -> &'static [*const u8] {
    let (argc, argv) = direct::argc_argv();
    assume!(argv as usize != 0 || argc == 0);

    if argc == 0 {
        return &[];
    }

    unsafe { slice::from_raw_parts(argv, argc as usize) }
}

/// Returns a slice of <code>&'static [CStr]</code>s.
#[must_use]
#[inline]
#[cold]
pub fn args_slice() -> &'static [&'static CStr] {
    let (argc, argv) = direct::argc_argv();
    assume!(argv as usize != 0 || argc == 0);

    if argc == 0 {
        return &[];
    }

    unsafe { slice::from_raw_parts(argv.cast::<&'static CStr>(), argc as usize) }
}

#[cold]
fn back(argv: *const *const u8, argc: u32) -> *const *const u8 {
    assume!(re, argv as usize != 0 || argc == 0);
    // point to one-past-the-last element to follow standard exclusive-end iteration semantics
    unsafe { argv.add(argc as usize) }
}
