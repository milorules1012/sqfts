//! Surface type model for SQFts (SPEC §1).

use std::fmt;

/// Primitive / branded names from SPEC §1.1 / §1.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    /// Gradual type.
    Any,
    /// `nil` / no value.
    Nothing,
    Number,
    String,
    Boolean,
    /// Untyped array (`any[]`).
    Array,
    Code,
    Object,
    Group,
    Side,
    Config,
    Control,
    Display,
    Task,
    Location,
    Namespace,
    HashMap,
    TeamMember,
    ScriptHandle,
    DiaryRecord,
    StructuredText,
    Exception,
    EdenEntity,
    EdenId,
    ForType,
    IfType,
    SwitchType,
    WhileType,
    WithType,
}

impl Primitive {
    /// Parse a lowerCamelCase primitive name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "any" => Self::Any,
            "nothing" => Self::Nothing,
            "number" => Self::Number,
            "string" => Self::String,
            "boolean" => Self::Boolean,
            "array" => Self::Array,
            "code" => Self::Code,
            "object" => Self::Object,
            "group" => Self::Group,
            "side" => Self::Side,
            "config" => Self::Config,
            "control" => Self::Control,
            "display" => Self::Display,
            "task" => Self::Task,
            "location" => Self::Location,
            "namespace" => Self::Namespace,
            "hashMap" => Self::HashMap,
            "teamMember" => Self::TeamMember,
            "scriptHandle" => Self::ScriptHandle,
            "diaryRecord" => Self::DiaryRecord,
            "structuredText" => Self::StructuredText,
            "exception" => Self::Exception,
            "edenEntity" => Self::EdenEntity,
            "edenId" => Self::EdenId,
            "forType" => Self::ForType,
            "ifType" => Self::IfType,
            "switchType" => Self::SwitchType,
            "whileType" => Self::WhileType,
            "withType" => Self::WithType,
            _ => return None,
        })
    }

    /// LowerCamelCase keyword spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Nothing => "nothing",
            Self::Number => "number",
            Self::String => "string",
            Self::Boolean => "boolean",
            Self::Array => "array",
            Self::Code => "code",
            Self::Object => "object",
            Self::Group => "group",
            Self::Side => "side",
            Self::Config => "config",
            Self::Control => "control",
            Self::Display => "display",
            Self::Task => "task",
            Self::Location => "location",
            Self::Namespace => "namespace",
            Self::HashMap => "hashMap",
            Self::TeamMember => "teamMember",
            Self::ScriptHandle => "scriptHandle",
            Self::DiaryRecord => "diaryRecord",
            Self::StructuredText => "structuredText",
            Self::Exception => "exception",
            Self::EdenEntity => "edenEntity",
            Self::EdenId => "edenId",
            Self::ForType => "forType",
            Self::IfType => "ifType",
            Self::SwitchType => "switchType",
            Self::WhileType => "whileType",
            Self::WithType => "withType",
        }
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Nominal brand over a structural array/tuple (SPEC §1.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Brand {
    Position2D,
    Position3D,
    PositionATL,
    PositionASL,
    PositionASLW,
    PositionAGL,
    PositionAGLS,
    PositionRelative,
    /// Union of all position brands.
    Position,
    Vector2D,
    Vector3D,
    Waypoint,
    Color,
    TurretPath,
    TreePath,
    UnitLoadout,
}

impl Brand {
    /// Parse a branded type name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "position2D" => Self::Position2D,
            "position3D" => Self::Position3D,
            "positionATL" => Self::PositionATL,
            "positionASL" => Self::PositionASL,
            "positionASLW" => Self::PositionASLW,
            "positionAGL" => Self::PositionAGL,
            "positionAGLS" => Self::PositionAGLS,
            "positionRelative" => Self::PositionRelative,
            "position" => Self::Position,
            "vector2D" => Self::Vector2D,
            "vector3D" => Self::Vector3D,
            "waypoint" => Self::Waypoint,
            "color" => Self::Color,
            "turretPath" => Self::TurretPath,
            "treePath" => Self::TreePath,
            "unitLoadout" => Self::UnitLoadout,
            _ => return None,
        })
    }

    /// Surface spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Position2D => "position2D",
            Self::Position3D => "position3D",
            Self::PositionATL => "positionATL",
            Self::PositionASL => "positionASL",
            Self::PositionASLW => "positionASLW",
            Self::PositionAGL => "positionAGL",
            Self::PositionAGLS => "positionAGLS",
            Self::PositionRelative => "positionRelative",
            Self::Position => "position",
            Self::Vector2D => "vector2D",
            Self::Vector3D => "vector3D",
            Self::Waypoint => "waypoint",
            Self::Color => "color",
            Self::TurretPath => "turretPath",
            Self::TreePath => "treePath",
            Self::UnitLoadout => "unitLoadout",
        }
    }
}

impl fmt::Display for Brand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A SQFts type expression.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Primitive keyword.
    Primitive(Primitive),
    /// Branded array/tuple.
    Brand(Brand),
    /// Named alias / interface reference.
    Named(String),
    /// Homogeneous array `T[]`.
    ArrayOf(Box<Type>),
    /// Fixed-shape tuple; each element may be optional (`T?`).
    Tuple(Vec<(Type, bool)>),
    /// Union `A | B | …`.
    Union(Vec<Type>),
}

impl Type {
    /// The gradual type.
    #[must_use]
    pub fn any() -> Self {
        Self::Primitive(Primitive::Any)
    }

    /// `nothing`.
    #[must_use]
    pub fn nothing() -> Self {
        Self::Primitive(Primitive::Nothing)
    }

    /// Wrap with `| nothing` if not already present.
    #[must_use]
    pub fn or_nothing(self) -> Self {
        match &self {
            Self::Union(parts) if parts.iter().any(|t| *t == Self::nothing()) => self,
            Self::Primitive(Primitive::Nothing) => self,
            Self::Union(parts) => {
                let mut parts = parts.clone();
                parts.push(Self::nothing());
                Self::Union(parts)
            }
            _ => Self::Union(vec![self, Self::nothing()]),
        }
    }

    /// Flatten nested unions and deduplicate.
    #[must_use]
    pub fn normalize(self) -> Self {
        match self {
            Self::Union(parts) => {
                let mut flat = Vec::new();
                for p in parts {
                    match p.normalize() {
                        Self::Union(inner) => flat.extend(inner),
                        other => flat.push(other),
                    }
                }
                flat.dedup();
                if flat.len() == 1 {
                    flat.pop().unwrap()
                } else {
                    Self::Union(flat)
                }
            }
            Self::ArrayOf(inner) => Self::ArrayOf(Box::new(inner.normalize())),
            Self::Tuple(elems) => Self::Tuple(
                elems
                    .into_iter()
                    .map(|(t, opt)| (t.normalize(), opt))
                    .collect(),
            ),
            other => other,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive(p) => write!(f, "{p}"),
            Self::Brand(b) => write!(f, "{b}"),
            Self::Named(n) => write!(f, "{n}"),
            Self::ArrayOf(inner) => write!(f, "{inner}[]"),
            Self::Tuple(elems) => {
                write!(f, "[")?;
                for (i, (t, opt)) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{t}")?;
                    if *opt {
                        write!(f, "?")?;
                    }
                }
                write!(f, "]")
            }
            Self::Union(parts) => {
                for (i, p) in parts.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{p}")?;
                }
                Ok(())
            }
        }
    }
}
