//! Byte-stable erasure of SQFts annotations (SPEC §7).

use std::ops::Range;

use crate::scan::{scan, Annotation, EraseHint, ScanError, ScanResult};
use crate::typ::Type;

/// Options controlling erasure.
#[derive(Debug, Clone, Default)]
pub struct EraseOptions {
    /// When true, typed `params` also emit runtime guard arrays (SPEC §7.4).
    pub emit_runtime_params: bool,
}

/// Maps erased-file offsets back to original offsets.
#[derive(Debug, Clone, Default)]
pub struct SpanMap {
    /// Sorted list of (erased_offset, original_offset) anchors.
    /// Between anchors, offsets advance 1:1 until the next edit.
    segments: Vec<SpanSegment>,
}

#[derive(Debug, Clone)]
struct SpanSegment {
    erased_start: usize,
    original_start: usize,
    /// Length in the erased output for this 1:1 (or replacement) region.
    erased_len: usize,
    /// Length in the original for this region.
    original_len: usize,
}

impl SpanMap {
    /// Map an offset in the erased source to the original source.
    #[must_use]
    pub fn to_original(&self, erased_offset: usize) -> usize {
        if self.segments.is_empty() {
            return erased_offset;
        }
        let mut best = self.segments[0].original_start;
        for seg in &self.segments {
            if erased_offset < seg.erased_start {
                break;
            }
            let delta = erased_offset - seg.erased_start;
            if delta <= seg.erased_len {
                // Prefer start of original for deleted regions (erased_len may be 0)
                if seg.erased_len == 0 {
                    best = seg.original_start;
                } else {
                    let ratio = if seg.erased_len == 0 {
                        0
                    } else {
                        delta.min(seg.original_len)
                    };
                    best = seg.original_start + ratio.min(seg.original_len);
                }
            } else {
                best = seg.original_start + seg.original_len;
            }
        }
        best
    }
}

/// Result of erasing a file.
#[derive(Debug, Clone)]
pub struct ErasedSource {
    /// Plain SQF text.
    pub text: String,
    /// Annotations discovered (from the original).
    pub annotations: Vec<Annotation>,
    /// Offset map erased → original.
    pub span_map: SpanMap,
    /// Original source.
    pub original: String,
}

/// Erase SQFts annotations from `source`.
pub fn erase(source: &str, options: &EraseOptions) -> Result<ErasedSource, ScanError> {
    let scanned = scan(source)?;
    erase_scanned(&scanned, options)
}

/// Erase using an already-scanned result.
pub fn erase_scanned(
    scanned: &ScanResult,
    options: &EraseOptions,
) -> Result<ErasedSource, ScanError> {
    let original = scanned.source.as_str();
    // Build edit list: (span, replacement)
    let mut edits: Vec<(Range<usize>, String)> = Vec::new();

    for ann in &scanned.annotations {
        let (span, replacement) = match &ann.erase {
            EraseHint::Delete { trailing_newline } => {
                let mut end = ann.span.end;
                if *trailing_newline {
                    end = consume_one_newline(original, end);
                }
                // Also absorb one leading newline? SPEC says trailing.
                // For blank-line cleanliness, if the whole line becomes empty, we still
                // only delete the statement + one newline.
                (ann.span.start..end, String::new())
            }
            EraseHint::Replace(text) => {
                let mut text = text.clone();
                if options.emit_runtime_params {
                    if let Some(lowered) = runtime_params_lowering(ann) {
                        text = lowered;
                    }
                }
                (ann.span.clone(), text)
            }
            EraseHint::DeleteTypeAnnot { .. } => (ann.span.clone(), String::new()),
            EraseHint::DeleteCast => (ann.span.clone(), String::new()),
        };
        edits.push((span, replacement));
    }

    // Sort by start descending to apply safely; reject overlaps.
    edits.sort_by(|a, b| b.0.start.cmp(&a.0.start));

    let mut segments = Vec::new();
    // Rebuild by walking original left-to-right for span map accuracy.
    edits.sort_by(|a, b| a.0.start.cmp(&b.0.start));
    let mut out = String::new();
    let mut cursor = 0usize;
    for (span, replacement) in &edits {
        if span.start < cursor {
            // Overlap — skip (shouldn't happen)
            continue;
        }
        // 1:1 copy
        let piece = &original[cursor..span.start];
        segments.push(SpanSegment {
            erased_start: out.len(),
            original_start: cursor,
            erased_len: piece.len(),
            original_len: piece.len(),
        });
        out.push_str(piece);
        // Replacement / deletion
        segments.push(SpanSegment {
            erased_start: out.len(),
            original_start: span.start,
            erased_len: replacement.len(),
            original_len: span.end - span.start,
        });
        out.push_str(replacement);
        cursor = span.end;
    }
    if cursor < original.len() {
        let piece = &original[cursor..];
        segments.push(SpanSegment {
            erased_start: out.len(),
            original_start: cursor,
            erased_len: piece.len(),
            original_len: piece.len(),
        });
        out.push_str(piece);
    }

    Ok(ErasedSource {
        text: out,
        annotations: scanned.annotations.clone(),
        span_map: SpanMap { segments },
        original: original.to_string(),
    })
}

fn consume_one_newline(src: &str, mut end: usize) -> usize {
    if end < src.len() && src.as_bytes()[end] == b'\r' {
        end += 1;
    }
    if end < src.len() && src.as_bytes()[end] == b'\n' {
        end += 1;
    }
    end
}

fn runtime_params_lowering(ann: &Annotation) -> Option<String> {
    use crate::scan::AnnotationKind;
    match &ann.kind {
        AnnotationKind::TypedParam {
            name,
            ty,
            optional_nil,
            has_default,
            default_span: _,
        } => {
            let exemplars = type_exemplars(ty);
            if exemplars.is_empty() {
                return None;
            }
            let guard = format!("[{}]", exemplars.join(", "));
            if *has_default {
                // Need the default from original — caller already built Replace with default;
                // rebuild with guard. We don't have original here easily; skip and let
                // the non-runtime form stand unless we pass more context.
                // For emit_runtime_params with default, form is ["name", default, [exemplars]]
                // The Replace text is already `["name", expr]` — extend it.
                None // handled below via re-scan path in erase when options set
            } else if *optional_nil {
                Some(format!("[\"{name}\", nil, {guard}]"))
            } else {
                // Required: use first exemplar as default for the runtime form
                let def = exemplars.first()?.as_str();
                Some(format!("[\"{name}\", {def}, {guard}]"))
            }
        }
        _ => None,
    }
}

/// Map a type to SQF runtime guard exemplars (SPEC §7.4).
#[must_use]
pub fn type_exemplars(ty: &Type) -> Vec<String> {
    match ty {
        Type::Primitive(p) => {
            use crate::typ::Primitive::*;
            let s = match p {
                Number => "0",
                String => "\"\"",
                Boolean => "false",
                Object => "objNull",
                Group => "grpNull",
                Array => "[]",
                Code => "{}",
                Config => "configNull",
                Side => "sideUnknown",
                Control => "controlNull",
                Display => "displayNull",
                Task => "taskNull",
                Location => "locationNull",
                Namespace => "missionNamespace",
                _ => return vec![],
            };
            vec![s.to_string()]
        }
        Type::Brand(_) | Type::Tuple(_) | Type::ArrayOf(_) => vec!["[]".into()],
        Type::Union(parts) => {
            let mut out = Vec::new();
            for p in parts {
                out.extend(type_exemplars(p));
            }
            out.dedup();
            out
        }
        Type::Named(_) => vec![],
        Type::StringLit(s) => vec![format_string_lit_exemplar(s)],
        Type::NumberLit(n) => vec![format!("{}", n.0)],
    }
}

fn format_string_lit_exemplar(s: &str) -> String {
    let mut out = String::from("\"");
    for ch in s.chars() {
        if ch == '"' {
            out.push('"');
            out.push('"');
        } else {
            out.push(ch);
        }
    }
    out.push('"');
    out
}

/// Erase with full runtime-params lowering (needs original source for defaults).
pub fn erase_with_runtime_params(source: &str) -> Result<ErasedSource, ScanError> {
    let scanned = scan(source)?;
    let mut options = EraseOptions {
        emit_runtime_params: false,
    };
    // First get normal erasure edits, then patch typed params.
    // Simpler: rebuild edits manually with runtime form.
    let original = scanned.source.as_str();
    let mut edits: Vec<(Range<usize>, String)> = Vec::new();
    for ann in &scanned.annotations {
        use crate::scan::AnnotationKind;
        match (&ann.kind, &ann.erase) {
            (
                AnnotationKind::TypedParam {
                    name,
                    ty,
                    optional_nil: _,
                    has_default,
                    default_span,
                },
                _,
            ) => {
                let exemplars = type_exemplars(ty);
                let text = if exemplars.is_empty() {
                    // fall back to normal erase
                    match &ann.erase {
                        EraseHint::Replace(t) => t.clone(),
                        _ => continue,
                    }
                } else {
                    let guard = format!("[{}]", exemplars.join(", "));
                    if *has_default {
                        let def = default_span
                            .as_ref()
                            .map(|s| original[s.start..s.end].trim())
                            .unwrap_or("nil");
                        format!("[\"{name}\", {def}, {guard}]")
                    } else {
                        let def = exemplars[0].as_str();
                        format!("[\"{name}\", {def}, {guard}]")
                    }
                };
                edits.push((ann.span.clone(), text));
            }
            (_, EraseHint::Delete { trailing_newline }) => {
                let mut end = ann.span.end;
                if *trailing_newline {
                    end = consume_one_newline(original, end);
                }
                edits.push((ann.span.start..end, String::new()));
            }
            (_, EraseHint::Replace(t)) => edits.push((ann.span.clone(), t.clone())),
            (_, EraseHint::DeleteTypeAnnot { .. } | EraseHint::DeleteCast) => {
                edits.push((ann.span.clone(), String::new()));
            }
        }
    }
    options.emit_runtime_params = true;
    // Apply edits
    edits.sort_by(|a, b| a.0.start.cmp(&b.0.start));
    let mut out = String::new();
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    for (span, replacement) in &edits {
        if span.start < cursor {
            continue;
        }
        let piece = &original[cursor..span.start];
        segments.push(SpanSegment {
            erased_start: out.len(),
            original_start: cursor,
            erased_len: piece.len(),
            original_len: piece.len(),
        });
        out.push_str(piece);
        segments.push(SpanSegment {
            erased_start: out.len(),
            original_start: span.start,
            erased_len: replacement.len(),
            original_len: span.end - span.start,
        });
        out.push_str(replacement);
        cursor = span.end;
    }
    if cursor < original.len() {
        let piece = &original[cursor..];
        segments.push(SpanSegment {
            erased_start: out.len(),
            original_start: cursor,
            erased_len: piece.len(),
            original_len: piece.len(),
        });
        out.push_str(piece);
    }
    let _ = options;
    Ok(ErasedSource {
        text: out,
        annotations: scanned.annotations.clone(),
        span_map: SpanMap { segments },
        original: original.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_plain_sqf() {
        let src = "private _x = 1;\n_x = _x + 1;\n";
        let erased = erase(src, &EraseOptions::default()).unwrap();
        assert_eq!(erased.text, src);
    }

    #[test]
    fn typed_private_and_params() {
        let src = r#"params [
    "_vehicle": object,
    "_fee": number = 0,
    "_note"?: string
];
private _owner: object = objNull;
private _result: string;
"#;
        let erased = erase(src, &EraseOptions::default()).unwrap();
        assert!(erased.text.contains("\"_vehicle\""));
        assert!(erased.text.contains("[\"_fee\", 0]"));
        assert!(erased.text.contains("\"_note\""));
        assert!(erased.text.contains("private _owner = objNull;"));
        assert!(erased.text.contains("private \"_result\";"));
        assert!(!erased.text.contains(": object"));
        assert!(!erased.text.contains(": number"));
        assert!(!erased.text.contains(": string"));
    }

    #[test]
    fn type_declare_erased() {
        let src = "type feeTier = [number, number];\nprivate _x = 1;\n";
        let erased = erase(src, &EraseOptions::default()).unwrap();
        assert_eq!(erased.text, "private _x = 1;\n");
    }

    #[test]
    fn cast_erased() {
        let src = "private _crew = (_this select 0) as object[];\n";
        let erased = erase(src, &EraseOptions::default()).unwrap();
        assert_eq!(erased.text, "private _crew = (_this select 0);\n");
    }

    #[test]
    fn worked_example_7_3() {
        let src = r#"// File: fn_impoundVehicle.sqfts
// Author: Example Mission
type feeTier = [number, number];

params [
    "_vehicle": object,
    "_fee": number = 0
];

private _owner: object = _vehicle getVariable ["project_owner", objNull];
"#;
        let erased = erase(src, &EraseOptions::default()).unwrap();
        assert!(!erased.text.contains("type feeTier"));
        assert!(erased.text.contains("\"_vehicle\""));
        assert!(erased.text.contains("[\"_fee\", 0]"));
        assert!(erased
            .text
            .contains("private _owner = _vehicle getVariable"));
    }
}
