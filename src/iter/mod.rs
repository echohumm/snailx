pub mod args;
pub mod mapped_args;

#[allow(clippy::inline_always)]
pub mod helpers {
    use core::{ffi::CStr, mem::size_of, slice};
    use crate::ffi::strlen;

    #[inline(always)]
    pub fn cstr(p: *const *const u8) -> &'static CStr {
        unsafe {
            assume!(!p.is_null());
            let p = p.read();
            let bytes = slice::from_raw_parts(p, strlen(p.cast()) + 1);
            assume!(
                !bytes.is_empty() && bytes[bytes.len() - 1] == 0,
                "CStr does not end with null byte"
            );

            &*(bytes as *const [u8] as *const CStr)
        }
    }

    // #[inline(always)]
    // pub fn cstr_r(p: *const u8) -> &'static CStr {
    //     unsafe {
    //         assume!(!p.is_null());
    //         let bytes = slice::from_raw_parts(p, strlen(p.cast()) + 1);
    //         assume!(
    //             !bytes.is_empty() && bytes[bytes.len() - 1] == 0,
    //             "CStr does not end with null byte"
    //         );
    // 
    //         &*(bytes as *const [u8] as *const CStr)
    //     }
    // }

    // // used because for some reason this is faster for nth, but slower for iteration?
    // #[inline(always)]
    // pub fn cstr_nth(p: *const *const u8) -> &'static CStr {
    //     unsafe {
    //         assume!(!p.is_null());
    //         CStr::from_ptr(p.read().cast())
    //     }
    // }

    // does the same thing as back.offset_from_unsigned(current) because it wasn't stable until 1.87
    #[allow(clippy::checked_conversions)]
    #[inline(always)]
    pub fn len(cur: *const *const u8, end: *const *const u8) -> usize {
        assume!(end as usize >= cur as usize, "ptr::len requires `back >= current`");
        let byte_diff = (end as usize).wrapping_sub(cur as usize);

        // strangely, making this const nets a 4% performance loss
        let elem_size = size_of::<*const u8>();
        assume!(elem_size.is_power_of_two());

        assume!(byte_diff <= (isize::MAX as usize), "distance must be <= isize::MAX bytes");

        byte_diff >> elem_size.trailing_zeros()
    }

    #[inline(always)]
    pub fn sz_hnt(cur: *const *const u8, end: *const *const u8) -> (usize, Option<usize>) {
        let len = len(cur, end);
        (len, Some(len))
    }

    #[rustversion::before(1.79)]
    #[inline(always)]
    pub unsafe fn unchecked_add(a: &mut usize, b: usize) {
        assume!(a.checked_add(b).is_some(), "integer overflow");
        *a += b;
    }
    #[rustversion::since(1.79)]
    #[inline(always)]
    pub unsafe fn unchecked_add(a: &mut usize, b: usize) {
        assume!(a.checked_add(b).is_some(), "integer overflow");
        *a = a.unchecked_add(b);
    }
}
