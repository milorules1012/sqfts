//! Stable STS diagnostic codes and reporting structures.

use std::ops::Range;

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

/// Stable error/warning codes (SPEC §5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StsCode {
    /// Unknown type name.
    UnknownType = 1001,
    /// Duplicate declaration conflict.
    DuplicateDecl = 1002,
    /// Malformed declaration file.
    BadDecl = 1003,
    /// Parse / scan error.
    SyntaxError = 1004,
    /// Argument type mismatch.
    ArgMismatch = 2003,
    /// Assignment type mismatch.
    AssignMismatch = 2004,
    /// Return type mismatch.
    ReturnMismatch = 2005,
    /// Using a `nothing` value.
    UseNothing = 2006,
    /// Brand mismatch.
    BrandMismatch = 2107,
    /// Illegal cast (no overlap).
    IllegalCast = 2108,
    /// Implicit `any` under `noImplicitAny`.
    ImplicitAny = 2201,
    /// Un-narrowed `T | nothing` under `strictNil`.
    StrictNil = 2202,
    /// Unknown HashMap key under `strictHashMap`.
    UnknownHashKey = 2203,
    /// Declaration vs definition disagree.
    DeclDefMismatch = 2301,
    /// Unknown command (informational when args are typed).
    UnknownCommand = 2401,
}

impl StsCode {
    /// `STSxxxx` spelling.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnknownType => "STS1001",
            Self::DuplicateDecl => "STS1002",
            Self::BadDecl => "STS1003",
            Self::SyntaxError => "STS1004",
            Self::ArgMismatch => "STS2003",
            Self::AssignMismatch => "STS2004",
            Self::ReturnMismatch => "STS2005",
            Self::UseNothing => "STS2006",
            Self::BrandMismatch => "STS2107",
            Self::IllegalCast => "STS2108",
            Self::ImplicitAny => "STS2201",
            Self::StrictNil => "STS2202",
            Self::UnknownHashKey => "STS2203",
            Self::DeclDefMismatch => "STS2301",
            Self::UnknownCommand => "STS2401",
        }
    }
}

/// One diagnostic.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: StsCode,
    pub severity: Severity,
    pub message: String,
    pub span: Option<Range<usize>>,
    pub related: Vec<(String, Range<usize>)>,
}

impl Diagnostic {
    #[must_use]
    pub fn error(code: StsCode, message: impl Into<String>, span: Range<usize>) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            span: Some(span),
            related: vec![],
        }
    }

    #[must_use]
    pub fn warning(code: StsCode, message: impl Into<String>, span: Range<usize>) -> Self {
        Self {
            code,
            severity: Severity::Warning,
            message: message.into(),
            span: Some(span),
            related: vec![],
        }
    }
}
