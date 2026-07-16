//! Normalize COMREF prose type strings onto [`crate::model::SqfType`].

use crate::model::SqfType;

/// Parse a type phrase from a parameter or return-value line.
///
/// Accepts idioms like `Object`, `Array of Strings`, `String or Code`,
/// `Array format PositionAGLS`, and comma/or lists.
#[must_use]
pub fn parse_type_phrase(raw: &str) -> SqfType {
    let cleaned = clean_type_text(raw);
    if cleaned.is_empty() || cleaned == "-" || cleaned == "\\" {
        return SqfType::Unknown;
    }
    if cleaned.eq_ignore_ascii_case("*return value needed*")
        || cleaned.eq_ignore_ascii_case("*syntax needed*")
    {
        return SqfType::Unknown;
    }

    if let Some(pos) = match_position_format(&cleaned) {
        return pos;
    }
    if let Some(vec) = match_vector_format(&cleaned) {
        return vec;
    }
    if let Some(color) = match_color(&cleaned) {
        return color;
    }

    // "Array of X" / "Array of Xs"
    if let Some(rest) = strip_array_of(&cleaned) {
        let inner = parse_type_phrase(rest);
        return SqfType::ArrayOf(Box::new(inner));
    }

    // Unions: prefer splitting on " or " at top level; also handle commas.
    if cleaned.contains(" or ") || cleaned.contains(',') {
        let parts = split_union(&cleaned);
        if parts.len() > 1 {
            let mut types: Vec<SqfType> = parts
                .into_iter()
                .map(|p| parse_type_phrase(p))
                .filter(|t| !matches!(t, SqfType::Unknown))
                .collect();
            types.dedup();
            if types.len() == 1 {
                return types.remove(0);
            }
            if !types.is_empty() {
                return SqfType::OneOf(types);
            }
        }
    }

    single_match(&cleaned).unwrap_or(SqfType::Unknown)
}

fn clean_type_text(raw: &str) -> String {
    let mut s = raw.trim().to_string();
    // Strip wrapping bold markdown
    if s.starts_with("**") && s.ends_with("**") && s.len() > 4 {
        s = s[2..s.len() - 2].to_string();
    }
    // HTML entities left over from scrape
    s = s
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    // Trailing punctuation noise
    s = s.trim_end_matches(['.', ';']).trim().to_string();
    // Collapse whitespace
    let mut out = String::new();
    let mut prev_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

fn strip_array_of(s: &str) -> Option<&str> {
    let lower = s.to_ascii_lowercase();
    for prefix in ["array of ", "arrays of "] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let start = s.len() - rest.len();
            let mut rest = &s[start..];
            // strip trailing plural 's' after a type word (Numbers → Number)
            if rest.ends_with('s') && !rest.eq_ignore_ascii_case("side") && !rest.contains(' ') {
                // keep as-is; single_match handles plural
            }
            // "Numbers" → hand to single match via cleaned rest
            if rest.ends_with('s') {
                let singular = &rest[..rest.len() - 1];
                if single_match(singular).is_some() {
                    rest = singular;
                }
            }
            return Some(rest.trim());
        }
    }
    None
}

fn split_union(s: &str) -> Vec<&str> {
    // Split on " or " first, then further on commas within each part.
    let mut parts = Vec::new();
    for or_part in s.split(" or ") {
        for comma_part in or_part.split(',') {
            let t = comma_part.trim();
            // Drop trailing clauses like "if failed"
            if t.is_empty() || t.starts_with("if ") {
                continue;
            }
            parts.push(t);
        }
    }
    parts
}

fn match_position_format(s: &str) -> Option<SqfType> {
    let lower = s.to_ascii_lowercase();
    // "Array format PositionAGLS" / "format PositionASL" / bare Position*
    let markers = [
        ("positionagls", SqfType::Position3dAGLS),
        ("positionagl", SqfType::Position3dAGL),
        ("positionaslw", SqfType::Position3DASLW),
        ("positionasl", SqfType::Position3dASL),
        ("positionatl", SqfType::Position3dATL),
        ("positionrelative", SqfType::Position3dRelative),
        ("position2d", SqfType::Position2d),
        ("position3d", SqfType::Position3d),
        ("position", SqfType::Position),
    ];
    for (key, typ) in &markers {
        if lower.contains(key)
            && (lower.contains("format")
                || lower.starts_with("position")
                || lower.contains("array"))
        {
            // Prefer more specific over bare "position"
            if *key == "position"
                && markers
                    .iter()
                    .any(|(k, _)| *k != "position" && lower.contains(*k))
            {
                continue;
            }
            return Some(typ.clone());
        }
        if lower == *key {
            return Some(typ.clone());
        }
    }
    None
}

fn match_vector_format(s: &str) -> Option<SqfType> {
    let lower = s.to_ascii_lowercase();
    if lower.contains("vector3d") || lower == "vector3d" {
        return Some(SqfType::Vector3d);
    }
    if lower.contains("vector2d") || lower == "vector2d" {
        return Some(SqfType::Vector2d);
    }
    if lower == "vector" || (lower.contains("format") && lower.contains("vector")) {
        return Some(SqfType::Vector);
    }
    None
}

fn match_color(s: &str) -> Option<SqfType> {
    let lower = s.to_ascii_lowercase();
    if lower.contains("color") {
        return Some(SqfType::Color);
    }
    None
}

fn single_match(value: &str) -> Option<SqfType> {
    let v = value
        .trim()
        .trim_matches(|c: char| c == '[' || c == ']' || c == '"' || c == '\'')
        .to_ascii_lowercase();
    // Strip trailing plural s for common types
    let candidates = [v.as_str(), v.strip_suffix('s').unwrap_or(v.as_str())];
    for cand in candidates {
        let mapped = match cand {
            "anything" | "any" => Some(SqfType::Anything),
            "array" | "arrays" => Some(SqfType::Array),
            "boolean" | "bool" => Some(SqfType::Boolean),
            "code" => Some(SqfType::Code),
            "config" => Some(SqfType::Config),
            "control" | "controls" => Some(SqfType::Control),
            "diary record" | "diaryrecord" => Some(SqfType::DiaryRecord),
            "display" | "displays" => Some(SqfType::Display),
            "eden entity" | "edenentity" => Some(SqfType::EdenEntity),
            "eden id" | "edenid" => Some(SqfType::EdenID),
            "exception" | "exception handle" | "exceptionhandle" | "exception handling" => {
                Some(SqfType::ExceptionHandle)
            }
            "for type" | "fortype" => Some(SqfType::ForType),
            "group" | "groups" => Some(SqfType::Group),
            "hashmap" | "hash map" => Some(SqfType::HashMap),
            "hashmapkey" | "hashmap key" | "hash map key" => Some(SqfType::HashMapKey),
            "if type" | "iftype" => Some(SqfType::IfType),
            "location" | "locations" => Some(SqfType::Location),
            "namespace" | "namespaces" => Some(SqfType::Namespace),
            "nothing" | "nil" => Some(SqfType::Nothing),
            "number" | "numbers" | "scalar" | "scalars" => Some(SqfType::Number),
            "object" | "objects" => Some(SqfType::Object),
            "script handle" | "scripthandle" | "script" => Some(SqfType::ScriptHandle),
            "side" => Some(SqfType::Side),
            "string" | "strings" => Some(SqfType::String),
            "structured text" | "structuredtext" => Some(SqfType::StructuredText),
            "switch type" | "switchtype" => Some(SqfType::SwitchType),
            "task" | "tasks" => Some(SqfType::Task),
            "team member" | "teammember" | "team members" => Some(SqfType::TeamMember),
            "path" | "tree view path" => Some(SqfType::Path),
            "turret path" | "turretpath" => Some(SqfType::TurretPath),
            "unit loadout array" | "unitloadoutarray" => Some(SqfType::UnitLoadoutArray),
            "waypoint" | "waypoints" => Some(SqfType::Waypoint),
            "while type" | "whiletype" => Some(SqfType::WhileType),
            "with type" | "withtype" => Some(SqfType::WithType),
            "color" => Some(SqfType::Color),
            "nan" | "unknown" => Some(SqfType::Unknown),
            _ => None,
        };
        if mapped.is_some() {
            return mapped;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_types() {
        assert_eq!(parse_type_phrase("Object"), SqfType::Object);
        assert_eq!(parse_type_phrase("Number"), SqfType::Number);
        assert_eq!(parse_type_phrase("Boolean"), SqfType::Boolean);
        assert_eq!(parse_type_phrase("Nothing"), SqfType::Nothing);
        assert_eq!(parse_type_phrase("Team Member"), SqfType::TeamMember);
        assert_eq!(parse_type_phrase("Script Handle"), SqfType::ScriptHandle);
        assert_eq!(parse_type_phrase("If Type"), SqfType::IfType);
    }

    #[test]
    fn arrays_and_unions() {
        assert_eq!(
            parse_type_phrase("Array of Strings"),
            SqfType::ArrayOf(Box::new(SqfType::String))
        );
        assert_eq!(
            parse_type_phrase("Array of Numbers"),
            SqfType::ArrayOf(Box::new(SqfType::Number))
        );
        assert_eq!(
            parse_type_phrase("String or Code"),
            SqfType::OneOf(vec![SqfType::String, SqfType::Code])
        );
        assert_eq!(
            parse_type_phrase("Object, Position2D or Position3D"),
            SqfType::OneOf(vec![
                SqfType::Object,
                SqfType::Position2d,
                SqfType::Position3d
            ])
        );
    }

    #[test]
    fn positions() {
        assert_eq!(
            parse_type_phrase("Array format PositionAGLS"),
            SqfType::Position3dAGLS
        );
        assert_eq!(
            parse_type_phrase("Array format PositionASL"),
            SqfType::Position3dASL
        );
        assert_eq!(parse_type_phrase("Position2D"), SqfType::Position2d);
        assert_eq!(parse_type_phrase("Position3D"), SqfType::Position3d);
    }
}
