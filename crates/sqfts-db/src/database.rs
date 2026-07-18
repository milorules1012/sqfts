//! Engine-command overload database.

use std::sync::Arc;

use anyhow::{Context, Result};
use indexmap::IndexMap;
use sqfts_syntax::Type;

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
    /// Parameters in order (wiki `params` list; binary left then right).
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

impl CommandDb {
    /// Load from arma3-wiki (git refresh, then embedded dist fallback), matching HEMTT.
    pub fn load_default() -> Result<Self> {
        Self::from_arma3_wiki(&load_wiki(false))
    }

    /// Load with an explicit force-pull of the arma3-wiki `dist` branch.
    pub fn load(force_pull: bool) -> Result<Self> {
        Self::from_arma3_wiki(&load_wiki(force_pull))
    }

    /// Build from an arma3-wiki `Wiki` instance (0.4.x `params` model, same as HEMTT).
    pub fn from_arma3_wiki(wiki: &arma3_wiki::Wiki) -> Result<Self> {
        use crate::convert::wiki_value_to_type;
        use arma3_wiki::model::Call;

        let mut commands = IndexMap::new();
        for (name, cmd) in wiki.commands().iter() {
            let key = name.to_ascii_lowercase();
            let mut overloads = Vec::new();
            for syntax in cmd.syntax() {
                let kind = match syntax.call() {
                    Call::Nular => CallKind::Nular,
                    Call::Unary(_) => CallKind::Unary,
                    Call::Binary(_, _) => CallKind::Binary,
                };
                let params: Vec<ParamSig> = syntax
                    .params()
                    .iter()
                    .map(|p| ParamSig {
                        name: p.name().to_string(),
                        ty: wiki_value_to_type(p.typ()),
                        optional: p.optional(),
                    })
                    .collect();
                let return_ty = wiki_value_to_type(&syntax.ret().0);
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

/// Prefer a git checkout of arma3-wiki `dist`; fall back to the crate's embedded snapshot.
fn load_wiki(force_pull: bool) -> arma3_wiki::Wiki {
    arma3_wiki::Wiki::load_git(force_pull).unwrap_or_else(|_| arma3_wiki::Wiki::load_dist())
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

        let random = db.overloads("random").expect("random");
        assert!(
            random.iter().any(|o| {
                o.kind == CallKind::Unary
                    && o.params
                        .iter()
                        .any(|p| p.ty == Type::Primitive(sqfts_syntax::Primitive::Number))
            }),
            "random should have a Number unary param, got {random:?}"
        );
    }
}
