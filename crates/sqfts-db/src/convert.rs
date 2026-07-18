//! Convert arma3-wiki [`Value`] into SQFts [`Type`]s.

use arma3_wiki::model::Value;
use float_ord::FloatOrd;
use sqfts_syntax::{Brand, Primitive, Type};

/// Convert an arma3-wiki `Value` into a SQFts type.
#[must_use]
pub fn wiki_value_to_type(value: &Value) -> Type {
    match value {
        Value::Anything | Value::Unknown => Type::any(),
        Value::Nothing => Type::nothing(),
        Value::Number => Type::Primitive(Primitive::Number),
        Value::String => Type::Primitive(Primitive::String),
        Value::Boolean => Type::Primitive(Primitive::Boolean),
        Value::ArrayUnknown | Value::ArrayDate => Type::Primitive(Primitive::Array),
        Value::ArrayUnsized { typ, .. } => Type::ArrayOf(Box::new(wiki_value_to_type(typ))),
        Value::ArraySized { types, .. } => Type::Tuple(
            types
                .iter()
                .map(|e| (wiki_value_to_type(&e.value), false))
                .collect(),
        ),
        Value::ArrayColor | Value::ArrayColorRgb | Value::ArrayColorRgba => {
            Type::Brand(Brand::Color)
        }
        Value::Code => Type::Primitive(Primitive::Code),
        Value::Config => Type::Primitive(Primitive::Config),
        Value::Control => Type::Primitive(Primitive::Control),
        Value::DiaryRecord => Type::Primitive(Primitive::DiaryRecord),
        Value::Display => Type::Primitive(Primitive::Display),
        Value::EdenEntity => Type::Primitive(Primitive::EdenEntity),
        Value::EdenID => Type::Primitive(Primitive::EdenId),
        Value::ExceptionHandle => Type::Primitive(Primitive::Exception),
        Value::ForType => Type::Primitive(Primitive::ForType),
        Value::Group => Type::Primitive(Primitive::Group),
        Value::HashMapUnknown | Value::HashMapKnownKeys(_) => Type::Primitive(Primitive::HashMap),
        Value::HashMapKey => Type::Named("hashMapKey".into()),
        Value::IfType => Type::Primitive(Primitive::IfType),
        Value::Location => Type::Primitive(Primitive::Location),
        Value::Namespace => Type::Primitive(Primitive::Namespace),
        Value::Object => Type::Primitive(Primitive::Object),
        Value::ScriptHandle => Type::Primitive(Primitive::ScriptHandle),
        Value::Side => Type::Primitive(Primitive::Side),
        Value::StructuredText => Type::Primitive(Primitive::StructuredText),
        Value::SwitchType => Type::Primitive(Primitive::SwitchType),
        Value::Task => Type::Primitive(Primitive::Task),
        Value::TeamMember => Type::Primitive(Primitive::TeamMember),
        Value::TurretPath => Type::Brand(Brand::TurretPath),
        Value::UnitLoadoutArray => Type::Brand(Brand::UnitLoadout),
        Value::Position => Type::Brand(Brand::Position),
        Value::Position2d => Type::Brand(Brand::Position2D),
        Value::Position3d => Type::Brand(Brand::Position3D),
        Value::Position3dASL => Type::Brand(Brand::PositionASL),
        Value::Position3DASLW => Type::Brand(Brand::PositionASLW),
        Value::Position3dATL => Type::Brand(Brand::PositionATL),
        Value::Position3dAGL => Type::Brand(Brand::PositionAGL),
        Value::Position3dAGLS => Type::Brand(Brand::PositionAGLS),
        Value::Position3dRelative => Type::Brand(Brand::PositionRelative),
        Value::Vector3d => Type::Brand(Brand::Vector3D),
        Value::Waypoint => Type::Brand(Brand::Waypoint),
        Value::WhileType => Type::Primitive(Primitive::WhileType),
        Value::WithType => Type::Primitive(Primitive::WithType),
        Value::OneOf(parts) => Type::Union(
            parts
                .iter()
                .map(|(v, _)| wiki_value_to_type(v))
                .collect(),
        )
        .normalize(),
    }
}

/// Convert a PascalCase / wiki-style type name into a SQFts type.
#[must_use]
pub fn wiki_name_to_type(name: &str) -> Type {
    let n = name.trim();
    if n.is_empty() || n == "-" {
        return Type::any();
    }
    if let Some(inner) = n.strip_prefix("ArrayOf") {
        return Type::ArrayOf(Box::new(wiki_name_to_type(inner)));
    }
    if n.contains(" or ") {
        let parts: Vec<Type> = n
            .split(" or ")
            .map(|p| wiki_name_to_type(p.trim()))
            .collect();
        if parts.len() > 1 {
            return Type::Union(parts).normalize();
        }
    }
    if n.contains(" | ") {
        let parts: Vec<Type> = n
            .split(" | ")
            .map(|p| wiki_name_to_type(p.trim()))
            .collect();
        if parts.len() > 1 {
            return Type::Union(parts).normalize();
        }
    }
    if let Some(ty) = parse_wiki_type_atom(n) {
        return ty;
    }
    let lower = n.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("array of ") {
        let start = n.len() - rest.len();
        return Type::ArrayOf(Box::new(wiki_name_to_type(&n[start..])));
    }
    match n {
        "Anything" | "Unknown" | "any" => Type::any(),
        "Nothing" | "nothing" => Type::nothing(),
        "Number" | "Scalar" | "number" => Type::Primitive(Primitive::Number),
        "String" | "string" => Type::Primitive(Primitive::String),
        "Boolean" | "boolean" => Type::Primitive(Primitive::Boolean),
        "Array" | "array" => Type::Primitive(Primitive::Array),
        "Code" | "code" => Type::Primitive(Primitive::Code),
        "Object" | "object" => Type::Primitive(Primitive::Object),
        "Group" | "group" => Type::Primitive(Primitive::Group),
        "Side" | "side" => Type::Primitive(Primitive::Side),
        "Config" | "config" => Type::Primitive(Primitive::Config),
        "Control" | "control" => Type::Primitive(Primitive::Control),
        "Display" | "display" => Type::Primitive(Primitive::Display),
        "Task" | "task" => Type::Primitive(Primitive::Task),
        "Location" | "location" => Type::Primitive(Primitive::Location),
        "Namespace" | "namespace" => Type::Primitive(Primitive::Namespace),
        "HashMap" | "hashMap" => Type::Primitive(Primitive::HashMap),
        "TeamMember" | "Team Member" | "teamMember" => Type::Primitive(Primitive::TeamMember),
        "ScriptHandle" | "Script Handle" | "scriptHandle" => {
            Type::Primitive(Primitive::ScriptHandle)
        }
        "DiaryRecord" | "Diary Record" => Type::Primitive(Primitive::DiaryRecord),
        "StructuredText" | "Structured Text" | "structuredText" => {
            Type::Primitive(Primitive::StructuredText)
        }
        "ExceptionHandle" | "Exception Handle" => Type::Primitive(Primitive::Exception),
        "EdenEntity" | "Eden Entity" => Type::Primitive(Primitive::EdenEntity),
        "EdenID" | "Eden ID" => Type::Primitive(Primitive::EdenId),
        "Waypoint" => Type::Brand(Brand::Waypoint),
        "Color" => Type::Brand(Brand::Color),
        "Position" => Type::Brand(Brand::Position),
        "Position2d" | "Position2D" => Type::Brand(Brand::Position2D),
        "Position3d" | "Position3D" => Type::Brand(Brand::Position3D),
        "Position3dASL" | "PositionASL" => Type::Brand(Brand::PositionASL),
        "Position3DASLW" | "PositionASLW" => Type::Brand(Brand::PositionASLW),
        "Position3dATL" | "PositionATL" => Type::Brand(Brand::PositionATL),
        "Position3dAGL" | "PositionAGL" => Type::Brand(Brand::PositionAGL),
        "Position3dAGLS" | "PositionAGLS" => Type::Brand(Brand::PositionAGLS),
        "Position3dRelative" | "PositionRelative" => Type::Brand(Brand::PositionRelative),
        "Vector" | "Vector3d" | "Vector3D" => Type::Brand(Brand::Vector3D),
        "Vector2d" | "Vector2D" => Type::Brand(Brand::Vector2D),
        "TurretPath" => Type::Brand(Brand::TurretPath),
        "Path" => Type::Brand(Brand::TreePath),
        "UnitLoadoutArray" => Type::Brand(Brand::UnitLoadout),
        other => Type::Named(other.to_string()),
    }
}

fn parse_wiki_type_atom(n: &str) -> Option<Type> {
    let n = n.trim();
    if n.len() >= 2 {
        let bytes = n.as_bytes();
        let quote = bytes[0];
        if (quote == b'"' || quote == b'\'') && bytes[bytes.len() - 1] == quote {
            let inner = &n[1..n.len() - 1];
            let q = quote as char;
            let doubled = format!("{q}{q}");
            return Some(Type::StringLit(inner.replace(&doubled, &q.to_string())));
        }
    }
    if let Ok(v) = n.parse::<f32>() {
        return Some(Type::NumberLit(FloatOrd(v)));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wiki_name_parses_literal_unions() {
        assert_eq!(
            wiki_name_to_type("\"west\" | \"east\""),
            Type::Union(vec![
                Type::StringLit("west".into()),
                Type::StringLit("east".into()),
            ])
        );
        assert_eq!(
            wiki_name_to_type("0 | 1 | 6"),
            Type::Union(vec![
                Type::NumberLit(FloatOrd(0.0)),
                Type::NumberLit(FloatOrd(1.0)),
                Type::NumberLit(FloatOrd(6.0)),
            ])
        );
    }

    #[test]
    fn wiki_value_maps_primitives() {
        assert_eq!(
            wiki_value_to_type(&Value::Number),
            Type::Primitive(Primitive::Number)
        );
        assert_eq!(
            wiki_value_to_type(&Value::Object),
            Type::Primitive(Primitive::Object)
        );
        assert_eq!(wiki_value_to_type(&Value::Nothing), Type::nothing());
    }
}
