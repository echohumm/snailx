#[cfg(feature = "indexing_parser")]
/// A parser which indexes arguments, but requires allocation, and all related datatypes.
pub mod indexing;
#[cfg(feature = "non_indexing_parser")]
/// A parser which doesn't index arguments or allocate, but requires redundant work, and all related
/// datatypes.
pub mod non_indexing;

// pub trait Parser {
//     #[must_use]
//     fn positional(&self, idx: usize) -> Option<&'static str>;
//     
//     #[must_use]
//     fn flag(&self, name: &'static str) -> bool;
//
//     #[must_use]
//     fn option(&self, name: &'static str) -> Option<OptValues>;
// }
