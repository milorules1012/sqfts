//! Cross-check extracted signatures against arma3-wiki command YAML.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_yaml::Value as YamlValue;

use crate::emit::safe_filename;
use crate::model::{ExtractedCommand, SqfType};
use crate::types::parse_type_phrase;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMismatch {
    pub command: String,
    pub location: String,
    pub comref: String,
    pub wiki: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingInWiki {
    pub command: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enrichment {
    pub command: String,
    pub location: String,
    pub comref_type: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiffReport {
    pub compared: usize,
    pub missing_in_wiki: Vec<MissingInWiki>,
    pub missing_in_comref: Vec<String>,
    pub enrichments: Vec<Enrichment>,
    pub mismatches: Vec<TypeMismatch>,
    pub agreements: usize,
    pub wiki_source: String,
}

#[derive(Debug, Clone)]
struct WikiSyntax {
    return_type: SqfType,
    params: Vec<(String, SqfType)>,
}

impl Default for WikiSyntax {
    fn default() -> Self {
        Self {
            return_type: SqfType::Unknown,
            params: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct WikiCommand {
    name: String,
    syntaxes: Vec<WikiSyntax>,
}

/// Ensure a local checkout of arma3-wiki `dist` exists and return the commands dir.
pub fn ensure_wiki_commands_dir(cache_root: &Path) -> anyhow::Result<PathBuf> {
    let repo = cache_root.join("arma3-wiki");
    let commands = repo.join("commands");
    if commands.is_dir() {
        return Ok(commands);
    }

    fs::create_dir_all(cache_root)?;
    if repo.exists() {
        fs::remove_dir_all(&repo)?;
    }

    let status = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            "dist",
            "https://github.com/acemod/arma3-wiki.git",
        ])
        .arg(&repo)
        .status();

    match status {
        Ok(s) if s.success() && commands.is_dir() => Ok(commands),
        Ok(s) => anyhow::bail!("git clone arma3-wiki failed with status {s}"),
        Err(e) => anyhow::bail!("git not available to clone arma3-wiki: {e}"),
    }
}

fn load_wiki_index(commands_dir: &Path) -> anyhow::Result<HashMap<String, WikiCommand>> {
    let mut map = HashMap::new();
    for entry in fs::read_dir(commands_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        if let Some(cmd) = parse_wiki_yaml(&text) {
            map.insert(cmd.name.to_ascii_lowercase(), cmd);
        }
    }
    Ok(map)
}

fn parse_wiki_yaml(text: &str) -> Option<WikiCommand> {
    let root: YamlValue = serde_yaml::from_str(text).ok()?;
    let name = root.get("name")?.as_str()?.to_string();
    let mut syntaxes = Vec::new();
    if let Some(YamlValue::Sequence(syns)) = root.get("syntax") {
        for syn in syns {
            syntaxes.push(parse_wiki_syntax(syn));
        }
    }
    Some(WikiCommand { name, syntaxes })
}

fn parse_wiki_syntax(syn: &YamlValue) -> WikiSyntax {
    let return_type = parse_wiki_return(syn.get("ret"));
    let mut params = Vec::new();

    // Modern/left-right shape
    collect_param_node(syn.get("left"), &mut params);
    collect_param_node(syn.get("right"), &mut params);

    // Dist-branch flat `params:` list
    if let Some(YamlValue::Sequence(list)) = syn.get("params") {
        for item in list {
            if let Some(p) = parse_param_item(item) {
                params.push(p);
            }
        }
    }

    WikiSyntax {
        return_type,
        params,
    }
}

fn parse_wiki_return(ret: Option<&YamlValue>) -> SqfType {
    let Some(ret) = ret else {
        return SqfType::Unknown;
    };
    // ret: [Type, desc] or { type: Type, desc: ... } or bare Type
    match ret {
        YamlValue::Sequence(seq) => {
            let typ = seq.first().map(yaml_type_to_sqf).unwrap_or(SqfType::Unknown);
            typ
        }
        YamlValue::Mapping(map) => {
            let key = YamlValue::String("type".into());
            map.get(&key).map(yaml_type_to_sqf).unwrap_or(SqfType::Unknown)
        }
        other => yaml_type_to_sqf(other),
    }
}

fn collect_param_node(node: Option<&YamlValue>, out: &mut Vec<(String, SqfType)>) {
    let Some(node) = node else {
        return;
    };
    if let Some(p) = parse_param_item(node) {
        out.push(p);
        return;
    }
    match node {
        YamlValue::Sequence(items) => {
            for item in items {
                collect_param_node(Some(item), out);
            }
        }
        YamlValue::Mapping(map) => {
            // Externally tagged: Item: {...} / Array: [...] / Infinite: [...]
            for (k, v) in map {
                if let YamlValue::String(tag) = k {
                    if tag == "Item" {
                        if let Some(p) = parse_param_item(v) {
                            out.push(p);
                        }
                    } else if tag == "Array" || tag == "Infinite" {
                        collect_param_node(Some(v), out);
                    }
                }
            }
        }
        _ => {}
    }
}

fn parse_param_item(node: &YamlValue) -> Option<(String, SqfType)> {
    let map = node.as_mapping()?;
    let name_key = YamlValue::String("name".into());
    let type_key = YamlValue::String("type".into());
    let name = map.get(&name_key)?.as_str()?.to_string();
    let typ = map
        .get(&type_key)
        .map(yaml_type_to_sqf)
        .unwrap_or(SqfType::Unknown);
    Some((name, typ))
}

fn yaml_type_to_sqf(node: &YamlValue) -> SqfType {
    match node {
        YamlValue::String(s) => parse_type_phrase(s),
        YamlValue::Mapping(map) => {
            // Tagged enum forms: !Anything null, or { Anything: null }, or nested ArrayUnsized
            if let Some((k, v)) = map.iter().next() {
                if let YamlValue::String(tag) = k {
                    return match tag.as_str() {
                        "ArrayUnsized" | "ArrayOf" => {
                            let inner = v
                                .get("type")
                                .or_else(|| v.get("typ"))
                                .map(yaml_type_to_sqf)
                                .unwrap_or(SqfType::Unknown);
                            SqfType::ArrayOf(Box::new(inner))
                        }
                        "OneOf" => {
                            if let YamlValue::Sequence(parts) = v {
                                let types: Vec<SqfType> = parts
                                    .iter()
                                    .filter_map(|p| {
                                        p.get("type")
                                            .or_else(|| p.get("typ"))
                                            .map(yaml_type_to_sqf)
                                    })
                                    .collect();
                                if types.len() == 1 {
                                    types.into_iter().next().unwrap()
                                } else if types.is_empty() {
                                    SqfType::Unknown
                                } else {
                                    SqfType::OneOf(types)
                                }
                            } else {
                                SqfType::Unknown
                            }
                        }
                        "ArrayUnknown" | "ArrayEmpty" | "ArraySized" | "ArrayDate"
                        | "ArrayEdenEntities" => SqfType::Array,
                        "ArrayColor" | "ArrayColorRgb" | "ArrayColorRgba" => SqfType::Color,
                        "HashMapUnknown" | "HashMapKnownKeys" => SqfType::HashMap,
                        "NumberRange" => SqfType::Number,
                        "NumberEnum" => parse_yaml_number_enum(v).unwrap_or(SqfType::Number),
                        "StringEnum" => parse_yaml_string_enum(v).unwrap_or(SqfType::String),
                        other => parse_type_phrase(other),
                    };
                }
            }
            SqfType::Unknown
        }
        YamlValue::Tagged(tagged) => {
            let tag = tagged.tag.to_string();
            let tag = tag.trim_start_matches('!');
            match tag {
                "ArrayUnsized" => {
                    let inner = tagged
                        .value
                        .get("type")
                        .or_else(|| tagged.value.get("typ"))
                        .map(yaml_type_to_sqf)
                        .unwrap_or(SqfType::Unknown);
                    SqfType::ArrayOf(Box::new(inner))
                }
                "ArrayUnknown" | "ArrayEmpty" => SqfType::Array,
                "ArrayColor" | "ArrayColorRgb" | "ArrayColorRgba" => SqfType::Color,
                "HashMapUnknown" => SqfType::HashMap,
                other => parse_type_phrase(other),
            }
        }
        _ => SqfType::Unknown,
    }
}

/// Compare extracted engine commands against a local arma3-wiki commands directory.
pub fn diff_against_wiki_dir(commands: &[ExtractedCommand], wiki_commands_dir: &Path) -> DiffReport {
    let wiki = match load_wiki_index(wiki_commands_dir) {
        Ok(w) => w,
        Err(e) => {
            let mut report = DiffReport {
                wiki_source: format!("error loading {}: {e}", wiki_commands_dir.display()),
                ..DiffReport::default()
            };
            report.missing_in_wiki.push(MissingInWiki {
                command: "*".into(),
                note: format!("failed to load wiki index: {e}"),
            });
            return report;
        }
    };

    let mut report = DiffReport {
        wiki_source: wiki_commands_dir.display().to_string(),
        ..DiffReport::default()
    };
    let mut comref_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    for cmd in commands {
        comref_names.insert(cmd.name.to_ascii_lowercase());
        let Some(wiki_cmd) = wiki.get(&cmd.name.to_ascii_lowercase()) else {
            report.missing_in_wiki.push(MissingInWiki {
                command: cmd.name.clone(),
                note: "present in COMREF extraction, absent from arma3-wiki dist".into(),
            });
            continue;
        };
        report.compared += 1;

        let pair_count = cmd.syntaxes.len().max(wiki_cmd.syntaxes.len());
        let mut cmd_agreed = true;

        for i in 0..pair_count {
            let location = format!("syntax[{i}]");
            match (cmd.syntaxes.get(i), wiki_cmd.syntaxes.get(i)) {
                (Some(ours), Some(theirs)) => {
                    if !types_compatible(&ours.return_type, &theirs.return_type) {
                        if theirs.return_type.is_unknown() && !ours.return_type.is_unknown() {
                            report.enrichments.push(Enrichment {
                                command: cmd.name.clone(),
                                location: format!("{location}.return"),
                                comref_type: ours.return_type.to_string(),
                                note: "wiki return is Unknown; COMREF has concrete type".into(),
                            });
                        } else if !ours.return_type.is_unknown() {
                            cmd_agreed = false;
                            report.mismatches.push(TypeMismatch {
                                command: cmd.name.clone(),
                                location: format!("{location}.return"),
                                comref: ours.return_type.to_string(),
                                wiki: theirs.return_type.to_string(),
                            });
                        }
                    }

                    let wiki_params: HashMap<String, SqfType> = theirs
                        .params
                        .iter()
                        .map(|(n, t)| (n.to_ascii_lowercase(), t.clone()))
                        .collect();

                    for p in &ours.params {
                        if p.typ.is_unknown() {
                            continue;
                        }
                        if let Some(wiki_ty) = wiki_params.get(&p.name.to_ascii_lowercase()) {
                            if wiki_ty.is_unknown() {
                                report.enrichments.push(Enrichment {
                                    command: cmd.name.clone(),
                                    location: format!("{location}.param.{}", p.name),
                                    comref_type: p.typ.to_string(),
                                    note: "wiki param is Unknown; COMREF has concrete type".into(),
                                });
                            } else if !types_compatible(&p.typ, wiki_ty) {
                                cmd_agreed = false;
                                report.mismatches.push(TypeMismatch {
                                    command: cmd.name.clone(),
                                    location: format!("{location}.param.{}", p.name),
                                    comref: p.typ.to_string(),
                                    wiki: wiki_ty.to_string(),
                                });
                            }
                        }
                    }
                }
                (Some(_), None) => {
                    report.enrichments.push(Enrichment {
                        command: cmd.name.clone(),
                        location: location.clone(),
                        comref_type: "(overload)".into(),
                        note: "COMREF has an extra syntax overload not present in wiki".into(),
                    });
                }
                (None, Some(_)) => {
                    report.mismatches.push(TypeMismatch {
                        command: cmd.name.clone(),
                        location,
                        comref: "(missing overload)".into(),
                        wiki: "(present)".into(),
                    });
                    cmd_agreed = false;
                }
                (None, None) => {}
            }
        }

        if cmd_agreed {
            report.agreements += 1;
        }
    }

    for name in wiki.keys() {
        if !comref_names.contains(name) {
            report.missing_in_comref.push(name.clone());
        }
    }
    report.missing_in_comref.sort();
    report
}

/// Compare against arma3-wiki, cloning `dist` into `cache_root` if needed.
pub fn diff_against_wiki(commands: &[ExtractedCommand], cache_root: &Path) -> anyhow::Result<DiffReport> {
    let dir = ensure_wiki_commands_dir(cache_root)?;
    Ok(diff_against_wiki_dir(commands, &dir))
}

fn types_compatible(a: &SqfType, b: &SqfType) -> bool {
    if a.is_unknown() || b.is_unknown() {
        return true;
    }
    if a == b {
        return true;
    }
    match (a, b) {
        (SqfType::Array, SqfType::ArrayOf(_)) | (SqfType::ArrayOf(_), SqfType::Array) => true,
        (SqfType::Color, SqfType::Array) | (SqfType::Array, SqfType::Color) => true,
        (SqfType::OneOf(parts), other) | (other, SqfType::OneOf(parts)) => {
            parts.iter().any(|p| types_compatible(p, other))
        }
        (SqfType::NumberEnum(_), SqfType::Number) | (SqfType::Number, SqfType::NumberEnum(_)) => {
            true
        }
        (SqfType::StringEnum(_), SqfType::String) | (SqfType::String, SqfType::StringEnum(_)) => {
            true
        }
        (l, r) if is_positiony(l) && is_positiony(r) => true,
        (l, SqfType::Array) | (SqfType::Array, l) if is_positiony(l) => true,
        _ => false,
    }
}

fn is_positiony(t: &SqfType) -> bool {
    matches!(
        t,
        SqfType::Position
            | SqfType::Position2d
            | SqfType::Position3d
            | SqfType::Position3dASL
            | SqfType::Position3DASLW
            | SqfType::Position3dATL
            | SqfType::Position3dAGL
            | SqfType::Position3dAGLS
            | SqfType::Position3dRelative
            | SqfType::Vector
            | SqfType::Vector2d
            | SqfType::Vector3d
    )
}

/// Write patch YAML files + a summary under `out_dir/patches/`.
pub fn emit_patches(report: &DiffReport, out_dir: &Path) -> anyhow::Result<()> {
    let patches = out_dir.join("patches");
    fs::create_dir_all(&patches)?;

    fs::write(patches.join("summary.yml"), serde_yaml::to_string(report)?)?;

    for (i, e) in report.enrichments.iter().enumerate() {
        let name = format!("{:04}_{}.yml", i, safe_filename(&e.command));
        fs::write(patches.join(name), serde_yaml::to_string(e)?)?;
    }

    Ok(())
}

fn parse_yaml_number_enum(v: &YamlValue) -> Option<SqfType> {
    let YamlValue::Sequence(seq) = v else {
        return None;
    };
    let mut nums = Vec::new();
    for item in seq {
        if let Some(n) = item.get("value").and_then(YamlValue::as_i64) {
            nums.push(n as i32);
        } else if let Some(n) = item.as_i64() {
            nums.push(n as i32);
        }
    }
    if nums.is_empty() {
        None
    } else {
        Some(SqfType::NumberEnum(nums))
    }
}

fn parse_yaml_string_enum(v: &YamlValue) -> Option<SqfType> {
    let YamlValue::Sequence(seq) = v else {
        return None;
    };
    let mut values = Vec::new();
    for item in seq {
        if let Some(s) = item.get("value").and_then(YamlValue::as_str) {
            values.push(s.to_string());
        } else if let Some(s) = item.as_str() {
            values.push(s.to_string());
        }
    }
    if values.is_empty() {
        None
    } else {
        Some(SqfType::StringEnum(values))
    }
}
