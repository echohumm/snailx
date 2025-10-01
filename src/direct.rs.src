#![allow(clippy::inline_always)]
//! The `direct` module provides direct access to argc and argv.

/// Returns `(argc, argv)`, where `argc` is the number of arguments and `argv` is a pointer to an
/// array of pointers to null-terminated strings, the program arguments.
#[must_use]
#[inline(always)]
#[cold]
pub fn argc_argv() -> (u32, *const *const u8) {
    imp::argc_argv()
}

/// Sets the value of `argc` and `argv`.
///
/// # Safety
///
/// The caller must ensure it is safe to modify `argc` and `argv` and no concurrent access is
/// occurring.
#[inline(always)]
#[cold]
pub unsafe fn set_argc_argv(argc: u32, argv: *const *const u8) -> (u32, *const *const u8) {
	imp::set_argc_argv(argc, argv)
}

macro_rules! cfgr {
    ($($after:tt)*) => {
		#[allow(unexpected_cfgs)]
		#[cfg(any(
			target_os = "linux",
			target_os = "android",
			target_os = "freebsd",
			target_os = "dragonfly",
			target_os = "netbsd",
			target_os = "openbsd",
			target_os = "cygwin",
			target_os = "solaris",
			target_os = "illumos",
			target_os = "emscripten",
			target_os = "haiku",
			target_os = "hermit",
			target_os = "l4re",
			target_os = "fuchsia",
			target_os = "redox",
			target_os = "vxworks",
			target_os = "horizon",
			target_os = "aix",
			target_os = "nto",
			target_os = "hurd",
			target_os = "rtems",
			target_os = "nuttx",
		))]
		$($after)*
	};
}

cfgr! {
    pub(crate) mod imp {
    use {
            core::{
                ptr,
                sync::atomic::{AtomicU32, AtomicPtr, Ordering}
            },
            libc::{c_int, c_uint},
        };

        static ARGC: AtomicU32 = AtomicU32::new(0);
        static ARGV: AtomicPtr<*const u8> = AtomicPtr::new(ptr::null_mut());
		
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        #[used]
		#[unsafe(link_section = ".init_array.00099")]
        static INIT: extern "C" fn(c_int, *const *const u8, *const *const u8) = {
            extern "C" fn init_wrapper(
                argc: c_int,
                argv: *const *const u8,
                _: *const *const u8
            ) {
				#[allow(clippy::cast_sign_loss)]
                ARGC.store(argc as c_uint, Ordering::Relaxed);
                ARGV.store(argv as *mut *const u8, Ordering::Relaxed);
            }
            init_wrapper
        };

		#[inline(always)]
		#[cold]
        pub fn argc_argv() -> (u32, *const *const u8) {
            let argv = ARGV.load(Ordering::Relaxed);

            (if argv.is_null() { 0 } else { ARGC.load(Ordering::Relaxed) }, argv.cast())
        }

		pub unsafe fn set_argc_argv(argc: u32, argv: *const *const u8) -> (u32, *const *const u8) {
			let old_argv = ARGV.swap(argv as *mut _, Ordering::Relaxed) as *const *const u8;
			let old_argc = ARGC.swap(argc as c_uint, Ordering::Relaxed);

			(if old_argv.is_null() { 0 } else { old_argc }, old_argv)
		}
    }
}

#[cfg(target_vendor = "apple")]
pub(crate) mod imp {
    use libc::{c_char, c_int};
	use crate::error::Error;

    unsafe extern "C" {
        fn _NSGetArgc() -> *mut c_int;
        fn _NSGetArgv() -> *mut *mut *mut c_char;
    }

	#[inline(always)]
	#[cold]
    pub fn argc_argv() -> (u32, *const *const u8) {
        unsafe { (_NSGetArgc().read() as u32, _NSGetArgv().read().cast()) }
    }

	pub unsafe fn set_argc_argv(argc: u32, argv: *const *const u8) -> (u32, *const *const u8) {
		let old_argv = _NSGetArgv();
		let old_argc = _NSGetArgc();

		old_argv.write(argv as *mut _);
		old_argc.write(argc as c_int);

		(old_argc.read() as u32, old_argv.read().cast())
	}
}

#[cfg(windows)]
pub(crate) mod imp {
	#[inline(always)]
	#[cold]
	pub fn argc_argv() -> (u32, *const *const u8) {
		compile_error!("windows is not yet supported. when or if it is, it will not be \
		zero-allocation like as is the purpose of this crate.");
		(0, ptr::null_mut())
	}

	pub unsafe fn set_argc_argv(argc: u32, argv: *const *const u8) -> (u32, *const *const u8) {
		(0, ptr::null_mut())
	}
}
