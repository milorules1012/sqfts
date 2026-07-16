//! SQFts annotation scanner, type-expression parser, and byte-stable eraser.

#![allow(missing_docs)]

mod erase;
mod scan;
mod typ;
mod type_parser;

pub use erase::{
    erase, erase_scanned, erase_with_runtime_params, type_exemplars, EraseOptions, ErasedSource,
    SpanMap,
};
pub use scan::{scan, Annotation, AnnotationKind, EraseHint, FnParam, ScanError, ScanResult};
pub use typ::{Brand, Primitive, Type};
pub use type_parser::{parse_type, TypeParseError};
