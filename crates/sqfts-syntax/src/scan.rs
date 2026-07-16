//! Annotation scanner for `.sqfts` / `.d.sqfts` sources.

use std::ops::Range;

use thiserror::Error;

use crate::typ::Type;
use crate::type_parser::{parse_type, TypeParseError};

/// Kind of SQFts annotation found in source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotationKind {
    /// `private _name: T` (with or without initializer).
    TypedPrivate {
        /// Local name including leading `_`.
        name: String,
        /// Declared type.
        ty: Type,
        /// True when there is no `= expr` (erases to `private "_name";`).
        declare_only: bool,
    },
    /// Typed entry inside a `params` array.
    TypedParam {
        /// Param name string contents (without quotes), e.g. `_vehicle`.
        name: String,
        /// Declared type.
        ty: Type,
        /// `?:` form (optional without default).
        optional_nil: bool,
        /// True when `= expr` follows the type.
        has_default: bool,
        /// Byte span of the default expression if present (exclusive of `=`).
        default_span: Option<Range<usize>>,
    },
    /// `type Name = Type;`
    TypeAlias {
        /// Alias name.
        name: String,
        /// Aliased type.
        ty: Type,
    },
    /// `interface Name { ... }`
    Interface {
        /// Interface name.
        name: String,
        /// Members: (key, optional, type).
        members: Vec<(String, bool, Type)>,
    },
    /// `declare name: Type;`
    DeclareVar {
        /// Global name.
        name: String,
        /// Declared type.
        ty: Type,
    },
    /// `declare function Name(...): Ret;`
    DeclareFn {
        /// Function symbol.
        name: String,
        /// Parameters.
        params: Vec<FnParam>,
        /// Return type.
        ret: Type,
    },
    /// `expr as Type`
    Cast {
        /// Target type.
        ty: Type,
    },
}

/// Parameter in a `declare function` signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnParam {
    /// Parameter name (may include leading `_`).
    pub name: String,
    /// Type.
    pub ty: Type,
    /// Optional (`name?:` or has default — for declare, `?` means optional).
    pub optional: bool,
}

/// A single annotation with its source span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    /// Byte span in the original source covering the erasable region
    /// (or the whole statement for declare/type/interface).
    pub span: Range<usize>,
    /// What was found.
    pub kind: AnnotationKind,
    /// Extra span info for erasure helpers.
    pub erase: EraseHint,
}

/// How to erase this annotation (SPEC §7.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EraseHint {
    /// Delete `span` (and optionally one trailing newline).
    Delete {
        /// Also delete one trailing `\r?\n` after span.
        trailing_newline: bool,
    },
    /// Replace `span` with this text.
    Replace(String),
    /// Delete only the `: Type` portion (span already narrowed).
    DeleteTypeAnnot {
        /// Optional preceding space span to delete.
        space_before: Option<Range<usize>>,
    },
    /// Cast: delete ` as Type` (span is that region).
    DeleteCast,
}

/// Result of scanning a source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanResult {
    /// Original source (borrowed into owned for convenience).
    pub source: String,
    /// Annotations in source order.
    pub annotations: Vec<Annotation>,
}

/// Scanner error.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScanError {
    /// Failed to parse a type expression.
    #[error("type error at {at}: {message}")]
    Type {
        /// Absolute byte offset.
        at: usize,
        /// Message.
        message: String,
    },
    /// Malformed SQFts construct.
    #[error("syntax error at {at}: {message}")]
    Syntax {
        /// Absolute byte offset.
        at: usize,
        /// Message.
        message: String,
    },
}

impl From<(usize, TypeParseError)> for ScanError {
    fn from((base, e): (usize, TypeParseError)) -> Self {
        match e {
            TypeParseError::Message { at, message } => Self::Type {
                at: base + at,
                message,
            },
        }
    }
}

/// Scan `source` for SQFts annotations.
pub fn scan(source: &str) -> Result<ScanResult, ScanError> {
    let mut annotations = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        // Skip comments and strings so we don't false-match inside them.
        if starts_with(source, i, "//") {
            i = skip_line(source, i);
            continue;
        }
        if starts_with(source, i, "/*") {
            i = skip_block_comment(source, i);
            continue;
        }
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            i = skip_string(source, i);
            continue;
        }

        // Statement-initial contextual keywords.
        if is_stmt_start(source, i) {
            if ident_at(source, i, "type") {
                if let Some(ann) = try_type_alias(source, i)? {
                    i = ann.span.end;
                    annotations.push(ann);
                    continue;
                }
            } else if ident_at(source, i, "declare") {
                if let Some(ann) = try_declare(source, i)? {
                    i = ann.span.end;
                    annotations.push(ann);
                    continue;
                }
            } else if ident_at(source, i, "interface") {
                if let Some(ann) = try_interface(source, i)? {
                    i = ann.span.end;
                    annotations.push(ann);
                    continue;
                }
            } else if ident_at(source, i, "private") {
                if let Some(ann) = try_typed_private(source, i)? {
                    i = ann.span.end;
                    annotations.push(ann);
                    continue;
                }
            } else if ident_at(source, i, "params") {
                let (end, mut params_anns) = scan_params(source, i)?;
                annotations.append(&mut params_anns);
                i = end;
                continue;
            }
        }

        // Postfix `as Type`
        if ident_at(source, i, "as") && is_cast_context(source, i) {
            if let Some(ann) = try_cast(source, i)? {
                i = ann.span.end;
                annotations.push(ann);
                continue;
            }
        }

        i += 1;
    }

    Ok(ScanResult {
        source: source.to_string(),
        annotations,
    })
}

fn starts_with(src: &str, i: usize, prefix: &str) -> bool {
    src[i..].starts_with(prefix)
}

fn skip_line(src: &str, mut i: usize) -> usize {
    while i < src.len() && src.as_bytes()[i] != b'\n' {
        i += 1;
    }
    if i < src.len() {
        i += 1;
    }
    i
}

fn skip_block_comment(src: &str, mut i: usize) -> usize {
    i += 2;
    while i + 1 < src.len() {
        if src.as_bytes()[i] == b'*' && src.as_bytes()[i + 1] == b'/' {
            return i + 2;
        }
        i += 1;
    }
    src.len()
}

fn skip_string(src: &str, i: usize) -> usize {
    let bytes = src.as_bytes();
    let quote = bytes[i];
    let mut j = i + 1;
    while j < bytes.len() {
        if bytes[j] == quote {
            // SQF doubles quotes to escape
            if j + 1 < bytes.len() && bytes[j + 1] == quote {
                j += 2;
                continue;
            }
            return j + 1;
        }
        j += 1;
    }
    src.len()
}

fn is_stmt_start(src: &str, i: usize) -> bool {
    if i == 0 {
        return true;
    }
    let prev = &src[..i];
    // After `;`, `{`, or newline / only whitespace since those.
    let trimmed = prev.trim_end_matches(|c: char| c == ' ' || c == '\t');
    trimmed.is_empty()
        || trimmed.ends_with(';')
        || trimmed.ends_with('{')
        || trimmed.ends_with('\n')
}

fn ident_at(src: &str, i: usize, word: &str) -> bool {
    if !src[i..].starts_with(word) {
        return false;
    }
    let end = i + word.len();
    let before_ok =
        i == 0 || !src.as_bytes()[i - 1].is_ascii_alphanumeric() && src.as_bytes()[i - 1] != b'_';
    let after_ok = end >= src.len()
        || !src.as_bytes()[end].is_ascii_alphanumeric() && src.as_bytes()[end] != b'_';
    before_ok && after_ok
}

fn skip_ws(src: &str, mut i: usize) -> usize {
    while i < src.len() && src.as_bytes()[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

fn parse_ident(src: &str, i: usize) -> Option<(String, usize)> {
    let bytes = src.as_bytes();
    if i >= bytes.len() {
        return None;
    }
    if !(bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') {
        return None;
    }
    let mut j = i + 1;
    while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
        j += 1;
    }
    Some((src[i..j].to_string(), j))
}

fn try_type_alias(src: &str, start: usize) -> Result<Option<Annotation>, ScanError> {
    // type Ident = Type ;
    let mut i = start + "type".len();
    i = skip_ws(src, i);
    let Some((name, j)) = parse_ident(src, i) else {
        return Ok(None);
    };
    i = skip_ws(src, j);
    if !src[i..].starts_with('=') {
        return Ok(None); // plain SQF `type = …`
    }
    i = skip_ws(src, i + 1);
    let (ty, consumed) = parse_type(&src[i..]).map_err(|e| (i, e))?;
    i += consumed;
    i = skip_ws(src, i);
    if !src[i..].starts_with(';') {
        return Err(ScanError::Syntax {
            at: i,
            message: "expected ';' after type alias".into(),
        });
    }
    let end = i + 1;
    Ok(Some(Annotation {
        span: start..end,
        kind: AnnotationKind::TypeAlias { name, ty },
        erase: EraseHint::Delete {
            trailing_newline: true,
        },
    }))
}

fn try_declare(src: &str, start: usize) -> Result<Option<Annotation>, ScanError> {
    let mut i = start + "declare".len();
    i = skip_ws(src, i);
    if ident_at(src, i, "function") {
        return try_declare_fn(src, start, i);
    }
    let Some((name, j)) = parse_ident(src, i) else {
        return Ok(None);
    };
    i = skip_ws(src, j);
    if !src[i..].starts_with(':') {
        return Ok(None);
    }
    i = skip_ws(src, i + 1);
    let (ty, consumed) = parse_type(&src[i..]).map_err(|e| (i, e))?;
    i += consumed;
    i = skip_ws(src, i);
    if !src[i..].starts_with(';') {
        return Err(ScanError::Syntax {
            at: i,
            message: "expected ';' after declare".into(),
        });
    }
    let end = i + 1;
    Ok(Some(Annotation {
        span: start..end,
        kind: AnnotationKind::DeclareVar { name, ty },
        erase: EraseHint::Delete {
            trailing_newline: true,
        },
    }))
}

fn try_declare_fn(src: &str, start: usize, fn_kw: usize) -> Result<Option<Annotation>, ScanError> {
    let mut i = fn_kw + "function".len();
    i = skip_ws(src, i);
    let Some((name, j)) = parse_ident(src, i) else {
        return Err(ScanError::Syntax {
            at: i,
            message: "expected function name".into(),
        });
    };
    i = skip_ws(src, j);
    if !src[i..].starts_with('(') {
        return Err(ScanError::Syntax {
            at: i,
            message: "expected '('".into(),
        });
    }
    i += 1;
    let mut params = Vec::new();
    i = skip_ws(src, i);
    if !src[i..].starts_with(')') {
        loop {
            i = skip_ws(src, i);
            let Some((pname, j)) = parse_ident(src, i) else {
                return Err(ScanError::Syntax {
                    at: i,
                    message: "expected param name".into(),
                });
            };
            i = j;
            let optional = if src[i..].starts_with('?') {
                i += 1;
                true
            } else {
                false
            };
            i = skip_ws(src, i);
            if !src[i..].starts_with(':') {
                return Err(ScanError::Syntax {
                    at: i,
                    message: "expected ':' in declare function param".into(),
                });
            }
            i = skip_ws(src, i + 1);
            let (ty, consumed) = parse_type(&src[i..]).map_err(|e| (i, e))?;
            i += consumed;
            params.push(FnParam {
                name: pname,
                ty,
                optional,
            });
            i = skip_ws(src, i);
            if src[i..].starts_with(',') {
                i += 1;
                continue;
            }
            break;
        }
    }
    i = skip_ws(src, i);
    if !src[i..].starts_with(')') {
        return Err(ScanError::Syntax {
            at: i,
            message: "expected ')'".into(),
        });
    }
    i = skip_ws(src, i + 1);
    if !src[i..].starts_with(':') {
        return Err(ScanError::Syntax {
            at: i,
            message: "expected return type".into(),
        });
    }
    i = skip_ws(src, i + 1);
    let (ret, consumed) = parse_type(&src[i..]).map_err(|e| (i, e))?;
    i += consumed;
    i = skip_ws(src, i);
    if !src[i..].starts_with(';') {
        return Err(ScanError::Syntax {
            at: i,
            message: "expected ';'".into(),
        });
    }
    let end = i + 1;
    Ok(Some(Annotation {
        span: start..end,
        kind: AnnotationKind::DeclareFn { name, params, ret },
        erase: EraseHint::Delete {
            trailing_newline: true,
        },
    }))
}

fn try_interface(src: &str, start: usize) -> Result<Option<Annotation>, ScanError> {
    let mut i = start + "interface".len();
    i = skip_ws(src, i);
    let Some((name, j)) = parse_ident(src, i) else {
        return Ok(None);
    };
    i = skip_ws(src, j);
    if !src[i..].starts_with('{') {
        return Ok(None);
    }
    i += 1;
    let mut members = Vec::new();
    loop {
        i = skip_ws(src, i);
        if src[i..].starts_with('}') {
            i += 1;
            break;
        }
        let Some((mname, j)) = parse_ident(src, i) else {
            return Err(ScanError::Syntax {
                at: i,
                message: "expected interface member".into(),
            });
        };
        i = j;
        let optional = if src[i..].starts_with('?') {
            i += 1;
            true
        } else {
            false
        };
        i = skip_ws(src, i);
        if !src[i..].starts_with(':') {
            return Err(ScanError::Syntax {
                at: i,
                message: "expected ':' in interface member".into(),
            });
        }
        i = skip_ws(src, i + 1);
        let (ty, consumed) = parse_type(&src[i..]).map_err(|e| (i, e))?;
        i += consumed;
        i = skip_ws(src, i);
        if src[i..].starts_with(';') {
            i += 1;
        }
        members.push((mname, optional, ty));
    }
    // Optional trailing semicolon is not in grammar; span ends at `}`
    Ok(Some(Annotation {
        span: start..i,
        kind: AnnotationKind::Interface { name, members },
        erase: EraseHint::Delete {
            trailing_newline: true,
        },
    }))
}

fn try_typed_private(src: &str, start: usize) -> Result<Option<Annotation>, ScanError> {
    // private _name: Type [= expr] ;
    let mut i = start + "private".len();
    i = skip_ws(src, i);
    let Some((name, j)) = parse_ident(src, i) else {
        return Ok(None);
    };
    if !name.starts_with('_') {
        return Ok(None);
    }
    i = skip_ws(src, j);
    if !src[i..].starts_with(':') {
        return Ok(None); // plain private
    }
    let colon = i;
    // space before colon?
    let space_before = if colon > 0 && src.as_bytes()[colon - 1] == b' ' {
        Some(colon - 1..colon)
    } else {
        None
    };
    i = skip_ws(src, colon + 1);
    let (ty, consumed) = parse_type(&src[i..]).map_err(|e| (i, e))?;
    i += consumed;
    let type_end = i;
    i = skip_ws(src, i);
    let declare_only = !src[i..].starts_with('=');
    if declare_only {
        if !src[i..].starts_with(';') {
            return Err(ScanError::Syntax {
                at: i,
                message: "expected ';' after typed private".into(),
            });
        }
        let end = i + 1;
        // Rewrite whole `private _x: T;` → `private "_x";`
        return Ok(Some(Annotation {
            span: start..end,
            kind: AnnotationKind::TypedPrivate {
                name: name.clone(),
                ty,
                declare_only: true,
            },
            erase: EraseHint::Replace(format!("private \"{name}\";")),
        }));
    }
    // Has initializer — only delete `: Type` (and optional space before)
    Ok(Some(Annotation {
        span: space_before.as_ref().map(|s| s.start).unwrap_or(colon)..type_end,
        kind: AnnotationKind::TypedPrivate {
            name,
            ty,
            declare_only: false,
        },
        erase: EraseHint::DeleteTypeAnnot { space_before },
    }))
}

fn scan_params(src: &str, start: usize) -> Result<(usize, Vec<Annotation>), ScanError> {
    // Find the `[` after params
    let mut i = start + "params".len();
    i = skip_ws(src, i);
    if !src[i..].starts_with('[') {
        return Ok((i, vec![]));
    }
    let array_start = i;
    // Walk the array with bracket depth, finding typed string entries.
    i += 1;
    let mut depth = 1i32;
    let mut anns = Vec::new();
    while i < src.len() && depth > 0 {
        if starts_with(src, i, "//") {
            i = skip_line(src, i);
            continue;
        }
        if starts_with(src, i, "/*") {
            i = skip_block_comment(src, i);
            continue;
        }
        let b = src.as_bytes()[i];
        if b == b'"' || b == b'\'' {
            // Maybe typed param: "name"?: Type or "name": Type [= expr]
            if depth == 1 {
                if let Some(ann) = try_typed_param_entry(src, i)? {
                    i = ann.span.end;
                    anns.push(ann);
                    continue;
                }
            }
            i = skip_string(src, i);
            continue;
        }
        match b {
            b'[' => depth += 1,
            b']' => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    let _ = array_start;
    Ok((i, anns))
}

fn try_typed_param_entry(src: &str, start: usize) -> Result<Option<Annotation>, ScanError> {
    // "name"?: Type  OR  "name": Type [= expr]
    let str_end = skip_string(src, start);
    if str_end <= start + 1 {
        return Ok(None);
    }
    let quote = src.as_bytes()[start] as char;
    let name = src[start + 1..str_end - 1].replace(&[quote, quote][..], &quote.to_string());
    let mut i = skip_ws(src, str_end);
    let optional_nil = if src[i..].starts_with('?') {
        i += 1;
        i = skip_ws(src, i);
        true
    } else {
        false
    };
    if !src[i..].starts_with(':') {
        return Ok(None);
    }
    let colon = i;
    i = skip_ws(src, colon + 1);
    let (ty, consumed) = parse_type(&src[i..]).map_err(|e| (i, e))?;
    i += consumed;
    let after_type = i;
    i = skip_ws(src, i);
    let (has_default, default_span, entry_end) = if src[i..].starts_with('=') {
        i = skip_ws(src, i + 1);
        let expr_start = i;
        // Scan expression until `,` or `]` at depth 0 (relative to entry)
        let expr_end = scan_expr_until_comma_or_bracket(src, i);
        (true, Some(expr_start..expr_end), expr_end)
    } else {
        (false, None, after_type)
    };

    let erase = if has_default {
        let def = &src[default_span.clone().unwrap()];
        EraseHint::Replace(format!("[\"{name}\", {def}]"))
    } else if optional_nil {
        // delete `?: Type` — replace from after string through type with nothing,
        // leaving the string. Span = from after string to after type.
        EraseHint::Replace(format!("\"{name}\""))
    } else {
        EraseHint::Replace(format!("\"{name}\""))
    };

    // For all typed param forms, replace the whole entry (string through type/default)
    // with the erased form.
    Ok(Some(Annotation {
        span: start..entry_end,
        kind: AnnotationKind::TypedParam {
            name,
            ty,
            optional_nil,
            has_default,
            default_span,
        },
        erase,
    }))
}

fn scan_expr_until_comma_or_bracket(src: &str, mut i: usize) -> usize {
    let start = i;
    let mut depth_sq = 0i32;
    let mut depth_rnd = 0i32;
    let mut depth_curly = 0i32;
    let mut last_non_ws = i;
    while i < src.len() {
        if starts_with(src, i, "//") {
            i = skip_line(src, i);
            continue;
        }
        if starts_with(src, i, "/*") {
            i = skip_block_comment(src, i);
            continue;
        }
        let b = src.as_bytes()[i];
        if b == b'"' || b == b'\'' {
            i = skip_string(src, i);
            last_non_ws = i;
            continue;
        }
        match b {
            b'[' => {
                depth_sq += 1;
                i += 1;
                last_non_ws = i;
            }
            b']' => {
                if depth_sq == 0 && depth_rnd == 0 && depth_curly == 0 {
                    return last_non_ws.max(start);
                }
                depth_sq -= 1;
                i += 1;
                last_non_ws = i;
            }
            b'(' => {
                depth_rnd += 1;
                i += 1;
                last_non_ws = i;
            }
            b')' => {
                depth_rnd -= 1;
                i += 1;
                last_non_ws = i;
            }
            b'{' => {
                depth_curly += 1;
                i += 1;
                last_non_ws = i;
            }
            b'}' => {
                depth_curly -= 1;
                i += 1;
                last_non_ws = i;
            }
            b',' if depth_sq == 0 && depth_rnd == 0 && depth_curly == 0 => {
                return last_non_ws.max(start);
            }
            _ => {
                if !b.is_ascii_whitespace() {
                    last_non_ws = i + 1;
                }
                i += 1;
            }
        }
    }
    last_non_ws.max(start)
}

fn is_cast_context(src: &str, i: usize) -> bool {
    // Previous non-ws must look like end of an expression.
    if i == 0 {
        return false;
    }
    let mut j = i;
    while j > 0 && src.as_bytes()[j - 1].is_ascii_whitespace() {
        j -= 1;
    }
    if j == 0 {
        return false;
    }
    let c = src.as_bytes()[j - 1];
    // identifier/number/string/)/]/closing
    c.is_ascii_alphanumeric()
        || c == b'_'
        || c == b')'
        || c == b']'
        || c == b'}'
        || c == b'"'
        || c == b'\''
}

fn try_cast(src: &str, start: usize) -> Result<Option<Annotation>, ScanError> {
    // ` as Type` — start points at `as`
    // Include one preceding space in the delete span when present.
    let span_start = if start > 0 && src.as_bytes()[start - 1] == b' ' {
        start - 1
    } else {
        start
    };
    let mut i = start + "as".len();
    i = skip_ws(src, i);
    // Must be followed by a type expression
    let Ok((ty, consumed)) = parse_type(&src[i..]) else {
        return Ok(None);
    };
    // Heuristic: if nothing was consumed meaningfully, skip
    if consumed == 0 {
        return Ok(None);
    }
    i += consumed;
    Ok(Some(Annotation {
        span: span_start..i,
        kind: AnnotationKind::Cast { ty },
        erase: EraseHint::DeleteCast,
    }))
}
