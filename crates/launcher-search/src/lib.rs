//! Platform-independent launcher search. Native scanning and actions are separate.
pub mod fuzzy;
pub mod ranking;
pub use ranking::{Contribution, Item, ItemKind, ResultRow, SearchContext, browse, merge, search};
pub mod picker;
pub mod reference_catalog;
