pub mod args;
pub mod mapped_args;

#[allow(clippy::inline_always)]
pub mod helpers {
    use core::mem::size_of;

    // does the same thing as back.offset_from_unsigned(current) because it wasn't stable until 1.87
    #[allow(clippy::checked_conversions, clippy::must_use_candidate, missing_docs)]
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
}
