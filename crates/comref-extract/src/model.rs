//! Intermediate representation for extracted COMREF commands.

use serde::{Deserialize, Serialize};

/// Canonical SQF value type aligned with `arma3_wiki::model::Value`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SqfType {
    Anything,
    Array,
    Boolean,
    Code,
    Config,
    Control,
    DiaryRecord,
    Display,
    EdenEntity,
    EdenID,
    ExceptionHandle,
    ForType,
    Group,
    HashMap,
    HashMapKey,
    IfType,
    Location,
    Namespace,
    Nothing,
    Number,
    /// Numeric enum from wiki (`0`, `1`, …).
    NumberEnum(Vec<i32>),
    Object,
    ScriptHandle,
    Side,
    String,
    /// String enum from wiki (`"west"`, …).
    StringEnum(Vec<String>),
    StructuredText,
    SwitchType,
    Task,
    TeamMember,
    Path,
    TurretPath,
    UnitLoadoutArray,
    Position,
    Position2d,
    Position3d,
    Position3dASL,
    Position3DASLW,
    Position3dATL,
    Position3dAGL,
    Position3dAGLS,
    Position3dRelative,
    Vector,
    Vector2d,
    Vector3d,
    Waypoint,
    WhileType,
    WithType,
    Color,
    Unknown,
    /// Homogeneous array: `Array of <T>`
    ArrayOf(Box<SqfType>),
    /// Union: `A or B`
    OneOf(Vec<SqfType>),
}

impl SqfType {
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    /// Compact name for reports / YAML type tags that match arma3-wiki style.
    #[must_use]
    pub fn wiki_name(&self) -> String {
        match self {
            Self::Anything => "Anything".into(),
            Self::Array => "Array".into(),
            Self::Boolean => "Boolean".into(),
            Self::Code => "Code".into(),
            Self::Config => "Config".into(),
            Self::Control => "Control".into(),
            Self::DiaryRecord => "DiaryRecord".into(),
            Self::Display => "Display".into(),
            Self::EdenEntity => "EdenEntity".into(),
            Self::EdenID => "EdenID".into(),
            Self::ExceptionHandle => "ExceptionHandle".into(),
            Self::ForType => "ForType".into(),
            Self::Group => "Group".into(),
            Self::HashMap => "HashMap".into(),
            Self::HashMapKey => "HashMapKey".into(),
            Self::IfType => "IfType".into(),
            Self::Location => "Location".into(),
            Self::Namespace => "Namespace".into(),
            Self::Nothing => "Nothing".into(),
            Self::Number => "Number".into(),
            Self::NumberEnum(values) => values
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("Or"),
            Self::Object => "Object".into(),
            Self::ScriptHandle => "ScriptHandle".into(),
            Self::Side => "Side".into(),
            Self::String => "String".into(),
            Self::StringEnum(values) => values.join("Or"),
            Self::StructuredText => "StructuredText".into(),
            Self::SwitchType => "SwitchType".into(),
            Self::Task => "Task".into(),
            Self::TeamMember => "TeamMember".into(),
            Self::Path => "Path".into(),
            Self::TurretPath => "TurretPath".into(),
            Self::UnitLoadoutArray => "UnitLoadoutArray".into(),
            Self::Position => "Position".into(),
            Self::Position2d => "Position2d".into(),
            Self::Position3d => "Position3d".into(),
            Self::Position3dASL => "Position3dASL".into(),
            Self::Position3DASLW => "Position3DASLW".into(),
            Self::Position3dATL => "Position3dATL".into(),
            Self::Position3dAGL => "Position3dAGL".into(),
            Self::Position3dAGLS => "Position3dAGLS".into(),
            Self::Position3dRelative => "Position3dRelative".into(),
            Self::Vector => "Vector".into(),
            Self::Vector2d => "Vector2d".into(),
            Self::Vector3d => "Vector3d".into(),
            Self::Waypoint => "Waypoint".into(),
            Self::WhileType => "WhileType".into(),
            Self::WithType => "WithType".into(),
            Self::Color => "Color".into(),
            Self::Unknown => "Unknown".into(),
            Self::ArrayOf(inner) => format!("ArrayOf{}", inner.wiki_name()),
            Self::OneOf(parts) => parts
                .iter()
                .map(Self::wiki_name)
                .collect::<Vec<_>>()
                .join("Or"),
        }
    }
}

impl std::fmt::Display for SqfType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArrayOf(inner) => write!(f, "Array of {inner}"),
            Self::OneOf(parts) => {
                let s = parts
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" | ");
                write!(f, "{s}")
            }
            other => write!(f, "{}", other.wiki_name()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub typ: SqfType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub optional: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// Arma 3 version string, e.g. `"2.12"`, when a `since` line precedes the param.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CallShape {
    Nular,
    Unary { right: String },
    Binary { left: String, right: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Syntax {
    pub call: CallShape,
    /// Raw wiki syntax line (unescaped).
    pub syntax_text: String,
    pub params: Vec<Param>,
    pub return_type: SqfType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_description: Option<String>,
    /// Per-syntax since version (bare `N.NN` line before the next Syntax header).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedCommand {
    pub name: String,
    /// Source markdown stem (after percent-decoding).
    pub source_file: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,
    pub syntaxes: Vec<Syntax>,
    /// First-line game-introduction version tokens.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intro_versions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseOutcome {
    Ok(ExtractedCommand),
    Stub { name: String, reason: String },
    Failed { name: String, reason: String },
    Skipped { name: String, reason: String },
}

impl ParseOutcome {
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Ok(cmd) => &cmd.name,
            Self::Stub { name, .. } | Self::Failed { name, .. } | Self::Skipped { name, .. } => {
                name
            }
        }
    }
}
