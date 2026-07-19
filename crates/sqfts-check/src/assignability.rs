//! Assignability lattice (SPEC §1).

use sqfts_syntax::{Brand, Primitive, Type};

use crate::config::CheckFlags;

/// Is `from` assignable to `to`?
#[must_use]
pub fn is_assignable(from: &Type, to: &Type, flags: &CheckFlags) -> bool {
    let from = normalize_brands(from, flags);
    let to = normalize_brands(to, flags);
    is_assignable_inner(&from, &to, flags)
}

fn normalize_brands(ty: &Type, flags: &CheckFlags) -> Type {
    if !flags.no_position_brands {
        return ty.clone();
    }
    match ty {
        Type::Brand(b) => brand_structural(*b),
        Type::ArrayOf(inner) => Type::ArrayOf(Box::new(normalize_brands(inner, flags))),
        Type::Tuple(elems) => Type::Tuple(
            elems
                .iter()
                .map(|(t, o)| (normalize_brands(t, flags), *o))
                .collect(),
        ),
        Type::Union(parts) => {
            Type::Union(parts.iter().map(|p| normalize_brands(p, flags)).collect())
        }
        other => other.clone(),
    }
}

fn brand_structural(b: Brand) -> Type {
    match b {
        Brand::Position2D | Brand::Vector2D => Type::Tuple(vec![
            (Type::Primitive(Primitive::Number), false),
            (Type::Primitive(Primitive::Number), false),
        ]),
        Brand::Position3D
        | Brand::PositionATL
        | Brand::PositionASL
        | Brand::PositionASLW
        | Brand::PositionAGL
        | Brand::PositionRelative
        | Brand::Vector3D => Type::Tuple(vec![
            (Type::Primitive(Primitive::Number), false),
            (Type::Primitive(Primitive::Number), false),
            (Type::Primitive(Primitive::Number), false),
        ]),
        Brand::PositionAGLS => Type::Tuple(vec![
            (Type::Primitive(Primitive::Number), false),
            (Type::Primitive(Primitive::Number), false),
            (Type::Primitive(Primitive::Number), true),
        ]),
        Brand::Position => Type::Primitive(Primitive::Array),
        Brand::Waypoint => Type::Tuple(vec![
            (Type::Primitive(Primitive::Group), false),
            (Type::Primitive(Primitive::Number), false),
        ]),
        Brand::Color => Type::Tuple(vec![
            (Type::Primitive(Primitive::Number), false),
            (Type::Primitive(Primitive::Number), false),
            (Type::Primitive(Primitive::Number), false),
            (Type::Primitive(Primitive::Number), true),
        ]),
        Brand::TurretPath | Brand::TreePath => {
            Type::ArrayOf(Box::new(Type::Primitive(Primitive::Number)))
        }
        Brand::UnitLoadout => Type::Primitive(Primitive::Array),
    }
}

/// Homogeneous element type of `T[]`, or of brands that are structurally `T[]`
/// (e.g. `turretPath` / `treePath` → `number`).
#[must_use]
pub fn array_element_ty(ty: &Type) -> Option<Type> {
    match ty {
        Type::ArrayOf(inner) => Some((**inner).clone()),
        Type::Brand(b) => match brand_structural(*b) {
            Type::ArrayOf(inner) => Some(*inner),
            _ => None,
        },
        _ => None,
    }
}

fn is_assignable_inner(from: &Type, to: &Type, flags: &CheckFlags) -> bool {
    if from == to {
        return true;
    }
    // any is bidirectional
    if matches!(from, Type::Primitive(Primitive::Any))
        || matches!(to, Type::Primitive(Primitive::Any))
    {
        return true;
    }
    // Named aliases treated as opaque unless equal (resolved by caller)
    match (from, to) {
        (Type::Union(parts), _) => parts.iter().all(|p| is_assignable_inner(p, to, flags)),
        (_, Type::Union(parts)) => parts.iter().any(|p| is_assignable_inner(from, p, flags)),
        (Type::ArrayOf(a), Type::ArrayOf(b)) => is_assignable_inner(a, b, flags),
        (Type::ArrayOf(_), Type::Primitive(Primitive::Array)) => true,
        (Type::Tuple(_), Type::Primitive(Primitive::Array)) => true,
        (Type::Tuple(elems), Type::ArrayOf(inner)) => elems
            .iter()
            .all(|(t, _)| is_assignable_inner(t, inner, flags)),
        (Type::Tuple(a), Type::Tuple(b)) => {
            if a.len() < b.iter().filter(|(_, o)| !*o).count() {
                return false;
            }
            for (i, (bt, bopt)) in b.iter().enumerate() {
                if let Some((at, _)) = a.get(i) {
                    if !is_assignable_inner(at, bt, flags) {
                        return false;
                    }
                } else if !bopt {
                    return false;
                }
            }
            true
        }
        // Brands
        (Type::Brand(a), Type::Brand(b)) => brands_assignable(*a, *b),
        (Type::Brand(a), other) => is_assignable_inner(&brand_structural(*a), other, flags),
        // Fresh number tuple → brand (literal assignability): handled when from is Tuple
        (Type::Tuple(elems), Type::Brand(b)) => {
            is_assignable_inner(&Type::Tuple(elems.clone()), &brand_structural(*b), flags)
        }
        (Type::Primitive(Primitive::Array), Type::Brand(_)) => false,
        // Interfaces / named: assignable to hashMap
        (Type::Named(_), Type::Primitive(Primitive::HashMap)) => true,
        (Type::Primitive(Primitive::HashMap), Type::Named(_)) => false, // needs cast
        (Type::StringLit(a), Type::StringLit(b)) => a.eq_ignore_ascii_case(b),
        (Type::StringLit(_), Type::Primitive(Primitive::String)) => true,
        (Type::NumberLit(_), Type::Primitive(Primitive::Number)) => true,
        _ => false,
    }
}

fn brands_assignable(from: Brand, to: Brand) -> bool {
    if from == to {
        return true;
    }
    // Specific → bare position3D / position
    match to {
        Brand::Position3D => matches!(
            from,
            Brand::PositionATL
                | Brand::PositionASL
                | Brand::PositionASLW
                | Brand::PositionAGL
                | Brand::PositionAGLS
                | Brand::PositionRelative
                | Brand::Position3D
        ),
        Brand::Position => matches!(
            from,
            Brand::Position
                | Brand::Position2D
                | Brand::Position3D
                | Brand::PositionATL
                | Brand::PositionASL
                | Brand::PositionASLW
                | Brand::PositionAGL
                | Brand::PositionAGLS
                | Brand::PositionRelative
        ),
        _ => false,
    }
}

/// Do two types overlap enough for an `as` cast?
#[must_use]
#[allow(dead_code)]
pub fn types_overlap(a: &Type, b: &Type, flags: &CheckFlags) -> bool {
    is_assignable(a, b, flags) || is_assignable(b, a, flags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemtt_sqf::Scalar;
    use sqfts_syntax::{Primitive, Type};

    #[test]
    fn string_lit_assigns_to_string() {
        let flags = CheckFlags::default();
        assert!(is_assignable(
            &Type::StringLit("west".into()),
            &Type::Primitive(Primitive::String),
            &flags
        ));
        assert!(!is_assignable(
            &Type::Primitive(Primitive::String),
            &Type::StringLit("west".into()),
            &flags
        ));
    }

    #[test]
    fn string_lits_match_case_insensitively() {
        let flags = CheckFlags::default();
        assert!(is_assignable(
            &Type::StringLit("WEST".into()),
            &Type::StringLit("west".into()),
            &flags
        ));
    }

    #[test]
    fn number_lit_assigns_to_number() {
        let flags = CheckFlags::default();
        assert!(is_assignable(
            &Type::NumberLit(Scalar(1.0)),
            &Type::Primitive(Primitive::Number),
            &flags
        ));
    }

    #[test]
    fn literal_in_union() {
        let flags = CheckFlags::default();
        let union = Type::Union(vec![
            Type::NumberLit(Scalar(0.0)),
            Type::NumberLit(Scalar(1.0)),
        ]);
        assert!(is_assignable(&Type::NumberLit(Scalar(1.0)), &union, &flags));
        assert!(!is_assignable(
            &Type::NumberLit(Scalar(2.0)),
            &union,
            &flags
        ));
    }
}
