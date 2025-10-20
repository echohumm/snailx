// TODO: try to implement try_fold/_rfold for both

pub mod args;
pub mod mapped_args;

#[allow(clippy::inline_always)]
pub mod helpers {
    /// Helper to get the unsigned remaining length between two pointers.
    ///
    /// # Safety
    ///
    /// See [`<*const *const u8>::offset_from`].
    // apparently the old implementation was painfully slow lol
    #[allow(clippy::cast_sign_loss, clippy::must_use_candidate, missing_docs)]
    #[inline]
    pub unsafe fn len(cur: *const *const u8, end: *const *const u8) -> usize {
        end.offset_from(cur) as usize
    }
}
