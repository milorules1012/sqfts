//! Emit arma3-wiki-inspired YAML for extracted commands.

use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::model::{CallShape, ExtractedCommand, Param, SqfType, Syntax};

#[derive(Serialize)]
struct YamlCommand<'a> {
    name: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    description: &'a str,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    alias: &'a [String],
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    groups: &'a [String],
    syntax: Vec<YamlSyntax<'a>>,
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    intro_versions: &'a [String],
    source_file: &'a str,
}

#[derive(Serialize)]
struct YamlSyntax<'a> {
    call: YamlCall<'a>,
    syntax_text: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    params: Vec<YamlParam<'a>>,
    #[serde(rename = "return")]
    ret: YamlReturn<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<&'a str>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum YamlCall<'a> {
    Nular,
    Unary { right: &'a str },
    Binary { left: &'a str, right: &'a str },
}

#[derive(Serialize)]
struct YamlParam<'a> {
    name: &'a str,
    #[serde(rename = "type")]
    typ: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<&'a str>,
}

#[derive(Serialize)]
struct YamlReturn<'a> {
    #[serde(rename = "type")]
    typ: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

/// Write one YAML file per command under `out_dir/commands/`.
pub fn emit_commands(commands: &[ExtractedCommand], out_dir: &Path) -> anyhow::Result<usize> {
    let commands_dir = out_dir.join("commands");
    fs::create_dir_all(&commands_dir)?;
    let mut written = 0usize;
    for cmd in commands {
        let yaml = render_command(cmd);
        let filename = safe_filename(&cmd.name) + ".yml";
        fs::write(commands_dir.join(filename), yaml)?;
        written += 1;
    }
    Ok(written)
}

#[must_use]
pub fn render_command(cmd: &ExtractedCommand) -> String {
    let syntax: Vec<YamlSyntax<'_>> = cmd.syntaxes.iter().map(render_syntax).collect();
    let doc = YamlCommand {
        name: &cmd.name,
        description: &cmd.description,
        alias: &cmd.aliases,
        groups: &cmd.groups,
        syntax,
        intro_versions: &cmd.intro_versions,
        source_file: &cmd.source_file,
    };
    // serde_yaml sometimes wraps oddly; keep it simple
    serde_yaml::to_string(&doc).unwrap_or_else(|e| format!("# serialize error: {e}\n"))
}

fn render_syntax(syn: &Syntax) -> YamlSyntax<'_> {
    YamlSyntax {
        call: match &syn.call {
            CallShape::Nular => YamlCall::Nular,
            CallShape::Unary { right } => YamlCall::Unary { right },
            CallShape::Binary { left, right } => YamlCall::Binary { left, right },
        },
        syntax_text: &syn.syntax_text,
        params: syn.params.iter().map(render_param).collect(),
        ret: YamlReturn {
            typ: type_tag(&syn.return_type),
            description: syn.return_description.as_deref(),
        },
        since: syn.since.as_deref(),
    }
}

fn render_param(p: &Param) -> YamlParam<'_> {
    YamlParam {
        name: &p.name,
        typ: type_tag(&p.typ),
        description: p.description.as_deref(),
        optional: p.optional,
        default: p.default.as_deref(),
        since: p.since.as_deref(),
    }
}

fn type_tag(t: &SqfType) -> String {
    match t {
        SqfType::ArrayOf(inner) => format!("Array of {}", type_tag(inner)),
        SqfType::OneOf(parts) => parts
            .iter()
            .map(type_tag)
            .collect::<Vec<_>>()
            .join(" or "),
        other => other.wiki_name(),
    }
}

/// Filesystem-safe filename for a command (operators get ASCII names).
#[must_use]
pub fn safe_filename(name: &str) -> String {
    match name {
        "!" => "op_not".into(),
        "+" => "op_plus".into(),
        "-" => "op_minus".into(),
        "*" => "op_mul".into(),
        "/" => "op_div".into(),
        "%" => "op_mod".into(),
        "^" => "op_pow".into(),
        "==" => "op_eq".into(),
        "!=" => "op_ne".into(),
        "=" => "op_assign".into(),
        "&&" => "op_and".into(),
        "||" => "op_or".into(),
        ">" => "op_gt".into(),
        ">=" => "op_gte".into(),
        "<" => "op_lt".into(),
        "<=" => "op_lte".into(),
        "#" => "op_hash".into(),
        ":" => "op_colon".into(),
        ">>" => "op_config_path".into(),
        other => other
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CallShape, ExtractedCommand, SqfType, Syntax};

    #[test]
    fn emits_alive_shaped_yaml() {
        let cmd = ExtractedCommand {
            name: "alive".into(),
            source_file: "alive".into(),
            description: "Check if alive".into(),
            aliases: vec![],
            groups: vec!["Object Manipulation".into()],
            syntaxes: vec![Syntax {
                call: CallShape::Unary {
                    right: "object".into(),
                },
                syntax_text: "alive object".into(),
                params: vec![Param {
                    name: "object".into(),
                    typ: SqfType::Object,
                    description: None,
                    optional: false,
                    default: None,
                    since: None,
                }],
                return_type: SqfType::Boolean,
                return_description: Some("true if alive".into()),
                since: None,
            }],
            intro_versions: vec!["1.00".into()],
        };
        let yaml = render_command(&cmd);
        assert!(yaml.contains("name: alive"));
        assert!(yaml.contains("type: Boolean"));
        assert!(yaml.contains("type: Object"));
        assert!(yaml.contains("kind: unary"));
    }
}
