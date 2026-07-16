//! Core type checker.

use std::collections::HashMap;
use std::io::Write;
use std::ops::Range;

use anyhow::Result;
use hemtt_common::config::{PDriveOption, PreprocessorOptions};
use hemtt_preprocessor::Processor;
use hemtt_sqf::parser::database::Database as HemttDb;
use hemtt_sqf::{BinaryCommand, Expression, Statement, Statements};
use hemtt_workspace::Workspace;
use sqfts_db::{CallKind, CommandDb};
use sqfts_syntax::{erase, Annotation, AnnotationKind, EraseOptions, Primitive, SpanMap, Type};

use crate::assignability::is_assignable;
use crate::config::CheckFlags;
use crate::decls::DeclarationSet;
use crate::diagnostics::{Diagnostic, Severity, StsCode};
use crate::symbols::SymbolTable;

/// Result of checking one file.
#[derive(Debug, Clone, Default)]
pub struct CheckResult {
    pub diagnostics: Vec<Diagnostic>,
}

/// Type-check a single source file.
pub fn check_source(
    source: &str,
    path: &str,
    db: &CommandDb,
    decls: &DeclarationSet,
    flags: &CheckFlags,
) -> Result<CheckResult> {
    let mut result = CheckResult::default();
    result.diagnostics.extend(decls.diagnostics.clone());

    // HEMTT's preprocessor strips comments and its AST spans are into that
    // processed buffer. We also normalize CRLF→LF so Position offsets (into the
    // virtual check.sqf) line up with Rust string byte indexes.
    let source_lf = source.replace("\r\n", "\n");
    let uses_crlf = source_lf.len() != source.len();

    let is_plain = path.ends_with(".sqf") && !path.ends_with(".d.sqfts");
    let (erased_lf, annotations, span_map, original_lf) = if is_plain {
        (
            source_lf.clone(),
            Vec::new(),
            SpanMap::default(),
            source_lf.clone(),
        )
    } else {
        match erase(&source_lf, &EraseOptions::default()) {
            Ok(e) => (e.text, e.annotations, e.span_map, e.original),
            Err(e) => {
                result.diagnostics.push(Diagnostic {
                    code: StsCode::SyntaxError,
                    severity: Severity::Error,
                    message: e.to_string(),
                    span: None,
                    related: vec![],
                });
                return Ok(result);
            }
        }
    };

    // Parse erased SQF via HEMTT (returns statements + Processed for span remap)
    let (statements, processed) = match parse_sqf(&erased_lf) {
        Ok(v) => v,
        Err(diags) => {
            for parse_diag in diags {
                let span = parse_diag.span.map(|span| {
                    let mapped = span_map.to_original(span.start)..span_map.to_original(span.end);
                    if uses_crlf {
                        lf_range_to_crlf(source, &mapped)
                    } else {
                        mapped
                    }
                });
                result.diagnostics.push(Diagnostic {
                    code: StsCode::SyntaxError,
                    severity: Severity::Error,
                    message: parse_diag.message,
                    span,
                    related: vec![],
                });
            }
            return Ok(result);
        }
    };

    let mut ctx = CheckCtx {
        db,
        flags: flags.clone(),
        symbols: decls.symbols.clone(),
        annotations: &annotations,
        span_map: &span_map,
        original: &original_lf,
        diagnostics: Vec::new(),
        typed_locals: HashMap::new(),
    };

    // Seed typed locals from annotations
    for ann in &annotations {
        match &ann.kind {
            AnnotationKind::TypedPrivate { name, ty, .. } => {
                ctx.typed_locals.insert(name.clone(), ty.clone());
            }
            AnnotationKind::TypedParam {
                name,
                ty,
                optional_nil,
                ..
            } => {
                let t = if *optional_nil {
                    ty.clone().or_nothing()
                } else {
                    ty.clone()
                };
                ctx.typed_locals.insert(name.clone(), t);
            }
            AnnotationKind::TypeAlias { name, ty } => {
                ctx.symbols.aliases.insert(name.clone(), ty.clone());
            }
            AnnotationKind::Interface { name, members } => {
                use crate::symbols::InterfaceMember;
                ctx.symbols.interfaces.insert(
                    name.clone(),
                    members
                        .iter()
                        .map(|(n, o, t)| InterfaceMember {
                            name: n.clone(),
                            optional: *o,
                            ty: t.clone(),
                        })
                        .collect(),
                );
            }
            AnnotationKind::DeclareVar { name, ty } => {
                ctx.symbols
                    .globals
                    .insert(name.clone(), (ty.clone(), path.to_string()));
            }
            AnnotationKind::DeclareFn { name, params, ret } => {
                use crate::symbols::FunctionSig;
                ctx.symbols.functions.insert(
                    name.clone(),
                    FunctionSig {
                        name: name.clone(),
                        params: params.clone(),
                        ret: ret.clone(),
                        file: path.to_string(),
                    },
                );
            }
            AnnotationKind::Cast { .. } => {}
        }
    }

    ctx.check_statements(&statements);

    // Map diagnostic spans: processed → erased LF → original LF → (optional) CRLF document
    for mut d in ctx.diagnostics {
        if let Some(span) = d.span.take() {
            let mapped = map_ast_span_to_source(&processed, &span_map, span);
            d.span = Some(if uses_crlf {
                lf_range_to_crlf(source, &mapped)
            } else {
                mapped
            });
        }
        for related in &mut d.related {
            let mapped = map_ast_span_to_source(&processed, &span_map, related.1.clone());
            related.1 = if uses_crlf {
                lf_range_to_crlf(source, &mapped)
            } else {
                mapped
            };
        }
        result.diagnostics.push(d);
    }
    let _ = path;
    let _ = erased_lf;
    Ok(result)
}

#[derive(Debug)]
struct ParseDiagnostic {
    message: String,
    span: Option<Range<usize>>,
}

fn parse_sqf(
    source_lf: &str,
) -> Result<(Statements, hemtt_workspace::reporting::Processed), Vec<ParseDiagnostic>> {
    let workspace = Workspace::builder()
        .memory()
        .finish(None, false, &PDriveOption::Disallow)
        .map_err(|e| vec![ParseDiagnostic::without_span(e.to_string())])?;
    let path = workspace
        .join("check.sqf")
        .map_err(|e| vec![ParseDiagnostic::without_span(e.to_string())])?;
    {
        let mut f = path
            .create_file()
            .map_err(|e| vec![ParseDiagnostic::without_span(e.to_string())])?;
        f.write_all(source_lf.as_bytes())
            .map_err(|e| vec![ParseDiagnostic::without_span(e.to_string())])?;
    }
    let processed = Processor::run(&path, &PreprocessorOptions::default()).map_err(|e| {
        vec![ParseDiagnostic::without_span(format!(
            "preprocessor: {e:?}"
        ))]
    })?;
    match hemtt_sqf::parser::run(&HemttDb::a3(false), &processed) {
        Ok(s) => Ok((s, processed)),
        Err(hemtt_sqf::parser::ParserError::ParsingError(codes))
        | Err(hemtt_sqf::parser::ParserError::LexingError(codes)) => Err(codes
            .iter()
            .map(|c| ParseDiagnostic::from_code(&**c, source_lf))
            .collect()),
    }
}

impl ParseDiagnostic {
    fn without_span(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    fn from_code(code: &dyn hemtt_workspace::reporting::Code, source_lf: &str) -> Self {
        if let Some(diag) = code.diagnostic() {
            let span = diag
                .labels
                .first()
                .map(|label| expand_empty_range(label.range().clone(), source_lf));
            Self {
                message: diag.message,
                span,
            }
        } else {
            Self {
                message: code.message(),
                span: None,
            }
        }
    }
}

fn expand_empty_range(range: Range<usize>, source: &str) -> Range<usize> {
    if range.start != range.end || range.start >= source.len() {
        return range;
    }

    let mut end = range.start + 1;
    while end < source.len() && !source.is_char_boundary(end) {
        end += 1;
    }
    range.start..end
}

/// Map an AST span (into HEMTT processed / comment-stripped output) back to the
/// LF original source via Processed mappings + annotation SpanMap.
fn map_ast_span_to_source(
    processed: &hemtt_workspace::reporting::Processed,
    span_map: &SpanMap,
    span: Range<usize>,
) -> Range<usize> {
    // Map inclusive endpoints through HEMTT's processed→source map, then through
    // the annotation SpanMap. Using end-1 avoids exclusive-end landing on the
    // *next* token (which produced multi-line false highlights).
    let erased_start = processed_offset_to_erased(processed, span.start);
    let erased_end = if span.end > span.start {
        let last_incl = processed_offset_to_erased(processed, span.end - 1);
        // Prefer the original end of the token that owns the last included offset.
        if let Some(m) = processed.mapping(span.end - 1) {
            m.original().end().offset().max(last_incl)
        } else {
            last_incl.saturating_add(1)
        }
    } else {
        erased_start
    };
    let start = span_map.to_original(erased_start);
    let end = span_map.to_original(erased_end.max(erased_start));
    start..end.max(start)
}

fn processed_offset_to_erased(
    processed: &hemtt_workspace::reporting::Processed,
    offset: usize,
) -> usize {
    let maps = processed.mappings(offset);
    let Some(m) = maps.last().copied().or_else(|| processed.mapping(offset)) else {
        return offset;
    };
    let p_start = m.processed_start().offset();
    let p_end = m.processed_end().offset();
    let o_start = m.original().start().offset();
    let o_end = m.original().end().offset();
    if p_end > p_start && offset >= p_start {
        let delta = offset - p_start;
        let o_len = o_end.saturating_sub(o_start);
        return o_start + delta.min(o_len);
    }
    o_start
}

/// Convert a byte range in LF-normalized text to the equivalent range in the
/// original CRLF document (`\n` → `\r\n` adds one byte per newline before the offset).
fn lf_range_to_crlf(crlf_source: &str, lf_range: &Range<usize>) -> Range<usize> {
    let start = lf_offset_to_crlf(crlf_source, lf_range.start);
    let end = lf_offset_to_crlf(crlf_source, lf_range.end);
    start..end.max(start)
}

fn lf_offset_to_crlf(crlf_source: &str, lf_offset: usize) -> usize {
    let lf = crlf_source.replace("\r\n", "\n");
    if lf_offset >= lf.len() {
        return crlf_source.len();
    }
    // Ensure we don't split a char
    let mut off = lf_offset;
    while off > 0 && !lf.is_char_boundary(off) {
        off -= 1;
    }
    let newlines = lf[..off].bytes().filter(|&b| b == b'\n').count();
    off + newlines
}

struct CheckCtx<'a> {
    db: &'a CommandDb,
    flags: CheckFlags,
    symbols: SymbolTable,
    #[allow(dead_code)]
    annotations: &'a [Annotation],
    #[allow(dead_code)]
    span_map: &'a SpanMap,
    #[allow(dead_code)]
    original: &'a str,
    diagnostics: Vec<Diagnostic>,
    typed_locals: HashMap<String, Type>,
}

impl CheckCtx<'_> {
    fn check_statements(&mut self, stmts: &Statements) {
        self.symbols.push_scope();
        // Seed scope with typed locals from params/private annotations
        for (name, ty) in self.typed_locals.clone() {
            self.symbols.define_local(&name, ty);
        }
        for stmt in stmts.content() {
            self.check_statement(stmt);
        }
        self.symbols.pop_scope();
    }

    fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::AssignLocal(name, expr, span) => {
                let ty = self.type_of(expr);
                if let Some(expected) = self.typed_locals.get(name).cloned() {
                    if !is_assignable(&ty, &expected, &self.flags) {
                        self.diagnostics.push(Diagnostic::error(
                            StsCode::AssignMismatch,
                            format!("`{name}` expected `{expected}`, got `{ty}`"),
                            span.clone(),
                        ));
                    }
                    self.symbols.define_local(name, expected);
                } else {
                    if self.flags.no_implicit_any && ty == Type::any() {
                        self.diagnostics.push(Diagnostic::error(
                            StsCode::ImplicitAny,
                            format!("`{name}` has implicit type `any`"),
                            span.clone(),
                        ));
                    }
                    self.symbols.define_local(name, ty);
                }
            }
            Statement::AssignGlobal(name, expr, span) => {
                // HEMTT classifies bare `_x = …` as AssignGlobal even when `_x` is a
                // local; still enforce the annotated / inferred local type.
                let ty = self.type_of(expr);
                if let Some(expected) = self.typed_locals.get(name).cloned() {
                    if !is_assignable(&ty, &expected, &self.flags) {
                        self.diagnostics.push(Diagnostic::error(
                            StsCode::AssignMismatch,
                            format!("`{name}` expected `{expected}`, got `{ty}`"),
                            span.clone(),
                        ));
                    }
                    self.symbols.define_local(name, expected);
                } else if let Some(expected) = self.symbols.lookup_local(name).cloned() {
                    if !is_assignable(&ty, &expected, &self.flags) {
                        self.diagnostics.push(Diagnostic::error(
                            StsCode::AssignMismatch,
                            format!("`{name}` expected `{expected}`, got `{ty}`"),
                            span.clone(),
                        ));
                    }
                } else if let Some((expected, _)) = self.symbols.globals.get(name).cloned() {
                    if !is_assignable(&ty, &expected, &self.flags) {
                        self.diagnostics.push(Diagnostic::error(
                            StsCode::AssignMismatch,
                            format!("`{ty}` not assignable to declared type `{expected}`"),
                            span.clone(),
                        ));
                    }
                }
            }
            Statement::Expression(expr, _) => {
                let _ = self.type_of(expr);
            }
        }
    }

    fn type_of(&mut self, expr: &Expression) -> Type {
        match expr {
            Expression::Number(_, _) => Type::Primitive(Primitive::Number),
            Expression::String(_, _, _) => Type::Primitive(Primitive::String),
            Expression::Boolean(_, _) => Type::Primitive(Primitive::Boolean),
            Expression::Array(elems, _) => {
                let tys: Vec<Type> = elems.iter().map(|e| self.type_of(e)).collect();
                if tys.is_empty() {
                    Type::Primitive(Primitive::Array)
                } else if tys.iter().all(|t| t == &tys[0]) {
                    // Could be tuple or array — prefer tuple when all known and not any
                    if tys.iter().all(|t| t != &Type::any()) {
                        Type::Tuple(tys.into_iter().map(|t| (t, false)).collect())
                    } else {
                        Type::ArrayOf(Box::new(tys[0].clone()))
                    }
                } else {
                    Type::Tuple(tys.into_iter().map(|t| (t, false)).collect())
                }
            }
            Expression::Code(stmts) => {
                self.check_statements(stmts);
                Type::Primitive(Primitive::Code)
            }
            Expression::Variable(name, _) => self.symbols.lookup_var(name),
            Expression::NularCommand(cmd, span) => {
                self.resolve_command(cmd.as_str(), CallKind::Nular, &[], span)
            }
            Expression::UnaryCommand(cmd, arg, span) => {
                let arg_ty = self.type_of(arg);
                // Special: isNil / isEqualType handled in binary/unary for narrowing later
                self.resolve_command(cmd.as_str(), CallKind::Unary, &[arg_ty], span)
            }
            Expression::BinaryCommand(cmd, left, right, span) => {
                self.type_binary(cmd, left, right, span)
            }
            Expression::ConsumeableArray(elems, span) => {
                self.type_of(&Expression::Array(elems.clone(), span.clone()))
            }
        }
    }

    fn type_binary(
        &mut self,
        cmd: &BinaryCommand,
        left: &Expression,
        right: &Expression,
        span: &Range<usize>,
    ) -> Type {
        let name = cmd.as_str();

        // call / spawn with declared functions: `args call fnName`
        if name.eq_ignore_ascii_case("call") || name.eq_ignore_ascii_case("spawn") {
            let fname = match right {
                Expression::Variable(n, _) => Some(n.as_str()),
                Expression::NularCommand(c, _) => Some(c.as_str()),
                _ => None,
            };
            if let Some(fname) = fname {
                if let Some(sig) = self.symbols.functions.get(fname).cloned() {
                    let arg_ty = self.type_of(left);
                    self.check_fn_args(&sig, &arg_ty, span);
                    return if name.eq_ignore_ascii_case("spawn") {
                        Type::Primitive(Primitive::ScriptHandle)
                    } else {
                        sig.ret
                    };
                }
            }
        }

        if name.eq_ignore_ascii_case("remoteExec") || name.eq_ignore_ascii_case("remoteExecCall") {
            // left is args, right is [funcName, targets, ...]
            if let Expression::Array(elems, _) = right {
                if let Some(Expression::String(func, fspan, _)) = elems.first() {
                    if let Some(sig) = self.symbols.functions.get(func.as_ref()).cloned() {
                        let arg_ty = self.type_of(left);
                        self.check_fn_args(&sig, &arg_ty, fspan);
                    }
                }
            }
        }

        let left_ty = self.type_of(left);
        let right_ty = self.type_of(right);

        // HashMap get/set with interfaces
        if name.eq_ignore_ascii_case("get") || name.eq_ignore_ascii_case("set") {
            if let Type::Named(iface) = &left_ty {
                if let Some(members) = self.symbols.interfaces.get(iface).cloned() {
                    if let Expression::String(key, key_span, _) = right {
                        if let Some(m) = members.iter().find(|m| m.name == key.as_ref()) {
                            if name.eq_ignore_ascii_case("get") {
                                return if m.optional {
                                    m.ty.clone().or_nothing()
                                } else {
                                    m.ty.clone()
                                };
                            }
                            // set: third arg is via array form usually `hm set [key, val]`
                        } else if self.flags.strict_hash_map {
                            self.diagnostics.push(Diagnostic::error(
                                StsCode::UnknownHashKey,
                                format!("unknown key `{key}` on interface `{iface}`"),
                                key_span.clone(),
                            ));
                        }
                    }
                }
            }
            // `hm set [key, val]` is binary set with array right
            if name.eq_ignore_ascii_case("set") {
                if let (Type::Named(iface), Expression::Array(elems, _)) = (&left_ty, right) {
                    if let Some(members) = self.symbols.interfaces.get(iface).cloned() {
                        if let Some(Expression::String(key, key_span, _)) = elems.first() {
                            if let Some(m) = members.iter().find(|m| m.name == key.as_ref()) {
                                if let Some(val) = elems.get(1) {
                                    let vt = self.type_of(val);
                                    if !is_assignable(&vt, &m.ty, &self.flags) {
                                        self.diagnostics.push(Diagnostic::error(
                                            StsCode::AssignMismatch,
                                            format!(
                                                "`{vt}` not assignable to `{ty}` for key `{key}`",
                                                ty = m.ty
                                            ),
                                            val.span(),
                                        ));
                                    }
                                }
                            } else if self.flags.strict_hash_map {
                                self.diagnostics.push(Diagnostic::error(
                                    StsCode::UnknownHashKey,
                                    format!("unknown key `{key}` on interface `{iface}`"),
                                    key_span.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // select / # element type
        if name == "#" || name.eq_ignore_ascii_case("select") {
            match &left_ty {
                Type::ArrayOf(inner) => return *inner.clone(),
                Type::Tuple(elems) => {
                    if let Expression::Number(n, _) = right {
                        let idx = n.0 as usize;
                        if let Some((t, _)) = elems.get(idx) {
                            return t.clone();
                        }
                    }
                    return Type::any();
                }
                _ => {}
            }
        }

        self.resolve_command(name, CallKind::Binary, &[left_ty, right_ty], span)
    }

    fn check_fn_args(
        &mut self,
        sig: &crate::symbols::FunctionSig,
        arg_ty: &Type,
        span: &Range<usize>,
    ) {
        let required: Vec<_> = sig.params.iter().filter(|p| !p.optional).collect();
        // Unpack tuple / array args
        let arg_tys: Vec<Type> = match arg_ty {
            Type::Tuple(elems) => elems.iter().map(|(t, _)| t.clone()).collect(),
            Type::ArrayOf(inner) => {
                // Homogeneous array — only check first param loosely
                if sig.params.is_empty() {
                    return;
                }
                if !is_assignable(inner, &sig.params[0].ty, &self.flags) {
                    self.diagnostics.push(Diagnostic::error(
                        StsCode::ArgMismatch,
                        format!(
                            "argument 1 is `{inner}[]` element, expected `{}`",
                            sig.params[0].ty
                        ),
                        span.clone(),
                    ));
                }
                return;
            }
            Type::Primitive(Primitive::Array) | Type::Primitive(Primitive::Any) => {
                // Untyped array / any — gradual, skip arity checks
                return;
            }
            other => {
                // Single bare value for 1-required-param functions
                if required.len() == 1
                    && !matches!(
                        required[0].ty,
                        Type::Primitive(Primitive::Array) | Type::Tuple(_) | Type::ArrayOf(_)
                    )
                {
                    vec![other.clone()]
                } else {
                    vec![other.clone()]
                }
            }
        };

        for (i, param) in sig.params.iter().enumerate() {
            if let Some(got) = arg_tys.get(i) {
                if !is_assignable(got, &param.ty, &self.flags) {
                    let code = match (&got, &param.ty) {
                        (Type::Brand(_), Type::Brand(_)) => StsCode::BrandMismatch,
                        _ => StsCode::ArgMismatch,
                    };
                    self.diagnostics.push(Diagnostic::error(
                        code,
                        format!("argument {} is `{got}`, expected `{}`", i + 1, param.ty),
                        span.clone(),
                    ));
                }
            } else if !param.optional {
                self.diagnostics.push(Diagnostic::error(
                    StsCode::ArgMismatch,
                    format!("missing argument {} (`{}`)", i + 1, param.name),
                    span.clone(),
                ));
            }
        }
    }

    fn resolve_command(
        &mut self,
        name: &str,
        kind: CallKind,
        args: &[Type],
        span: &Range<usize>,
    ) -> Type {
        let overloads = self.db.overloads_kind(name, kind);
        if overloads.is_empty() {
            return Type::any();
        }

        let any_arg = args.iter().any(|a| a == &Type::any());

        // First matching overload
        for ov in &overloads {
            if Self::args_match(args, &ov.params, &self.flags) {
                return ov.return_ty.clone();
            }
        }

        if any_arg {
            // Union of all returns
            let mut rets: Vec<Type> = overloads.iter().map(|o| o.return_ty.clone()).collect();
            rets.dedup();
            if rets.len() == 1 {
                return rets.pop().unwrap();
            }
            return Type::Union(rets).normalize();
        }

        // No match and no any — error
        self.diagnostics.push(Diagnostic::error(
            StsCode::ArgMismatch,
            format!(
                "no overload of `{name}` accepts arguments [{}]",
                args.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            span.clone(),
        ));
        Type::any()
    }

    fn args_match(args: &[Type], params: &[sqfts_db::ParamSig], flags: &CheckFlags) -> bool {
        // Wiki params list all named params; for unary/binary the first N map to args.
        // For binary commands typically 2 top-level args (left, right); right may be an array
        // covering many params — we only check the top-level arity loosely.
        if args.is_empty() && (params.is_empty() || params.iter().all(|p| p.optional)) {
            return true;
        }
        // Very gradual: if the expected type is any / array / named unknown, accept.
        let soft = |expected: &Type| {
            matches!(
                expected,
                Type::Primitive(Primitive::Any | Primitive::Array)
                    | Type::Named(_)
                    | Type::ArrayOf(_)
                    | Type::Tuple(_)
            )
        };
        if args.len() == 1 && !params.is_empty() {
            return soft(&params[0].ty) || is_assignable(&args[0], &params[0].ty, flags);
        }
        if args.len() == 2 && params.len() >= 2 {
            let left_ok = soft(&params[0].ty) || is_assignable(&args[0], &params[0].ty, flags);
            if !left_ok {
                return false;
            }
            if soft(&params[1].ty) || is_assignable(&args[1], &params[1].ty, flags) {
                return true;
            }
            if let Type::Tuple(elems) = &args[1] {
                let rest = &params[1..];
                if elems.len() >= rest.iter().filter(|p| !p.optional).count() {
                    return elems
                        .iter()
                        .zip(rest.iter())
                        .all(|((t, _), p)| soft(&p.ty) || is_assignable(t, &p.ty, flags));
                }
            }
            if matches!(
                args[1],
                Type::Primitive(Primitive::Any | Primitive::Array)
                    | Type::ArrayOf(_)
                    | Type::Tuple(_)
            ) {
                return true;
            }
            return false;
        }
        if args.is_empty() {
            return params.iter().all(|p| p.optional) || params.is_empty();
        }
        args.iter()
            .zip(params.iter())
            .all(|(a, p)| soft(&p.ty) || is_assignable(a, &p.ty, flags))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decls::{load_one, DeclarationSet};
    use sqfts_db::CommandDb;

    #[test]
    fn spec_section_8_example() {
        let db = CommandDb::load_default().unwrap();
        let mut decls = DeclarationSet::default();
        load_one(
            r#"declare project_serviceFee: number;
declare function TAG_fnc_checkPayment(_unit: object, _amount: number): boolean;
"#,
            "mission.d.sqfts",
            &mut decls,
        )
        .unwrap();

        let src = r#"private _ok = [player, "500"] call TAG_fnc_checkPayment;
project_serviceFee = true;
"#;
        let mut flags = CheckFlags::default();
        flags.check_plain_sqf = true;
        let result = check_source(src, "fn_payFee.sqf", &db, &decls, &flags).unwrap();
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == StsCode::ArgMismatch),
            "expected STS2003, got {:?}",
            result.diagnostics
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == StsCode::AssignMismatch),
            "expected STS2004, got {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn parser_diagnostics_are_plain_and_spanned() {
        let db = CommandDb::load_default().unwrap();
        let decls = DeclarationSet::default();
        let flags = CheckFlags {
            check_plain_sqf: true,
            ..Default::default()
        };

        let result = check_source("@", "bad.sqf", &db, &decls, &flags).unwrap();
        let diag = result
            .diagnostics
            .first()
            .expect("expected parse diagnostic");

        assert_eq!(diag.code, StsCode::SyntaxError);
        assert_eq!(diag.message, "Use of an invalid token");
        assert!(!diag.message.contains('\u{1b}'));
        assert_eq!(diag.span, Some(0..1));
    }

    #[test]
    fn bare_reassignment_checks_typed_local() {
        let db = CommandDb::load_default().unwrap();
        let decls = DeclarationSet::default();
        let flags = CheckFlags::default();

        // Bare `_testVar = …` is AssignGlobal in HEMTT, but must still respect
        // the annotated local type from `private _testVar: number = …`.
        let src = r#"private _testVar: number = 3;
_testVar = "test";
"#;
        let result = check_source(src, "reassign.sqfts", &db, &decls, &flags).unwrap();
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == StsCode::AssignMismatch),
            "expected STS2004 for string→number local reassignment, got {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn bare_reassignment_checks_inferred_local() {
        let db = CommandDb::load_default().unwrap();
        let decls = DeclarationSet::default();
        let flags = CheckFlags::default();

        let src = r#"private _n = 3;
_n = "test";
"#;
        let result = check_source(src, "reassign_inferred.sqfts", &db, &decls, &flags).unwrap();
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == StsCode::AssignMismatch),
            "expected STS2004 for string→number inferred local reassignment, got {:?}",
            result.diagnostics
        );
    }
}
