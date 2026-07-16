//! Engine-command overload database.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use serde::Deserialize;
use sqfts_syntax::Type;

use crate::convert::wiki_name_to_type;
use crate::overlay::Overlay;

/// Call arity shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CallKind {
    Nular,
    Unary,
    Binary,
}

/// One parameter in an overload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamSig {
    /// Parameter name from the wiki.
    pub name: String,
    /// Expected type.
    pub ty: Type,
    /// Whether the parameter is optional.
    pub optional: bool,
}

/// One syntax overload for a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Overload {
    /// Nular / unary / binary.
    pub kind: CallKind,
    /// Parameters in order (left then right for binary).
    pub params: Vec<ParamSig>,
    /// Return type.
    pub return_ty: Type,
}

/// Lookup table of engine commands.
#[derive(Debug, Clone, Default)]
pub struct CommandDb {
    /// Lowercase command name → overloads.
    commands: IndexMap<String, Vec<Overload>>,
}

#[derive(Debug, Deserialize)]
struct YamlCommand {
    name: String,
    #[serde(default)]
    alias: Vec<String>,
    #[serde(default)]
    syntax: Vec<YamlSyntax>,
}

#[derive(Debug, Deserialize)]
struct YamlSyntax {
    call: YamlCall,
    #[serde(default)]
    params: Vec<YamlParam>,
    #[serde(rename = "return")]
    ret: YamlReturn,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum YamlCall {
    Nular,
    Unary {
        #[serde(default)]
        #[allow(dead_code)]
        right: String,
    },
    Binary {
        #[serde(default)]
        #[allow(dead_code)]
        left: String,
        #[serde(default)]
        #[allow(dead_code)]
        right: String,
    },
}

#[derive(Debug, Deserialize)]
struct YamlParam {
    name: String,
    #[serde(rename = "type")]
    typ: String,
    #[serde(default)]
    optional: bool,
}

#[derive(Debug, Deserialize)]
struct YamlReturn {
    #[serde(rename = "type")]
    typ: String,
}

impl CommandDb {
    /// Load from Phase 1 YAML (`out/commands`) and optional overlay patches.
    pub fn load_default() -> Result<Self> {
        let candidates = command_dir_candidates();
        for dir in &candidates {
            if dir.is_dir() {
                let overlay = {
                    let patches = dir.parent().map(|p| p.join("patches"));
                    match patches {
                        Some(p) if p.is_dir() => Overlay::load_dir(&p).unwrap_or_default(),
                        _ => Overlay::default(),
                    }
                };
                return Self::from_phase1_dir(dir, &overlay);
            }
        }
        // Last resort: try arma3-wiki git/dist (may panic on bad embeds — catch via thread)
        match std::panic::catch_unwind(arma3_wiki::Wiki::load_dist) {
            Ok(wiki) => Self::from_arma3_wiki(&wiki, &Overlay::default()),
            Err(_) => bail!(
                "no Phase 1 command database found (tried {:?}) and arma3-wiki dist failed to load",
                candidates
            ),
        }
    }

    /// Load all `*.yml` command files from a Phase 1 output directory.
    pub fn from_phase1_dir(dir: &Path, overlay: &Overlay) -> Result<Self> {
        let mut commands = IndexMap::new();
        for entry in std::fs::read_dir(dir)
            .with_context(|| format!("reading command dir {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "yml" && ext != "yaml" {
                continue;
            }
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            let cmd: YamlCommand = serde_yaml::from_str(&text)
                .with_context(|| format!("parsing {}", path.display()))?;
            let key = cmd.name.to_ascii_lowercase();
            let mut overloads = Vec::new();
            for (si, syntax) in cmd.syntax.iter().enumerate() {
                let kind = match &syntax.call {
                    YamlCall::Nular => CallKind::Nular,
                    YamlCall::Unary { .. } => CallKind::Unary,
                    YamlCall::Binary { .. } => CallKind::Binary,
                };
                let mut params: Vec<ParamSig> = syntax
                    .params
                    .iter()
                    .map(|p| ParamSig {
                        name: p.name.clone(),
                        ty: wiki_name_to_type(&p.typ),
                        optional: p.optional,
                    })
                    .collect();
                let mut return_ty = wiki_name_to_type(&syntax.ret.typ);

                if let Some(ov) = overlay.commands.get(&key) {
                    for (osi, pi, ty) in &ov.params {
                        if *osi == si {
                            if let Some(slot) = params.get_mut(*pi) {
                                if slot.ty == Type::any() {
                                    slot.ty = ty.clone();
                                }
                            }
                        }
                    }
                    for (osi, ty) in &ov.returns {
                        if *osi == si && return_ty == Type::any() {
                            return_ty = ty.clone();
                        }
                    }
                }

                overloads.push(Overload {
                    kind,
                    params,
                    return_ty,
                });
            }
            commands.insert(key, overloads.clone());
            for alias in &cmd.alias {
                commands
                    .entry(alias.to_ascii_lowercase())
                    .or_insert_with(|| overloads.clone());
            }
        }
        Ok(Self { commands })
    }

    /// Build from an arma3-wiki `Wiki` instance (0.5.x left/right Param model).
    pub fn from_arma3_wiki(wiki: &arma3_wiki::Wiki, overlay: &Overlay) -> Result<Self> {
        use arma3_wiki::model::Call;
        use crate::convert::wiki_value_to_type;

        let mut commands = IndexMap::new();
        for (name, cmd) in wiki.commands().iter() {
            let key = name.to_ascii_lowercase();
            let mut overloads = Vec::new();
            for (si, syntax) in cmd.syntax().iter().enumerate() {
                let kind = match syntax.call() {
                    Call::Nular => CallKind::Nular,
                    Call::Unary(_) => CallKind::Unary,
                    Call::Binary(_, _) => CallKind::Binary,
                };
                let mut params = Vec::new();
                if let Some(p) = syntax.left() {
                    params.push(wiki_param_to_sig("left", p));
                }
                if let Some(p) = syntax.right() {
                    params.push(wiki_param_to_sig("right", p));
                }
                let mut return_ty = wiki_value_to_type(syntax.ret().typ());
                if let Some(ov) = overlay.commands.get(&key) {
                    for (osi, pi, ty) in &ov.params {
                        if *osi == si {
                            if let Some(slot) = params.get_mut(*pi) {
                                if slot.ty == Type::any() {
                                    slot.ty = ty.clone();
                                }
                            }
                        }
                    }
                    for (osi, ty) in &ov.returns {
                        if *osi == si && return_ty == Type::any() {
                            return_ty = ty.clone();
                        }
                    }
                }
                overloads.push(Overload {
                    kind,
                    params,
                    return_ty,
                });
            }
            commands.insert(key, overloads.clone());
            for alias in cmd.alias() {
                commands
                    .entry(alias.to_ascii_lowercase())
                    .or_insert_with(|| overloads.clone());
            }
        }
        Ok(Self { commands })
    }

    /// Number of distinct command names.
    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Whether the database is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Look up overloads for a command (case-insensitive).
    #[must_use]
    pub fn overloads(&self, name: &str) -> Option<&[Overload]> {
        self.commands
            .get(&name.to_ascii_lowercase())
            .map(|v| v.as_slice())
    }

    /// Overloads matching a call kind.
    #[must_use]
    pub fn overloads_kind(&self, name: &str, kind: CallKind) -> Vec<&Overload> {
        self.overloads(name)
            .map(|ovs| ovs.iter().filter(|o| o.kind == kind).collect())
            .unwrap_or_default()
    }

    /// Iterate all command names (lowercase keys).
    pub fn command_names(&self) -> impl Iterator<Item = &str> {
        self.commands.keys().map(|s| s.as_str())
    }
}

fn wiki_param_to_sig(fallback_name: &str, p: &arma3_wiki::model::Param) -> ParamSig {
    use arma3_wiki::model::Param;
    use crate::convert::wiki_value_to_type;
    let (name, optional) = match p {
        Param::Item(item) => (item.name().to_string(), item.optional()),
        _ => (fallback_name.to_string(), false),
    };
    ParamSig {
        name,
        ty: wiki_value_to_type(&p.as_value()),
        optional,
    }
}

fn command_dir_candidates() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(v) = std::env::var("SQFTS_COMMANDS_DIR") {
        dirs.push(PathBuf::from(v));
    }
    // workspace-relative when running from repo root
    dirs.push(PathBuf::from("out/commands"));
    // relative to this crate → ../../out/commands
    dirs.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../out/commands"));
    dirs
}

/// Shared database handle.
pub type SharedDb = Arc<CommandDb>;

/// Convenience: load once into an Arc.
pub fn load_shared() -> Result<SharedDb> {
    Ok(Arc::new(CommandDb::load_default().context("CommandDb")?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_and_spot_checks() {
        let db = CommandDb::load_default().expect("load commands");
        assert!(
            db.len() > 2000,
            "expected thousands of commands, got {}",
            db.len()
        );

        let set_damage = db.overloads("setDamage").expect("setDamage");
        assert!(!set_damage.is_empty());
        assert!(set_damage.iter().any(|o| o.return_ty == Type::nothing()));

        let get_pos = db.overloads("getPos").expect("getPos");
        assert!(!get_pos.is_empty());

        assert!(db.overloads("addAction").is_some());
        assert!(db.overloads("forEach").is_some());
        assert!(db.overloads("select").is_some());
        assert!(db.overloads("remoteExec").is_some());
        assert!(db.overloads("player").is_some());
    }
}
