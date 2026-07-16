//! LSP backend: diagnostics, hover, completion.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use sqfts_check::Severity;
use sqfts_db::CallKind;
use sqfts_project::Project;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::line_index::{identifier_at, LineIndex};

const DEBOUNCE_MS: u64 = 300;

pub struct Backend {
    client: Client,
    project: Mutex<Option<Project>>,
    documents: DashMap<Url, String>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            project: Mutex::new(None),
            documents: DashMap::new(),
        }
    }

    async fn ensure_project(&self, root: Option<&Path>) -> anyhow::Result<()> {
        let mut guard = self.project.lock().await;
        if guard.is_some() {
            return Ok(());
        }
        let path = root
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".into());
        let project = Project::load(&path)?;
        *guard = Some(project);
        Ok(())
    }

    async fn publish_diagnostics_for(&self, uri: &Url, text: &str) {
        let path = uri_to_path(uri);
        let Some(path) = path else {
            return;
        };
        let result = {
            let guard = self.project.lock().await;
            let Some(project) = guard.as_ref() else {
                return;
            };
            match project.check_file(&path, text) {
                Ok(r) => r,
                Err(e) => {
                    self.client
                        .log_message(MessageType::ERROR, format!("check failed: {e:#}"))
                        .await;
                    return;
                }
            }
        };
        let idx = LineIndex::new(text);
        let diags: Vec<Diagnostic> = result
            .diagnostics
            .iter()
            .map(|d| map_diagnostic(d, text, &idx, uri))
            .collect();
        self.client
            .publish_diagnostics(uri.clone(), diags, None)
            .await;
    }

    async fn hover_markdown(&self, text: &str, pos: Position) -> Option<String> {
        let idx = LineIndex::new(text);
        let offset = idx.offset(text, pos);
        let (_, name) = identifier_at(text, offset)?;
        let guard = self.project.lock().await;
        let project = guard.as_ref()?;

        if let Some(sig) = project.decls.symbols.functions.get(&name) {
            let params: Vec<String> = sig
                .params
                .iter()
                .map(|p| {
                    if p.optional {
                        format!("{}?: {}", p.name, p.ty)
                    } else {
                        format!("{}: {}", p.name, p.ty)
                    }
                })
                .collect();
            return Some(format!(
                "```sqfts\ndeclare function {}({}): {};\n```\n_{}_",
                sig.name,
                params.join(", "),
                sig.ret,
                sig.file
            ));
        }
        if let Some((ty, file)) = project.decls.symbols.globals.get(&name) {
            return Some(format!(
                "```sqfts\ndeclare {name}: {ty};\n```\n_{file}_"
            ));
        }
        if let Some(ovs) = project.db.overloads(&name) {
            let mut lines = vec![format!("**engine command** `{name}`\n")];
            for ov in ovs {
                lines.push(format_overload(&name, ov));
            }
            return Some(lines.join("\n"));
        }
        None
    }

    async fn completions(&self, text: &str, pos: Position) -> Vec<CompletionItem> {
        let idx = LineIndex::new(text);
        let offset = idx.offset(text, pos);
        let prefix = identifier_at(text, offset)
            .map(|(_, n)| n.to_ascii_lowercase())
            .unwrap_or_default();

        let guard = self.project.lock().await;
        let Some(project) = guard.as_ref() else {
            return vec![];
        };
        let mut items = Vec::new();
        for name in project.db.command_names() {
            if prefix.is_empty() || name.starts_with(&prefix) {
                items.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some("engine command".into()),
                    ..Default::default()
                });
            }
            if items.len() > 200 {
                break;
            }
        }
        for name in project.decls.symbols.functions.keys() {
            if prefix.is_empty() || name.to_ascii_lowercase().starts_with(&prefix) {
                items.push(CompletionItem {
                    label: name.clone(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some("declared function".into()),
                    ..Default::default()
                });
            }
        }
        for name in project.decls.symbols.globals.keys() {
            if prefix.is_empty() || name.to_ascii_lowercase().starts_with(&prefix) {
                items.push(CompletionItem {
                    label: name.clone(),
                    kind: Some(CompletionItemKind::VARIABLE),
                    detail: Some("declared global".into()),
                    ..Default::default()
                });
            }
        }
        for kw in [
            "type",
            "interface",
            "declare",
            "function",
            "as",
            "private",
            "params",
        ] {
            if prefix.is_empty() || kw.starts_with(&prefix) {
                items.push(CompletionItem {
                    label: kw.into(),
                    kind: Some(CompletionItemKind::KEYWORD),
                    ..Default::default()
                });
            }
        }
        items
    }

    async fn recheck_all_open(&self) {
        let entries: Vec<(Url, String)> = self
            .documents
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect();
        for (uri, text) in entries {
            self.publish_diagnostics_for(&uri, &text).await;
        }
    }
}

fn format_overload(name: &str, ov: &sqfts_db::Overload) -> String {
    let params: Vec<String> = ov
        .params
        .iter()
        .map(|p| {
            let opt = if p.optional { "?" } else { "" };
            format!("{}{opt}: {}", p.name, p.ty)
        })
        .collect();
    let shape = match ov.kind {
        CallKind::Nular => format!("`{name}` → `{}`", ov.return_ty),
        CallKind::Unary => format!(
            "`{name} {}` → `{}`",
            params.first().cloned().unwrap_or_else(|| "_".into()),
            ov.return_ty
        ),
        CallKind::Binary => format!(
            "`{} {name} {}` → `{}`",
            params.first().cloned().unwrap_or_else(|| "_".into()),
            params.get(1).cloned().unwrap_or_else(|| "_".into()),
            ov.return_ty
        ),
    };
    format!("- {shape}")
}

fn map_diagnostic(
    d: &sqfts_check::Diagnostic,
    text: &str,
    idx: &LineIndex,
    uri: &Url,
) -> Diagnostic {
    let range = d
        .span
        .as_ref()
        .map(|s| idx.range(text, s.start, s.end))
        .unwrap_or_else(|| Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 0,
            },
        });
    let severity = match d.severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
        Severity::Note => DiagnosticSeverity::INFORMATION,
    };
    let related = d
        .related
        .iter()
        .map(|(msg, span)| DiagnosticRelatedInformation {
            location: Location {
                uri: uri.clone(),
                range: idx.range(text, span.start, span.end),
            },
            message: msg.clone(),
        })
        .collect::<Vec<_>>();
    Diagnostic {
        range,
        severity: Some(severity),
        code: Some(NumberOrString::String(d.code.as_str().into())),
        source: Some("sqfts".into()),
        message: d.message.clone(),
        related_information: if related.is_empty() {
            None
        } else {
            Some(related)
        },
        ..Default::default()
    }
}

fn uri_to_path(uri: &Url) -> Option<PathBuf> {
    uri.to_file_path().ok()
}

fn path_is_decl_or_config(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    name.ends_with(".d.sqfts") || name == "sqfts.toml"
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        let root = params
            .root_uri
            .as_ref()
            .and_then(|u| u.to_file_path().ok())
            .or_else(|| {
                params
                    .workspace_folders
                    .as_ref()
                    .and_then(|f| f.first())
                    .and_then(|f| f.uri.to_file_path().ok())
            });
        if let Err(e) = self.ensure_project(root.as_deref()).await {
            self.client
                .log_message(
                    MessageType::WARNING,
                    format!("project load deferred: {e:#}"),
                )
                .await;
        }
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec!["_".into()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "sqfts-language-server".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "SQFts language server ready")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        self.documents.insert(uri.clone(), text.clone());
        if self.project.lock().await.is_none() {
            let root = uri_to_path(&uri).and_then(|p| p.parent().map(Path::to_path_buf));
            let _ = self.ensure_project(root.as_deref()).await;
        }
        self.publish_diagnostics_for(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().last() {
            self.documents.insert(uri.clone(), change.text);
        }
        let docs = Arc::new(self.documents.clone());
        let project = {
            let g = self.project.lock().await;
            g.clone()
        };
        let client = self.client.clone();
        let uri2 = uri;
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(DEBOUNCE_MS)).await;
            let Some(text) = docs.get(&uri2).map(|e| e.value().clone()) else {
                return;
            };
            let Some(project) = project else {
                return;
            };
            let Some(path) = uri_to_path(&uri2) else {
                return;
            };
            let Ok(result) = project.check_file(&path, &text) else {
                return;
            };
            let idx = LineIndex::new(&text);
            let diags: Vec<Diagnostic> = result
                .diagnostics
                .iter()
                .map(|d| map_diagnostic(d, &text, &idx, &uri2))
                .collect();
            client.publish_diagnostics(uri2, diags, None).await;
        });
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(path) = uri_to_path(&uri) {
            if path_is_decl_or_config(&path) {
                let mut guard = self.project.lock().await;
                if let Some(project) = guard.as_mut() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let result = if name == "sqfts.toml" {
                        project.reload_config()
                    } else {
                        project.reload_declarations()
                    };
                    if let Err(e) = result {
                        self.client
                            .log_message(MessageType::ERROR, format!("reload failed: {e:#}"))
                            .await;
                    }
                }
                drop(guard);
                self.recheck_all_open().await;
                return;
            }
        }
        if let Some(text) = self.documents.get(&uri) {
            self.publish_diagnostics_for(&uri, text.value()).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let Some(text) = self.documents.get(&uri).map(|e| e.value().clone()) else {
            return Ok(None);
        };
        Ok(self.hover_markdown(&text, pos).await.map(|value| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
            range: None,
        }))
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let Some(text) = self.documents.get(&uri).map(|e| e.value().clone()) else {
            return Ok(None);
        };
        let items = self.completions(&text, pos).await;
        Ok(Some(CompletionResponse::Array(items)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqfts_check::{Diagnostic as SqDiag, Severity as SqSev, StsCode};

    #[test]
    fn maps_diagnostic_code() {
        let text = "private _x = 1;";
        let idx = LineIndex::new(text);
        let uri = Url::parse("file:///test.sqfts").unwrap();
        let d = SqDiag {
            code: StsCode::ArgMismatch,
            severity: SqSev::Error,
            message: "bad".into(),
            span: Some(0..7),
            related: vec![],
        };
        let lsp = map_diagnostic(&d, text, &idx, &uri);
        assert_eq!(
            lsp.code,
            Some(NumberOrString::String(StsCode::ArgMismatch.as_str().into()))
        );
        assert_eq!(lsp.range.start.line, 0);
    }
}
