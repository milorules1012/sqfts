//! Strictness flags (SPEC §5.1).

use serde::Deserialize;

/// Checker flags — all default **off**.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CheckFlags {
    /// Error when a params/assignment yields `any` with no annotation.
    pub no_implicit_any: bool,
    /// `T | nothing` must be narrowed before use as `T`.
    pub strict_nil: bool,
    /// Collapse position brands to structural shapes.
    pub no_position_brands: bool,
    /// Unknown interface keys are errors.
    pub strict_hash_map: bool,
    /// Also check plain `.sqf` files (engine DB only).
    pub check_plain_sqf: bool,
}
