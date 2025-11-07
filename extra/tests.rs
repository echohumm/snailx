#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::iter_nth_zero)]
extern crate core;
extern crate snailx;

use snailx::{CStr, bench_helpers::strlen};

// used as it is a documented safety condition (which tests decide to violate by default) that
//  `set_argc_argv()` is called while there is no other concurrent access.
static GLOBAL_LOCK: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

const ARG_SET_0: [*const u8; 0] = [];

const ARG_SET_1: [*const u8; 1] = ["bash\0".as_ptr()];

const ARG_SET_7: [*const u8; 7] = [
    "some\0".as_ptr(),
    "random\0".as_ptr(),
    "text\0".as_ptr(),
    "for\0".as_ptr(),
    "testing\0".as_ptr(),
    "purposes\0".as_ptr(),
    "only\0".as_ptr()
];

const ARG_SET_8: [*const u8; 8] = [
    "some\0".as_ptr(),
    "random\0".as_ptr(),
    "text\0".as_ptr(),
    "for\0".as_ptr(),
    "testing\0".as_ptr(),
    "purposes\0".as_ptr(),
    "only\0".as_ptr(),
    "plus one\0".as_ptr()
];

const ARG_SET_SPEC: [*const u8; 2] =
    ["nerdfont+ half-battery\\charging: 󰢞\0".as_ptr(), "zero-width(space; \"\u{200B}\"\0".as_ptr()];

const ARG_SET_LONG: [*const u8; 2] = [
    "very long argument which includes way too much randomly typed text. should this be \
     specifically made for this or random generated, perhaps something like lorem ipsum? \
     probably. will it? absolutely not. i'm just gonna keep typing until i run out of ideas. i'm \
     very tired right now.\0"
        .as_ptr(),
    concat!(include_str!("./bee_movie.txt"), "\0").as_ptr()
];

#[cfg(feature = "__test_cause_ub")]
const ARG_SET_BLANK: [*const u8; 1] = ["\0".as_ptr()];

const ARG_SET_NULL: [*const u8; 2] =
    ["this\0should\0be\0truncated\0".as_ptr(), "same\0here".as_ptr()];

const ARG_SET_UTF8: [[*const u8; 3]; 3] = [
    // valid
    ["test\0".as_ptr(), "one\0".as_ptr(), "hi\0".as_ptr()],
    // invalid
    [b"abc\xFF\0".as_ptr(), b"\xF0\x28\x8C\x28\0".as_ptr(), b"\x28\x8C\x28\xF0\0".as_ptr()],
    // mixed
    ["test\0".as_ptr(), b"\xF0\x28\x8C\x28\0".as_ptr(), "hi\0".as_ptr()]
];

unsafe fn set_args_empty() -> &'static [*const u8] {
    snailx::direct::set_argc_argv(ARG_SET_0.len() as u32, ARG_SET_0.as_ptr());
    &ARG_SET_0
}

unsafe fn set_args_one() -> &'static [*const u8] {
    snailx::direct::set_argc_argv(ARG_SET_1.len() as u32, ARG_SET_1.as_ptr());
    &ARG_SET_1
}

unsafe fn set_args_odd() -> &'static [*const u8] {
    snailx::direct::set_argc_argv(ARG_SET_7.len() as u32, ARG_SET_7.as_ptr());
    &ARG_SET_7
}
unsafe fn set_args_even() -> &'static [*const u8] {
    snailx::direct::set_argc_argv(ARG_SET_8.len() as u32, ARG_SET_8.as_ptr());
    &ARG_SET_8
}

unsafe fn set_args_spec() -> &'static [*const u8] {
    snailx::direct::set_argc_argv(ARG_SET_SPEC.len() as u32, ARG_SET_SPEC.as_ptr());
    &ARG_SET_SPEC
}

unsafe fn set_args_long() -> &'static [*const u8] {
    snailx::direct::set_argc_argv(ARG_SET_LONG.len() as u32, ARG_SET_LONG.as_ptr());
    &ARG_SET_LONG
}

#[cfg(feature = "__test_cause_ub")]
unsafe fn set_args_blank() -> &'static [*const u8] {
    snailx::direct::set_argc_argv(ARG_SET_BLANK.len() as u32, ARG_SET_BLANK.as_ptr());
    &ARG_SET_BLANK
}

unsafe fn set_args_null() -> &'static [*const u8] {
    snailx::direct::set_argc_argv(ARG_SET_NULL.len() as u32, ARG_SET_NULL.as_ptr());
    &ARG_SET_NULL
}

unsafe fn set_args_utf8(i: usize) -> &'static [*const u8] {
    let args = &ARG_SET_UTF8[i];
    snailx::direct::set_argc_argv(args.len() as u32, args.as_ptr());
    args
}

// helper macro to run tests on many different sets of arguments. useful for edge cases like
//  off-by-one on odd counts or oob access on 0 counts.
macro_rules! test_i {
    ($name:ident, $($body:tt)*) => {
        fn test_inner($name: &[*const u8]) {
            $(
                $body
            )*
        }

        // used to prevent panics from deadlocking the testing process
        struct PanicHandler {}
        impl PanicHandler {
            fn new() -> PanicHandler {
                while GLOBAL_LOCK.compare_exchange(false, true, core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst).is_err() {
                    core::hint::spin_loop();
                }
                PanicHandler {}
            }
        }
        impl Drop for PanicHandler {
            fn drop(&mut self) {
                GLOBAL_LOCK.store(false, core::sync::atomic::Ordering::SeqCst);
            }
        }

        unsafe {
            let panic_handler = PanicHandler::new();

            test_inner(set_args_odd());
            test_inner(set_args_even());
            test_inner(set_args_empty());
            test_inner(set_args_one());

            test_inner(set_args_spec());
            test_inner(set_args_long());
            #[cfg(feature = "__test_cause_ub")]
            test_inner(set_args_blank());
            test_inner(set_args_null());

            test_inner(set_args_utf8(0));

            // TODO: use other 2 utf8 sets as well

            // suppress warning and drop the handler to unlock
            drop(panic_handler);
        }
    }
}

// basic functionality tests

// prefixed with _ so it runs first just in case miri decides it doesn't like ffi like happened once
#[test]
fn _strlen_correct() {
    const TEST_STR: &str = "test123\0";
    const TEST_STR_2: &str = "test1234\0";

    assert_eq!(unsafe { strlen(TEST_STR.as_ptr()) }, TEST_STR.len() - 1);
    assert_eq!(unsafe { strlen(TEST_STR_2.as_ptr()) }, TEST_STR_2.len() - 1);
}

// count tests

#[test]
fn iter_count() {
    test_i! {
        a,
        let args = snailx::Args::new();

        let mut cnt = 0;
        for _ in args {
            cnt += 1;
        }
        assert_eq!(cnt, a.len());
        assert_eq!(snailx::Args::new().count(), cnt);
    }
}

#[test]
fn len() {
    test_i! {
        a,
        let args = snailx::Args::new();

        assert_eq!(args.len(), a.len());
    }
}

// content tests

#[test]
fn cstrs_correct() {
    test_i! {
        a,
        assert_eq!(
            snailx::Args::new().collect::<Vec<_>>().as_slice(),
            a
                .iter()
                .map(|&s| unsafe { CStr::from_ptr(s) })
                .collect::<Vec<_>>()
                .as_slice()
        );
    }
}

#[test]
fn slice_correct() {
    test_i! {
        a,
        assert_eq!(
            snailx::Args::new().as_slice(),
            a
                .iter()
                .map(|&s| unsafe { CStr::from_ptr(s) })
                .collect::<Vec<_>>()
                .as_slice()
        );
    }
}

#[test]
fn os_correct() {
    test_i! {
        a,
        assert_eq!(
            snailx::MappedArgs::osstr()
                .collect::<Vec<_>>()
                .as_slice(),
            a
                .iter()
                .map(|&s| snailx::bench_helpers::to_osstr(s).unwrap()).collect::<Vec<_>>()
        );
    }
}

#[test]
fn slice() {
    test_i! {
        a,
        let args = snailx::Args::new().as_slice();

        assert_eq!(args, a);
    }
}

#[test]
fn cstr_iter() {
    test_i! {
        a,
        let args = snailx::Args::new();

        for (i, arg) in args.enumerate() {
            assert_eq!(arg.to_stdlib(), unsafe { CStr::from_ptr(a[i]).to_stdlib() });
        }
    }
}

#[test]
fn os_iter() {
    test_i! {
        a,
        let args = snailx::MappedArgs::osstr();

        for (i, arg) in args.enumerate() {
            assert_eq!(arg, snailx::bench_helpers::to_osstr(a[i]).unwrap());
        }
    }
}

#[test]
fn os_count() {
    test_i! {
        a,
        let args = snailx::MappedArgs::osstr();

        let mut cnt = 0;
        for _ in args {
            cnt += 1;
        }
        assert_eq!(cnt, a.len());
        assert_eq!(snailx::MappedArgs::osstr().count(), cnt);
    }
}

#[test]
fn cstr_nth() {
    test_i! {
        a,
        let mut args = snailx::Args::new();

        // consumes first 3 if they exist
        if !a.is_empty() {
            assert_eq!(
                args.nth(0).unwrap().to_stdlib(),
                unsafe { CStr::from_ptr(a[0]).to_stdlib() }
            );
        }
        if a.len() > 2 {
            assert_eq!(
                args.nth(1).unwrap().to_stdlib(),
                unsafe { CStr::from_ptr(a[2]).to_stdlib() }
            );
        }

        // checks that the remaining count is correct
        if a.len() > 3 {
            assert!(args.nth(a.len().saturating_sub(3)).is_none());
        }
    }
}

#[test]
fn os_nth() {
    test_i! {
        a,
        let mut args = snailx::MappedArgs::osstr();

        if !a.is_empty() {
            assert_eq!(args.nth(0).unwrap(), snailx::bench_helpers::to_osstr(a[0]).unwrap());
        }
        if a.len() > 2 {
            assert_eq!(args.nth(1).unwrap(), snailx::bench_helpers::to_osstr(a[2]).unwrap());
        }

        if a.len() > 3 {
            assert!(args.nth(a.len().saturating_sub(3)).is_none());
        }
    }
}

#[test]
fn cstr_size_hint_and_len() {
    test_i! {
        a,
        let args = snailx::Args::new();

        assert_eq!(args.size_hint(), (a.len(), Some(a.len())));
        assert_eq!(args.len(), a.len());
    }
}

#[test]
fn utf8_size_hint() {
    test_i! {
        a,
        let args = snailx::MappedArgs::utf8();

        assert_eq!(args.size_hint(), (0, Some(a.len())));
    }
}

#[test]
fn os_size_hint_and_len() {
    test_i! {
        a,
        let args = snailx::MappedArgs::osstr();

        #[cfg(not(feature = "infallible_map"))]
        assert_eq!(args.size_hint(), (0, Some(a.len())));
        #[cfg(feature = "infallible_map")]
        assert_eq!(args.size_hint(), (a.len(), Some(a.len())));
    }
}

// TODO: test all utf8 tests with test_utf8, not just test_i/first 2 utf8 sets

// utf-8 validity tests

#[cfg(not(feature = "assume_valid_str"))]
macro_rules! test_utf8 {
    ($valid:ident, $slice:ident, $($body:tt)*) => {
        fn test_inner($valid: bool, $slice: &[*const u8]) {
            $(
                $body
            )*
        }

        unsafe {
            test_inner(true, set_args_utf8(0));
            test_inner(false, set_args_utf8(1));
        }
    };
}

#[test]
fn utf8_correct() {
    test_i! {
        a,
        assert_eq!(
            snailx::MappedArgs::utf8().collect::<Vec<_>>().as_slice(),
            a.iter().map(|&s| snailx::bench_helpers::try_to_str(s).unwrap())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn utf8_iter() {
    test_i! {
        a,
        let args = snailx::MappedArgs::utf8();

        for (i, arg) in args.enumerate() {
            assert_eq!(arg, snailx::bench_helpers::try_to_str(a[i]).unwrap());
        }
    }
}

#[test]
fn utf8_count() {
    test_i! {
        a,
        let args = snailx::MappedArgs::utf8();

        let mut cnt = 0;
        for _ in args {
            cnt += 1;
        }
        assert_eq!(cnt, a.len());
        assert_eq!(snailx::MappedArgs::utf8().count(), cnt);
    }
}

#[test]
fn utf8_nth() {
    test_i! {
        a,
        let mut args = snailx::MappedArgs::utf8();

        if !a.is_empty() {
            assert_eq!(args.nth(0).unwrap(), snailx::bench_helpers::try_to_str(a[0]).unwrap());
        }
        if a.len() > 2 {
            assert_eq!(args.nth(1).unwrap(), snailx::bench_helpers::try_to_str(a[2]).unwrap());
        }

        if a.len() > 3 {
            assert!(args.nth(a.len().saturating_sub(3)).is_none());
        }
    }
}

#[cfg(not(feature = "assume_valid_str"))]
#[test]
fn try_to_str() {
    test_utf8! {
        v, a,
        for &arg in a {
            if v {
                assert_eq!(
                    snailx::bench_helpers::try_to_str(arg),
                    Some(unsafe {
                        snailx::switch!(
                            core::str::from_utf8_unchecked(
                                core::slice::from_raw_parts(arg, strlen(arg))
                            )
                        )
                    })
                );
            } else {
                assert!(snailx::bench_helpers::try_to_str(arg).is_none());
            }
        }
    }
}

#[cfg(not(feature = "assume_valid_str"))]
#[test]
fn utf8_iter_no_invalid() {
    test_utf8! {
        v, a,
        let mut args = snailx::MappedArgs::utf8();

        for i in 0..=a.len() {
            let arg = args.next();

            if v && i < a.len() {
               assert_eq!(
                    arg,
                    Some(unsafe {
                        snailx::switch!(
                            core::str::from_utf8_unchecked(
                                core::slice::from_raw_parts(a[i], strlen(a[i]))
                            )
                        )
                    })
                );
            } else {
                assert!(arg.is_none());
            }
        }
    }
}

#[cfg(not(feature = "assume_valid_str"))]
#[test]
fn utf8_nth_no_invalid() {
    test_utf8! {
        v, a,
        let mut args = snailx::MappedArgs::utf8();

        let arg = args.nth(1);

        if v {
            assert_eq!(
                    arg,
                    Some(unsafe {
                        snailx::switch!(
                            core::str::from_utf8_unchecked(
                                core::slice::from_raw_parts(a[1], strlen(a[1]))
                            )
                        )
                    })
                );
        } else {
            assert!(arg.is_none());
        }
    }
}

#[cfg(not(feature = "assume_valid_str"))]
#[test]
fn utf8_skips_invalid() {
    let a = unsafe { set_args_utf8(2) };

    let mut args = snailx::MappedArgs::utf8();

    assert_eq!(
        args.next(),
        Some(unsafe {
            snailx::switch!(core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                a[0],
                strlen(a[0])
            )))
        })
    );
    assert_eq!(
        args.next(),
        Some(unsafe {
            snailx::switch!(core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                a[2],
                strlen(a[2])
            )))
        })
    );
    assert!(args.next().is_none());
}

// reversed iteration tests

#[cfg(feature = "rev_iter")]
#[test]
fn iter_count_back() {
    test_i! {
        a,
        let mut args = snailx::Args::new();

        let mut cnt = 0;
        while args.next_back().is_some() {
            cnt += 1;
        }
        assert_eq!(cnt, a.len());
        assert_eq!(snailx::Args::new().rev().count(), cnt);
    }
}

#[cfg(feature = "rev_iter")]
#[test]
fn cstr_iter_back() {
    test_i! {
        a,
        let mut args = snailx::Args::new();

        for i in (0..a.len()).rev() {
            let arg = args.next_back().unwrap();
            assert_eq!(arg.to_stdlib(), unsafe { CStr::from_ptr(a[i]).to_stdlib() });
        }
    }
}

#[cfg(feature = "rev_iter")]
#[test]
fn os_iter_back() {
    test_i! {
        a,
        let mut args = snailx::MappedArgs::osstr();

        for i in (0..a.len()).rev() {
            let arg = args.next_back().unwrap();
            assert_eq!(arg, snailx::bench_helpers::to_osstr(a[i]).unwrap());
        }
    }
}

#[cfg(feature = "rev_iter")]
#[test]
fn os_count_back() {
    test_i! {
        a,
        let mut args = snailx::MappedArgs::osstr();

        let mut cnt = 0;
        while args.next_back().is_some() {
            cnt += 1;
        }
        assert_eq!(cnt, a.len());
        assert_eq!(snailx::MappedArgs::osstr().rev().count(), cnt);
    }
}

#[cfg(feature = "rev_iter")]
#[test]
fn cstr_nth_back() {
    test_i! {
        a,
        let mut args = snailx::Args::new();

        if !a.is_empty() {
            assert_eq!(
                args.nth_back(0).unwrap().to_stdlib(),
                unsafe { CStr::from_ptr(a[a.len() - 1]).to_stdlib() }
            );
        }
        if a.len() > 2 {
            assert_eq!(
                args.nth_back(1).unwrap().to_stdlib(),
                unsafe { CStr::from_ptr(a[a.len() - 3]).to_stdlib() }
            );
        }

        if a.len() > 3 {
            assert!(args.nth_back(a.len().saturating_sub(3)).is_none());
        }
    }
}

#[cfg(feature = "rev_iter")]
#[test]
fn os_nth_back() {
    test_i! {
        a,
        let mut args = snailx::MappedArgs::osstr();

        if !a.is_empty() {
            assert_eq!(
                args.nth_back(0).unwrap(),
                snailx::bench_helpers::to_osstr(a[a.len() - 1]).unwrap()
            );
        }
        if a.len() > 2 {
            assert_eq!(
                args.nth_back(1).unwrap(),
                snailx::bench_helpers::to_osstr(a[a.len() - 3]).unwrap()
            );
        }

        if a.len() > 3 {
            assert!(args.nth_back(a.len().saturating_sub(3)).is_none());
        }
    }
}

#[cfg(feature = "rev_iter")]
#[test]
fn utf8_iter_back() {
    test_i! {
        a,
        let mut args = snailx::MappedArgs::utf8();

        for i in (0..a.len()).rev() {
            let arg = args.next_back().unwrap();
            assert_eq!(arg, snailx::bench_helpers::try_to_str(a[i]).unwrap());
        }
    }
}

#[cfg(feature = "rev_iter")]
#[test]
fn utf8_count_back() {
    test_i! {
        a,
        let mut args = snailx::MappedArgs::utf8();

        let mut cnt = 0;
        while args.next_back().is_some() {
            cnt += 1;
        }
        assert_eq!(cnt, a.len());
        assert_eq!(snailx::MappedArgs::utf8().rev().count(), cnt);
    }
}

#[cfg(feature = "rev_iter")]
#[test]
fn utf8_nth_back() {
    test_i! {
        a,
        let mut args = snailx::MappedArgs::utf8();

        if !a.is_empty() {
            assert_eq!(
                args.nth_back(0).unwrap(),
                snailx::bench_helpers::try_to_str(a[a.len() - 1]).unwrap()
            );
        }
        if a.len() > 2 {
            assert_eq!(
                args.nth_back(1).unwrap(),
                snailx::bench_helpers::try_to_str(a[a.len() - 3]).unwrap()
            );
        }

        if a.len() > 3 {
            assert!(args.nth_back(a.len().saturating_sub(3)).is_none());
        }
    }
}

// utf-8 reversed tests for invalid/mixed data

#[cfg(all(not(feature = "assume_valid_str"), feature = "rev_iter"))]
#[test]
fn utf8_iter_no_invalid_back() {
    test_utf8! {
        v, a,
        let mut args = snailx::MappedArgs::utf8();

        for i in (0..=a.len()).rev() {
            let arg = args.next_back();

            if v && i > 0 {
                let idx = i - 1;
                assert_eq!(
                    arg,
                    Some(unsafe {
                        snailx::switch!(core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                            a[idx],
                            strlen(a[idx])
                        )))
                    })
                );
            } else {
                assert!(arg.is_none());
            }
        }
    }
}

#[cfg(all(not(feature = "assume_valid_str"), feature = "rev_iter"))]
#[test]
fn utf8_nth_no_invalid_back() {
    test_utf8! {
        v, a,
        let mut args = snailx::MappedArgs::utf8();

        let arg = args.nth_back(1);

        if v {
            let idx = a.len() - 2;
            assert_eq!(
                arg,
                Some(unsafe {
                    snailx::switch!(core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                        a[idx],
                        strlen(a[idx])
                    )))
                })
            );
        } else {
            assert!(arg.is_none());
        }
    }
}

#[cfg(all(not(feature = "assume_valid_str"), feature = "rev_iter"))]
#[test]
fn utf8_skips_invalid_back() {
    let a = unsafe { set_args_utf8(2) };

    let mut args = snailx::MappedArgs::utf8();

    assert_eq!(
        args.next_back(),
        Some(unsafe {
            snailx::switch!(core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                a[2],
                strlen(a[2])
            )))
        })
    );
    assert_eq!(
        args.next_back(),
        Some(unsafe {
            snailx::switch!(core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                a[0],
                strlen(a[0])
            )))
        })
    );
    assert!(args.next_back().is_none());
}

// fold/rfold tests

#[test]
fn cstr_fold_count() {
    test_i! {
        a,

        let cnt = snailx::Args::new().fold(0usize, |acc, _| acc + 1);
        assert_eq!(cnt, a.len());
    }
}

#[cfg(feature = "rev_iter")]
#[test]
fn cstr_rfold_count() {
    test_i! {
        a,
        let cnt = snailx::Args::new().rfold(0usize, |acc, _| acc + 1);
        assert_eq!(cnt, a.len());
    }
}

#[test]
fn cstr_fold_strlen_sum() {
    test_i! {
        a,
        let total = snailx::Args::new().fold(0usize, |acc, c| {
            acc + unsafe { strlen(c.as_ptr()) }
        });
        let expect = a.iter().map(|&p| unsafe { strlen(p) }).sum::<usize>();
        assert_eq!(total, expect);
    }
}

#[cfg(feature = "rev_iter")]
#[test]
fn cstr_rfold_strlen_sum() {
    test_i! {
        a,
        let total = snailx::Args::new().rfold(0usize, |acc, c| {
            acc + unsafe { strlen(c.as_ptr()) }
        });
        let expect = a.iter().map(|&p| unsafe { strlen(p) }).sum::<usize>();
        assert_eq!(total, expect);
    }
}

#[test]
fn utf8_fold_collect_correct() {
    test_i! {
        a,
        let got = snailx::MappedArgs::utf8().fold(Vec::new(), |mut v, s| {
            v.push(s);
            v
        });
        let expect: Vec<_> = a
            .iter()
            .filter_map(|&p| snailx::bench_helpers::try_to_str(p))
            .collect();
        assert_eq!(got.as_slice(), expect.as_slice());
    }
}

#[cfg(feature = "rev_iter")]
#[test]
fn utf8_rfold_collect_correct() {
    test_i! {
        a,
        let got = snailx::MappedArgs::utf8().rfold(Vec::new(), |mut v, s| {
            v.push(s);
            v
        });
        let expect: Vec<_> = a
            .iter()
            .filter_map(|&p| snailx::bench_helpers::try_to_str(p))
            .rev()
            .collect();
        assert_eq!(got.as_slice(), expect.as_slice());
    }
}

// TODO: test parser
