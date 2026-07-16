//! `.d.sqfts` declaration loading (SPEC §4).

use std::path::Path;

use thiserror::Error;

use sqfts_syntax::{scan, AnnotationKind, ScanError, Type};

use crate::diagnostics::{Diagnostic, Severity, StsCode};
use crate::symbols::{FunctionSig, InterfaceMember, SymbolTable};

/// Error loading declarations.
#[derive(Debug, Error)]
pub enum DeclError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Scan(#[from] ScanError),
    #[error("{0}")]
    Conflict(String),
}

/// Loaded declaration set (project-wide).
#[derive(Debug, Clone, Default)]
pub struct DeclarationSet {
    pub symbols: SymbolTable,
    pub diagnostics: Vec<Diagnostic>,
}

/// Load all declaration files into a symbol table.
pub fn load_declarations(paths: &[impl AsRef<Path>]) -> Result<DeclarationSet, DeclError> {
    let mut set = DeclarationSet::default();
    for path in paths {
        let path = path.as_ref();
        let src = std::fs::read_to_string(path)?;
        let file = path.display().to_string();
        load_one(&src, &file, &mut set)?;
    }
    Ok(set)
}

/// Parse declarations from source text.
pub fn load_one(src: &str, file: &str, set: &mut DeclarationSet) -> Result<(), DeclError> {
    let scanned = scan(src)?;
    for ann in scanned.annotations {
        match ann.kind {
            AnnotationKind::TypeAlias { name, ty } => {
                if let Some(prev) = set.symbols.aliases.get(&name) {
                    if prev != &ty {
                        set.diagnostics.push(Diagnostic {
                            code: StsCode::DuplicateDecl,
                            severity: Severity::Error,
                            message: format!("conflicting type alias `{name}`"),
                            span: Some(ann.span.clone()),
                            related: vec![],
                        });
                    }
                } else {
                    set.symbols.aliases.insert(name, ty);
                }
            }
            AnnotationKind::Interface { name, members } => {
                let mapped: Vec<InterfaceMember> = members
                    .into_iter()
                    .map(|(n, opt, ty)| InterfaceMember {
                        name: n,
                        optional: opt,
                        ty,
                    })
                    .collect();
                if let Some(prev) = set.symbols.interfaces.get(&name) {
                    if prev != &mapped {
                        set.diagnostics.push(Diagnostic {
                            code: StsCode::DuplicateDecl,
                            severity: Severity::Error,
                            message: format!("conflicting interface `{name}`"),
                            span: Some(ann.span.clone()),
                            related: vec![],
                        });
                    }
                } else {
                    set.symbols.interfaces.insert(name, mapped);
                }
            }
            AnnotationKind::DeclareVar { name, ty } => {
                if let Some((prev, prev_file)) = set.symbols.globals.get(&name) {
                    if prev != &ty {
                        set.diagnostics.push(Diagnostic {
                            code: StsCode::DuplicateDecl,
                            severity: Severity::Error,
                            message: format!(
                                "conflicting declaration of `{name}` (also in {prev_file})"
                            ),
                            span: Some(ann.span.clone()),
                            related: vec![],
                        });
                    }
                } else {
                    set.symbols
                        .globals
                        .insert(name, (ty, file.to_string()));
                }
            }
            AnnotationKind::DeclareFn { name, params, ret } => {
                let sig = FunctionSig {
                    name: name.clone(),
                    params,
                    ret,
                    file: file.to_string(),
                };
                if let Some(prev) = set.symbols.functions.get(&name) {
                    if prev.params != sig.params || prev.ret != sig.ret {
                        set.diagnostics.push(Diagnostic {
                            code: StsCode::DuplicateDecl,
                            severity: Severity::Error,
                            message: format!(
                                "conflicting function declaration `{name}` (also in {})",
                                prev.file
                            ),
                            span: Some(ann.span.clone()),
                            related: vec![],
                        });
                    }
                } else {
                    set.symbols.functions.insert(name, sig);
                }
            }
            // Inline annotations in .d.sqfts are unexpected but ignore for loading
            _ => {}
        }
    }
    let _ = Type::any(); // keep import used in docs
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_decls() {
        let src = r#"
type moneyTarget = object | group;
declare project_playerCash: number;
declare function TAG_fnc_checkPayment(_unit: object, _amount: number): boolean;
"#;
        let mut set = DeclarationSet::default();
        load_one(src, "test.d.sqfts", &mut set).unwrap();
        assert!(set.symbols.aliases.contains_key("moneyTarget"));
        assert!(set.symbols.globals.contains_key("project_playerCash"));
        assert!(set.symbols.functions.contains_key("TAG_fnc_checkPayment"));
    }

    #[test]
    fn duplicate_conflict() {
        let a = "declare project_x: number;\n";
        let b = "declare project_x: string;\n";
        let mut set = DeclarationSet::default();
        load_one(a, "a.d.sqfts", &mut set).unwrap();
        load_one(b, "b.d.sqfts", &mut set).unwrap();
        assert!(set
            .diagnostics
            .iter()
            .any(|d| d.code == StsCode::DuplicateDecl));
    }
}
