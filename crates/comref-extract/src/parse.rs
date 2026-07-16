//! Parse a single COMREF-md wiki-export markdown file into [`ExtractedCommand`].

use regex::Regex;
use std::sync::OnceLock;

use crate::model::{CallShape, ExtractedCommand, Param, ParseOutcome, SqfType, Syntax};
use crate::types::parse_type_phrase;

/// Parse the full markdown content of one COMREF page.
pub fn parse_comref_page(source_stem: &str, content: &str) -> ParseOutcome {
    let name = command_name_from_stem(source_stem);
    if content.contains("*Syntax needed*") {
        return ParseOutcome::Stub {
            name,
            reason: "syntax needed stub".into(),
        };
    }

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return ParseOutcome::Failed {
            name,
            reason: "empty file".into(),
        };
    }

    let intro_versions = parse_intro_versions(lines[0]);
    let description = extract_labeled_block(&lines, "Description:");
    let aliases = extract_labeled_block(&lines, "Alias:")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let groups = split_groups(&extract_labeled_block(&lines, "Groups:"));

    let syntax_blocks = match collect_syntax_blocks(&lines) {
        Ok(blocks) if !blocks.is_empty() => blocks,
        Ok(_) => {
            return ParseOutcome::Failed {
                name,
                reason: "no syntax blocks found".into(),
            };
        }
        Err(e) => {
            return ParseOutcome::Failed {
                name: name.clone(),
                reason: e,
            };
        }
    };

    let mut syntaxes = Vec::new();
    for block in syntax_blocks {
        match parse_syntax_block(&name, &block) {
            Ok(syn) => syntaxes.push(syn),
            Err(e) => {
                return ParseOutcome::Failed {
                    name,
                    reason: format!("syntax parse error: {e}"),
                };
            }
        }
    }

    if syntaxes.is_empty() {
        return ParseOutcome::Failed {
            name,
            reason: "no parsed syntaxes".into(),
        };
    }

    // Require at least one non-unknown return type OR at least one typed param
    // (some nullary commands only have a return).
    let has_useful_types = syntaxes.iter().any(|s| {
        !s.return_type.is_unknown()
            || s.params.iter().any(|p| !p.typ.is_unknown())
    });
    if !has_useful_types {
        return ParseOutcome::Stub {
            name,
            reason: "no usable types in any syntax".into(),
        };
    }

    ParseOutcome::Ok(ExtractedCommand {
        name,
        source_file: source_stem.to_string(),
        description,
        aliases,
        groups,
        syntaxes,
        intro_versions,
    })
}

/// Map a filename stem onto the canonical command name (mirrors arma3-wiki).
#[must_use]
pub fn command_name_from_stem(stem: &str) -> String {
    // Match encoded forms first (as stored on disk), then decoded variants.
    match stem {
        "! a" | "!_a" | "%21_a" => return "!".into(),
        "+" | "%2B" => return "+".into(),
        "-" => return "-".into(),
        "a == b" | "a_%3D%3D_b" => return "==".into(),
        "a != b" | "a_!%3D_b" => return "!=".into(),
        "a = b" | "a_%3D_b" => return "=".into(),
        "a ^ b" | "a_%5E_b" => return "^".into(),
        "a % b" | "a_%25_b" => return "%".into(),
        "a && b" | "a_%26%26_b" => return "&&".into(),
        "a * b" | "a_*_b" | "a_%2A_b" => return "*".into(),
        "a / b" | "a_/_b" | "a_%2F_b" => return "/".into(),
        "a : b" | "a_:_b" | "a_%3A_b" => return ":".into(),
        "a >= b" | "a_greater%3D_b" => return ">=".into(),
        "a > b" | "a_greater_b" => return ">".into(),
        "a # b" | "a_hash_b" => return "#".into(),
        "a <= b" | "a_less%3D_b" => return "<=".into(),
        "a < b" | "a_less_b" => return "<".into(),
        "a || b" | "a_or_b" => return "||".into(),
        "config >> name" | "config_greater_greater_name" => return ">>".into(),
        _ => {}
    }

    let decoded = percent_decode(stem);
    match decoded.as_str() {
        "! a" | "!_a" => "!".into(),
        "a && b" | "a_&&_b" => "&&".into(),
        "a || b" | "a_||_b" => "||".into(),
        "a != b" | "a_!=_b" => "!=".into(),
        "a == b" | "a_==_b" => "==".into(),
        "a >= b" | "a_>=_b" => ">=".into(),
        "a <= b" | "a_<=_b" => "<=".into(),
        "a > b" | "a_>_b" => ">".into(),
        "a < b" | "a_<_b" => "<".into(),
        "a % b" | "a_%_b" => "%".into(),
        "a ^ b" | "a_^_b" => "^".into(),
        "a = b" | "a_=_b" => "=".into(),
        "a # b" | "a_#_b" => "#".into(),
        "a * b" | "a_*_b" => "*".into(),
        "a / b" | "a_/_b" => "/".into(),
        "a : b" | "a_:_b" => ":".into(),
        "config >> name" | "config_>>_name" => ">>".into(),
        other => other.to_string(),
    }
}

fn percent_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .into_owned()
}

fn parse_intro_versions(first_line: &str) -> Vec<String> {
    let line = first_line
        .trim()
        .trim_start_matches(|c: char| !c.is_ascii_digit() && c != '.');
    line.split_whitespace()
        .filter(|tok| version_re().is_match(tok))
        .map(ToOwned::to_owned)
        .collect()
}

fn version_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^\d+\.\d+$").expect("regex"))
}

fn extract_labeled_block(lines: &[&str], label: &str) -> String {
    let mut collecting = false;
    let mut buf = String::new();
    for &line in lines {
        let trimmed = line.trim_end();
        if trimmed.starts_with("### ") {
            if collecting {
                break;
            }
            continue;
        }
        // Label sits alone on a line (often with trailing spaces in the scrape).
        if trimmed.trim() == label {
            collecting = true;
            continue;
        }
        if collecting {
            // Stop at next label-like line ending with ':'
            let t = trimmed.trim();
            if !t.is_empty()
                && t.ends_with(':')
                && !t.contains(' ')
                && t.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            {
                break;
            }
            if matches!(
                t,
                "Alias:"
                    | "Multiplayer:"
                    | "Problems:"
                    | "Execution:"
                    | "Groups:"
                    | "Syntax:"
                    | "Parameters:"
                    | "Return Value:"
                    | "See also:"
            ) {
                break;
            }
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(t);
        }
    }
    buf.trim().to_string()
}

fn split_groups(raw: &str) -> Vec<String> {
    // Scraped Groups line concatenates without separators. Split using a known dictionary.
    if raw.is_empty() {
        return Vec::new();
    }
    let known = known_groups();
    let mut remaining = raw.to_string();
    let mut found = Vec::new();
    // Greedy longest-match from the start, repeatedly.
    while !remaining.is_empty() {
        let mut matched: Option<&str> = None;
        for g in known {
            if remaining.starts_with(g) {
                if matched.map_or(true, |m| g.len() > m.len()) {
                    matched = Some(g);
                }
            }
        }
        if let Some(g) = matched {
            found.push(g.to_string());
            remaining = remaining[g.len()..].to_string();
        } else {
            // Fallback: keep the rest as one token
            found.push(remaining);
            break;
        }
    }
    found
}

fn known_groups() -> &'static [&'static str] {
    &[
        "Object Manipulation",
        "Mission Information",
        "Program Flow",
        "Unit Control",
        "Math",
        "Variables",
        "Strings",
        "Arrays",
        "HashMap",
        "Positions",
        "GUI Control",
        "System",
        "Time",
        "Environment",
        "Triggers",
        "Waypoints",
        "Weapons",
        "Vehicle Inventory",
        "Vehicle",
        "Interaction",
        "Multiplayer",
        "Diagnostic",
        "Eden Editor",
        "Map",
        "Markers",
        "Camera Control",
        "Rendered Geometry",
        "Event Handlers",
        "Configs",
        "Groups",
        "Sides",
        "Namespaces",
        "Structured Text",
        "Leaderboards",
        "Broken Commands",
        "Uncategorised",
        "Stamina System",
        "Sensors",
        "Ropes and Sling Loading",
        "RTD",
        "Particle Systems",
        "Locations",
        "Kaipo",
        "Inventory",
        "Garbage Collection",
        "Flags",
        "Dynamic Simulation",
        "Designers",
        "Curator",
        "Conversations",
        "Custom Features",
        "Briefing",
        "Artillery",
        "Animation",
        "AI",
        "Teams",
        "Object Detection",
        "Program Flow",
    ]
}

#[derive(Debug)]
struct SyntaxBlock {
    syntax_text: String,
    /// Raw text between Parameters: and Return Value: (may be empty for nullary).
    params_text: String,
    return_text: String,
    since: Option<String>,
}

fn collect_syntax_blocks(lines: &[&str]) -> Result<Vec<SyntaxBlock>, String> {
    // Find ### Syntax / ### Alternative Syntax / ### Syntax N headers
    let mut headers: Vec<(usize, &str)> = Vec::new();
    for (i, &line) in lines.iter().enumerate() {
        let t = line.trim();
        if t == "### Syntax"
            || t == "### Alternative Syntax"
            || t.starts_with("### Syntax ")
        {
            headers.push((i, t));
        }
        if t == "### Examples" || t == "### Additional Information" || t == "### Notes" {
            break;
        }
    }
    if headers.is_empty() {
        return Err("no ### Syntax headers".into());
    }

    let mut blocks = Vec::new();
    for (idx, &(start, _)) in headers.iter().enumerate() {
        let end = headers
            .get(idx + 1)
            .map(|(i, _)| *i)
            .or_else(|| {
                lines.iter().enumerate().find_map(|(i, l)| {
                    let t = l.trim();
                    if i > start
                        && matches!(
                            t,
                            "### Examples" | "### Additional Information" | "### Notes"
                        )
                    {
                        Some(i)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(lines.len());

        let slice = &lines[start..end];
        let syntax_text = take_after_label(slice, "Syntax:")
            .ok_or_else(|| "missing Syntax: label".to_string())?;
        let params_text = take_after_label(slice, "Parameters:").unwrap_or_default();
        let return_text = take_after_label(slice, "Return Value:")
            .ok_or_else(|| "missing Return Value: label".to_string())?;

        // Bare version number on its own line after Return Value (syntactic since)
        let since = slice
            .iter()
            .rev()
            .find_map(|l| {
                let t = l.trim();
                if version_re().is_match(t) {
                    Some(t.to_string())
                } else {
                    None
                }
            });

        if syntax_text.contains("*Syntax needed*") {
            return Err("stub syntax".into());
        }
        let syntax_text = unescape_syntax(&syntax_text);
        if syntax_text == "-"
            || syntax_text.is_empty()
            || syntax_text.to_ascii_lowercase().starts_with("see ")
        {
            return Err(format!("alias/redirect syntax: {syntax_text}"));
        }

        blocks.push(SyntaxBlock {
            syntax_text,
            params_text,
            return_text,
            since,
        });
    }
    Ok(blocks)
}

fn take_after_label(lines: &[&str], label: &str) -> Option<String> {
    let mut iter = lines.iter();
    while let Some(&line) = iter.next() {
        if line.trim() == label {
            let mut buf = String::new();
            for &next in iter.by_ref() {
                let t = next.trim();
                // Stop at next section label or ### header
                if t.starts_with("### ")
                    || matches!(
                        t,
                        "Parameters:"
                            | "Return Value:"
                            | "Syntax:"
                            | "Example 1:"
                            | "See also:"
                            | "Groups:"
                            | "Alias:"
                            | "Multiplayer:"
                            | "Problems:"
                    )
                    || version_re().is_match(t)
                {
                    break;
                }
                if !buf.is_empty() {
                    buf.push('\n');
                }
                buf.push_str(t);
            }
            return Some(buf.trim().to_string());
        }
    }
    None
}

fn unescape_syntax(s: &str) -> String {
    let mut s = s
        .replace("\\[", "[")
        .replace("\\]", "]")
        .replace("\\>", ">")
        .replace("\\<", "<")
        .replace("\\*", "*")
        .replace("\\-", "-")
        .replace("\\|", "|");
    // Strip wrapping bold: **waves**
    if s.starts_with("**") && s.ends_with("**") && s.len() > 4 {
        s = s[2..s.len() - 2].to_string();
    }
    // Strip residual HTML
    s = strip_html_tags(&s);
    s.trim().to_string()
}

fn strip_html_tags(s: &str) -> String {
    let re = html_tag_re();
    re.replace_all(s, "").into_owned()
}

fn html_tag_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"<[^>]+>").expect("regex"))
}

fn parse_syntax_block(command: &str, block: &SyntaxBlock) -> Result<Syntax, String> {
    let call = infer_call_shape(command, &block.syntax_text)?;
    let params = parse_params(&block.params_text)?;
    let (return_type, return_description) = parse_return(&block.return_text);

    Ok(Syntax {
        call,
        syntax_text: block.syntax_text.clone(),
        params,
        return_type,
        return_description,
        since: block.since.clone(),
    })
}

fn infer_call_shape(command: &str, syntax_text: &str) -> Result<CallShape, String> {
    let text = syntax_text.trim();
    // Nullary: syntax is just the command name (possibly bold already stripped)
    if text.eq_ignore_ascii_case(command)
        || text.replace(' ', "").eq_ignore_ascii_case(&command.replace(' ', ""))
    {
        return Ok(CallShape::Nular);
    }

    // Find the command token as a whole word in the syntax line.
    // Prefer case-sensitive match, then case-insensitive.
    let cmd_pos = find_command_token(text, command)
        .ok_or_else(|| format!("command `{command}` not found in syntax `{text}`"))?;

    let left = text[..cmd_pos].trim();
    let right = text[cmd_pos + command.len()..].trim();
    // Also try with the original casing from the syntax
    let right = if right.is_empty() {
        // command may appear with different casing — recompute using matched length
        let matched_len = command.len();
        text[cmd_pos + matched_len..].trim()
    } else {
        right
    };

    let left_name = first_arg_name(left);
    let right_name = first_arg_name(right);

    match (left_name.as_deref(), right_name.as_deref()) {
        (None, None) => Ok(CallShape::Nular),
        (None, Some(r)) => Ok(CallShape::Unary {
            right: r.to_string(),
        }),
        (Some(l), Some(r)) => Ok(CallShape::Binary {
            left: l.to_string(),
            right: r.to_string(),
        }),
        (Some(_), None) => Err(format!("binary-looking left without right: `{text}`")),
    }
}

fn find_command_token(text: &str, command: &str) -> Option<usize> {
    // Exact
    if let Some(p) = text.find(command) {
        return Some(p);
    }
    // Case-insensitive scan
    let lower_text = text.to_ascii_lowercase();
    let lower_cmd = command.to_ascii_lowercase();
    lower_text.find(&lower_cmd)
}

fn first_arg_name(side: &str) -> Option<String> {
    let s = side.trim();
    if s.is_empty() {
        return None;
    }
    if s.starts_with('[') {
        // Array argument — name it from the first identifier inside, or "array"
        let inner = s.trim_start_matches('[').trim();
        let name = inner
            .split([',', ' ', ']'])
            .find(|p| !p.is_empty())
            .unwrap_or("array");
        return Some(sanitize_arg_name(name));
    }
    // Single identifier (stop at whitespace)
    let name = s.split_whitespace().next().unwrap_or(s);
    Some(sanitize_arg_name(name))
}

fn sanitize_arg_name(name: &str) -> String {
    name.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .to_string()
}

fn parse_params(params_text: &str) -> Result<Vec<Param>, String> {
    if params_text.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Strip HTML tables entirely from the params block
    let text = strip_html_tables(params_text);
    let text = strip_html_tags(&text);

    let lines: Vec<&str> = text.lines().collect();
    let mut params = Vec::new();
    let mut pending_since: Option<String> = None;
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            i += 1;
            continue;
        }

        // `since  2.12` on its own line
        if let Some(ver) = parse_since_line(line) {
            pending_since = Some(ver);
            i += 1;
            continue;
        }

        // Skip bullet sub-lines (belong to previous param description / prose unions).
        // Do not treat bold markdown `**name**:` as a bullet.
        if (line.starts_with("- ") || line.starts_with("* "))
            && !line.starts_with("**")
        {
            i += 1;
            continue;
        }
        if line == "-" || line == "*" {
            i += 1;
            continue;
        }

        // Param line: `name: Type` or `**name**: Type - desc` or `name: Type - (Optional, default X) desc`
        // Also: `name: Type or Type` and multiline unions starting with `name - can be one of:`
        if let Some(param) = try_parse_param_line(line, &mut pending_since) {
            // Peek ahead for multiline "can be one of:" union bodies — already handled in type phrase if inline.
            // Collect continuation bullet type lines for prose unions.
            let mut p = param;
            if p.typ.is_unknown() || matches!(p.typ, SqfType::Anything) {
                // Look ahead for bullet types
                let mut j = i + 1;
                let mut union_types = Vec::new();
                while j < lines.len() {
                    let l = lines[j].trim();
                    if l.is_empty() {
                        j += 1;
                        continue;
                    }
                    if let Some(stripped) = l.strip_prefix("- ").or_else(|| l.strip_prefix("* ")) {
                        // `_x`: iterated item  OR  `[[Type]]` style stripped to `Type - desc`
                        let type_src = stripped
                            .split_once(" - ")
                            .map(|(ty, _)| ty)
                            .unwrap_or(stripped);
                        let t = parse_type_phrase(type_src);
                        if !t.is_unknown() {
                            union_types.push(t);
                        }
                        j += 1;
                        continue;
                    }
                    // Next top-level param or since line ends the body
                    if parse_since_line(l).is_some() || looks_like_param_start(l) {
                        break;
                    }
                    j += 1;
                }
                if union_types.len() > 1 {
                    union_types.dedup();
                    p.typ = SqfType::OneOf(union_types);
                } else if union_types.len() == 1 {
                    p.typ = union_types.remove(0);
                }
            }
            params.push(p);
            i += 1;
            continue;
        }

        i += 1;
    }
    Ok(params)
}

fn strip_html_tables(s: &str) -> String {
    let re = html_table_re();
    re.replace_all(s, "").into_owned()
}

fn html_table_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?is)<table\b[^>]*>.*?</table>").expect("regex")
    })
}

fn parse_since_line(line: &str) -> Option<String> {
    let t = line.trim();
    let lower = t.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("since") {
        let ver = rest.trim();
        if version_re().is_match(ver) {
            // Preserve digits from original
            let digits: String = t
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .collect();
            let digits = digits.split_whitespace().next().unwrap_or("").to_string();
            if version_re().is_match(&digits) {
                return Some(digits);
            }
        }
    }
    None
}

fn looks_like_param_start(line: &str) -> bool {
    let t = line.trim().trim_start_matches('*').trim_start_matches('*').trim();
    // `name: Type` — name without spaces (or bold-wrapped), then colon
    if let Some((name, rest)) = t.split_once(':') {
        let name = name.trim().trim_matches('*').trim();
        if name.is_empty() || name.contains(' ') {
            // allow "Team Member" style? no — that's a type. Param names are camelCase/identifiers.
            // Some use spaces rarely; allow if rest looks like a type.
        }
        let rest = rest.trim();
        if !rest.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '*')
        {
            return true;
        }
        // bold name: already trimmed *
        if !rest.is_empty() && !name.is_empty() {
            return true;
        }
    }
    false
}

fn try_parse_param_line(line: &str, pending_since: &mut Option<String>) -> Option<Param> {
    let mut t = line.trim().to_string();
    // Strip leading/trailing bold markers around the whole line fragments
    t = t.replace("**", "");
    // Non-breaking spaces sometimes appear as `\u{00a0}`
    t = t.replace('\u{00a0}', " ");

    // Pattern: name: TypeRest
    let (name_part, rest) = t.split_once(':')?;
    let name = name_part.trim().trim_matches('*').trim().to_string();
    if name.is_empty() {
        return None;
    }
    // Reject "Example 1" style false positives
    if name.starts_with("Example") || name == "Syntax" || name == "Return Value" {
        return None;
    }

    let rest = rest.trim();
    // Prose union header without inline type: `toSpawn - can be one of:`
    if rest.is_empty()
        || rest.to_ascii_lowercase().contains("can be one of")
        || rest.to_ascii_lowercase().contains("one of:")
    {
        let (optional, default, desc) = parse_optional_default(rest);
        return Some(Param {
            name,
            typ: SqfType::Unknown, // filled from bullets by caller
            description: if desc.is_empty() { None } else { Some(desc) },
            optional,
            default,
            since: pending_since.take(),
        });
    }

    let (type_part, desc_part) = split_type_and_desc(rest);
    let typ = parse_type_phrase(type_part);
    let (optional, default, desc) = parse_optional_default(desc_part);

    Some(Param {
        name,
        typ,
        description: if desc.is_empty() { None } else { Some(desc) },
        optional,
        default,
        since: pending_since.take(),
    })
}

fn split_type_and_desc(rest: &str) -> (&str, &str) {
    // Optional marker may sit on the type side before the dash:
    // `Boolean (Optional, default true) - false to skip...`
    if let Some(idx) = rest.find(" (Optional").or_else(|| {
        let lower = rest.to_ascii_lowercase();
        lower.find(" (optional").map(|i| {
            // ASCII so byte index matches
            let _ = lower;
            i
        })
    }) {
        // Only treat as desc split if this is before a type/desc dash, or there is no dash.
        let dash = rest.find(" - ");
        if dash.is_none_or(|d| idx < d) {
            return (rest[..idx].trim(), rest[idx..].trim());
        }
    }
    // Prefer " - " as delimiter between type and description
    if let Some((ty, desc)) = rest.split_once(" - ") {
        return (ty.trim(), desc.trim());
    }
    (rest.trim(), "")
}

fn parse_optional_default(desc: &str) -> (bool, Option<String>, String) {
    let mut optional = false;
    let mut default = None;
    let mut desc = desc.to_string();

    // (Optional, default X) or (Optional)
    if let Some(start) = desc.find("(Optional").or_else(|| {
        desc.to_ascii_lowercase()
            .find("(optional")
            .map(|i| {
                // map back — same byte index for ASCII
                i
            })
    }) {
        if let Some(end_rel) = desc[start..].find(')') {
            let end = start + end_rel;
            let inner = &desc[start + 1..end]; // without parens
            optional = true;
            if let Some(def) = inner
                .split_once("default")
                .map(|(_, d)| d.trim().trim_start_matches(',').trim().to_string())
            {
                if !def.is_empty() {
                    default = Some(def);
                }
            }
            desc = format!("{}{}", &desc[..start], &desc[end + 1..])
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
        }
    } else if desc.to_ascii_lowercase().contains("(optional)") {
        optional = true;
        desc = desc.replace("(Optional)", "").replace("(optional)", "");
        desc = desc.split_whitespace().collect::<Vec<_>>().join(" ");
    }

    (optional, default, desc.trim().to_string())
}

fn parse_return(return_text: &str) -> (SqfType, Option<String>) {
    let t = return_text.trim();
    if t.is_empty()
        || t == "-"
        || t == "\\"
        || t.eq_ignore_ascii_case("*return value needed*")
        || t == "\\-"
    {
        return (SqfType::Unknown, None);
    }
    let (type_part, desc_part) = split_type_and_desc(t);
    let typ = parse_type_phrase(type_part);
    let desc = desc_part.trim();
    (
        typ,
        if desc.is_empty() {
            None
        } else {
            Some(desc.to_string())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_names() {
        assert_eq!(command_name_from_stem("a_%26%26_b"), "&&");
        assert_eq!(command_name_from_stem("a_less%3D_b"), "<=");
        assert_eq!(command_name_from_stem("%2B"), "+");
        assert_eq!(command_name_from_stem("! a"), "!");
        assert_eq!(command_name_from_stem("setDamage"), "setDamage");
        assert_eq!(command_name_from_stem("a_%2A_b"), "*");
        assert_eq!(command_name_from_stem("a_%2F_b"), "/");
        assert_eq!(command_name_from_stem("a_%3A_b"), ":");
        assert_eq!(command_name_from_stem("config_greater_greater_name"), ">>");
    }
}
