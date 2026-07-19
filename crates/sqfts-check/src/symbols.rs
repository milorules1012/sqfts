//! Symbol tables for locals, globals, functions, aliases, interfaces.

use std::collections::HashMap;

use sqfts_syntax::{FnParam, Type};

/// Declared function signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSig {
    pub name: String,
    pub params: Vec<FnParam>,
    pub ret: Type,
    pub file: String,
}

/// Interface member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceMember {
    pub name: String,
    pub optional: bool,
    pub ty: Type,
}

/// Project-level and file-level symbols.
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    pub globals: HashMap<String, (Type, String)>,
    pub functions: HashMap<String, FunctionSig>,
    pub aliases: HashMap<String, Type>,
    pub interfaces: HashMap<String, Vec<InterfaceMember>>,
    /// Local stack: each scope is a map.
    pub locals: Vec<HashMap<String, Type>>,
}

impl SymbolTable {
    #[must_use]
    pub fn new() -> Self {
        let mut s = Self::default();
        s.push_scope();
        s
    }

    pub fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.locals.pop();
    }

    pub fn define_local(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    #[must_use]
    pub fn lookup_local(&self, name: &str) -> Option<&Type> {
        for scope in self.locals.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t);
            }
        }
        None
    }

    #[must_use]
    pub fn lookup_var(&self, name: &str) -> Type {
        if let Some(t) = self.lookup_local(name) {
            return t.clone();
        }
        if let Some((t, _)) = self.globals.get(name) {
            return t.clone();
        }
        Type::any()
    }

    /// Resolve a named type (alias or interface → hashMap-like named).
    #[must_use]
    pub fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Named(n) => {
                if let Some(a) = self.aliases.get(n) {
                    return self.resolve_type(a);
                }
                if self.interfaces.contains_key(n) {
                    return Type::Named(n.clone()); // keep interface identity
                }
                ty.clone()
            }
            Type::ArrayOf(inner) => Type::ArrayOf(Box::new(self.resolve_type(inner))),
            Type::Tuple(elems) => Type::Tuple(
                elems
                    .iter()
                    .map(|(t, o)| (self.resolve_type(t), *o))
                    .collect(),
            ),
            Type::Code { params, ret } => Type::Code {
                params: params
                    .iter()
                    .map(|p| sqfts_syntax::CodeParam {
                        name: p.name.clone(),
                        ty: self.resolve_type(&p.ty),
                        optional: p.optional,
                    })
                    .collect(),
                ret: Box::new(self.resolve_type(ret)),
            },
            Type::Union(parts) => Type::Union(parts.iter().map(|p| self.resolve_type(p)).collect()),
            other => other.clone(),
        }
    }
}
