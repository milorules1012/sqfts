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
        Type::Code { params, ret } => Type::Code {
            params: params
                .iter()
                .map(|p| sqfts_syntax::CodeParam {
                    name: p.name.clone(),
                    ty: normalize_brands(&p.ty, flags),
                    optional: p.optional,
                })
                .collect(),
            ret: Box::new(normalize_brands(ret, flags)),
        },
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
        // Gradual: opaque `code` ↔ parameterized `code(…) : R`
        (Type::Primitive(Primitive::Code), Type::Code { .. })
        | (Type::Code { .. }, Type::Primitive(Primitive::Code)) => true,
        // Parameterized code: contravariant params, covariant return
        (
            Type::Code {
                params: from_params,
                ret: from_ret,
            },
            Type::Code {
                params: to_params,
                ret: to_ret,
            },
        ) => {
            code_params_assignable(from_params, to_params, flags)
                && is_assignable_inner(from_ret, to_ret, flags)
        }
        _ => false,
    }
}

/// Can a value of type `code(from_params): _` be used where `code(to_params): _` is expected?
///
/// Params are contravariant: each expected param type must be assignable *to* the
/// implementation's corresponding param (the callee may accept a wider input).
fn code_params_assignable(
    from_params: &[sqfts_syntax::CodeParam],
    to_params: &[sqfts_syntax::CodeParam],
    flags: &CheckFlags,
) -> bool {
    // Implementation may require extra trailing params the expected signature omits — reject.
    if from_params.len() > to_params.len()
        && from_params[to_params.len()..].iter().any(|p| !p.optional)
    {
        return false;
    }
    for (i, to_p) in to_params.iter().enumerate() {
        if let Some(from_p) = from_params.get(i) {
            if !is_assignable_inner(&to_p.ty, &from_p.ty, flags) {
                return false;
            }
        } else if !to_p.optional {
            // Expected requires a param the implementation does not declare.
            return false;
        }
    }
    true
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

    fn code(params: Vec<(&str, Type, bool)>, ret: Type) -> Type {
        Type::Code {
            params: params
                .into_iter()
                .map(|(name, ty, optional)| sqfts_syntax::CodeParam {
                    name: name.into(),
                    ty,
                    optional,
                })
                .collect(),
            ret: Box::new(ret),
        }
    }

    #[test]
    fn opaque_code_assigns_both_ways_with_parameterized() {
        let flags = CheckFlags::default();
        let typed = code(
            vec![("unit", Type::Primitive(Primitive::Object), false)],
            Type::Primitive(Primitive::Boolean),
        );
        let opaque = Type::Primitive(Primitive::Code);
        assert!(is_assignable(&opaque, &typed, &flags));
        assert!(is_assignable(&typed, &opaque, &flags));
    }

    #[test]
    fn code_return_is_covariant() {
        let flags = CheckFlags::default();
        let returns_bool = code(vec![], Type::Primitive(Primitive::Boolean));
        let returns_any = code(vec![], Type::any());
        assert!(is_assignable(&returns_bool, &returns_any, &flags));
        assert!(is_assignable(&returns_any, &returns_bool, &flags)); // any is bidirectional
        let returns_number = code(vec![], Type::Primitive(Primitive::Number));
        assert!(!is_assignable(&returns_bool, &returns_number, &flags));
    }

    #[test]
    fn code_params_are_contravariant() {
        let flags = CheckFlags::default();
        // Impl accepts any; expected requires object — OK (object <: any into impl)
        let impl_any = code(
            vec![("x", Type::any(), false)],
            Type::Primitive(Primitive::Boolean),
        );
        let expect_object = code(
            vec![("x", Type::Primitive(Primitive::Object), false)],
            Type::Primitive(Primitive::Boolean),
        );
        assert!(is_assignable(&impl_any, &expect_object, &flags));
        // Impl accepts only object; expected supplies any — not OK without any on expected side...
        // any is bidirectional so object←any and any←object both work via any rule on the
        // param types themselves. Use number vs object instead:
        let impl_object = code(
            vec![("x", Type::Primitive(Primitive::Object), false)],
            Type::Primitive(Primitive::Boolean),
        );
        let expect_number = code(
            vec![("x", Type::Primitive(Primitive::Number), false)],
            Type::Primitive(Primitive::Boolean),
        );
        assert!(!is_assignable(&impl_object, &expect_number, &flags));
        // Wider expected input (number) into narrower impl (object) fails;
        // narrower expected (object) into wider impl (any) succeeds — already checked.
        assert!(!is_assignable(&impl_object, &expect_number, &flags));
    }

    #[test]
    fn parameterized_code_assigns_to_code_string_union() {
        let flags = CheckFlags::default();
        let typed = code(
            vec![("unit", Type::Primitive(Primitive::Object), false)],
            Type::Primitive(Primitive::Nothing),
        );
        let union = Type::Union(vec![
            Type::Primitive(Primitive::Code),
            Type::Primitive(Primitive::String),
        ]);
        assert!(is_assignable(&typed, &union, &flags));
        assert!(is_assignable(
            &Type::Primitive(Primitive::String),
            &union,
            &flags
        ));
        assert!(!is_assignable(
            &Type::Primitive(Primitive::Number),
            &union,
            &flags
        ));
    }
}
