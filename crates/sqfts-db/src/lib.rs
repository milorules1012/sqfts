//! Engine-command type database for SQFts.

#![allow(missing_docs)]

mod convert;
mod database;

pub use convert::{wiki_name_to_type, wiki_value_to_type};
pub use database::{load_shared, CallKind, CommandDb, Overload, ParamSig, SharedDb};
