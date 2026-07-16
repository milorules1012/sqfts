//! SQFts project configuration, file discovery, and shared check session.

#![allow(missing_docs)]

mod config;
mod declgen;
mod discovery;
mod project;

pub use config::{DeclgenConfig, SqftsConfig};
pub use declgen::{generate_declarations, DeclgenOptions, GeneratedDecl};
pub use discovery::{collect_decls, collect_sources};
pub use project::Project;
