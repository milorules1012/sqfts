//! SQFts type checker.

#![allow(missing_docs)]

mod assignability;
mod checker;
mod config;
mod decls;
mod diagnostics;
mod symbols;

pub use checker::{check_source, CheckResult};
pub use config::CheckFlags;
pub use decls::{load_declarations, load_one, DeclError, DeclarationSet};
pub use diagnostics::{Diagnostic, Severity, StsCode};
pub use symbols::{FunctionSig, InterfaceMember, SymbolTable};
