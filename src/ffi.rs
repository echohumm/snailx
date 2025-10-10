use core::ffi::{c_char};

#[allow(non_camel_case_types)]
type size_t = usize;

unsafe extern "C" {
    pub fn strlen(s: *const c_char) -> size_t;
}
