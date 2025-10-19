pub mod args;
pub mod mapped_args;

// TODO: make sure all nths are correct

#[allow(clippy::inline_always)]
pub mod helpers {
    /// Helper to get the unsigned remaining length between two pointers.
    // apparently the old implementation was painfully slow lol
    #[allow(clippy::checked_conversions, clippy::must_use_candidate, missing_docs)]
    #[inline]
    pub unsafe fn len(cur: *const *const u8, end: *const *const u8) -> usize {
        #[allow(clippy::cast_sign_loss)]
        unsafe {
            end.offset_from(cur) as usize
        }
    }
}
