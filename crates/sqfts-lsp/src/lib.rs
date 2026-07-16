//! SQFts language server library (for tests).

#![allow(missing_docs)]

pub mod backend;
pub mod line_index;

pub use backend::Backend;
pub use line_index::{identifier_at, LineIndex};
