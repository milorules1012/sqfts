use comref_extract::model::{CallShape, ParseOutcome, SqfType};
use comref_extract::parse::parse_comref_page;

fn parse_fixture(stem: &str) -> ParseOutcome {
    let path = format!("tests/fixtures/{stem}.md");
    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    parse_comref_page(stem, &content)
}

fn expect_ok(stem: &str) -> comref_extract::model::ExtractedCommand {
    match parse_fixture(stem) {
        ParseOutcome::Ok(cmd) => cmd,
        other => panic!("{stem}: expected Ok, got {other:?}"),
    }
}

#[test]
fn alive_unary() {
    let cmd = expect_ok("alive");
    assert_eq!(cmd.name, "alive");
    assert_eq!(cmd.syntaxes.len(), 1);
    assert!(matches!(
        cmd.syntaxes[0].call,
        CallShape::Unary { ref right } if right == "object"
    ));
    assert_eq!(cmd.syntaxes[0].params[0].typ, SqfType::Object);
    assert_eq!(cmd.syntaxes[0].return_type, SqfType::Boolean);
}

#[test]
fn time_nular() {
    let cmd = expect_ok("time");
    assert_eq!(cmd.syntaxes.len(), 1);
    assert!(matches!(cmd.syntaxes[0].call, CallShape::Nular));
    assert!(cmd.syntaxes[0].params.is_empty());
    assert_eq!(cmd.syntaxes[0].return_type, SqfType::Number);
}

#[test]
fn set_damage_two_syntaxes() {
    let cmd = expect_ok("setDamage");
    assert!(cmd.syntaxes.len() >= 2);
    assert_eq!(cmd.syntaxes[0].return_type, SqfType::Nothing);
    assert_eq!(cmd.syntaxes[0].params[0].typ, SqfType::Object);
    assert_eq!(cmd.syntaxes[0].params[1].typ, SqfType::Number);
    // Alternative syntax has optional params
    let alt = &cmd.syntaxes[1];
    assert!(alt
        .params
        .iter()
        .any(|p| p.name == "useEffects" && p.optional));
    assert!(alt.params.iter().any(|p| p.name == "killer"));
}

#[test]
fn get_pos_three_syntaxes() {
    let cmd = expect_ok("getPos");
    assert!(cmd.syntaxes.len() >= 3);
    assert!(matches!(
        cmd.syntaxes[0].return_type,
        SqfType::Position3dAGLS | SqfType::Position3dAGL | SqfType::Array | SqfType::Position
    ));
}

#[test]
fn foreach_strips_html_table() {
    let cmd = expect_ok("forEach");
    assert!(cmd.syntaxes.len() >= 2);
    let first = &cmd.syntaxes[0];
    assert!(
        first.params.iter().any(|p| p.name == "code"),
        "expected code param, got {:?}",
        first.params
    );
    assert!(
        first
            .params
            .iter()
            .any(|p| p.name == "array" && p.typ == SqfType::Array),
        "expected array param"
    );
}

#[test]
fn operator_and() {
    let cmd = expect_ok("a_%26%26_b");
    assert_eq!(cmd.name, "&&");
    assert!(cmd.syntaxes.len() >= 2);
    assert!(matches!(cmd.syntaxes[0].call, CallShape::Binary { .. }));
    assert_eq!(cmd.syntaxes[0].return_type, SqfType::Boolean);
}

#[test]
fn if_control_structure() {
    let cmd = expect_ok("if");
    assert!(!cmd.syntaxes.is_empty());
    assert!(
        matches!(cmd.syntaxes[0].return_type, SqfType::IfType)
            || cmd.syntaxes[0].return_type.to_string().contains("If"),
        "got {:?}",
        cmd.syntaxes[0].return_type
    );
}

#[test]
fn add_action_optionals() {
    let cmd = expect_ok("addAction");
    let syn = &cmd.syntaxes[0];
    assert!(syn.params.iter().any(|p| p.name == "object"));
    assert!(syn
        .params
        .iter()
        .any(|p| p.name == "title" && p.typ == SqfType::String));
    assert!(syn
        .params
        .iter()
        .any(|p| p.name == "arguments" && p.optional));
    assert_eq!(syn.return_type, SqfType::Number);
}

#[test]
fn player_nular() {
    let cmd = expect_ok("player");
    assert!(matches!(cmd.syntaxes[0].call, CallShape::Nular));
    assert_eq!(cmd.syntaxes[0].return_type, SqfType::Object);
}

#[test]
fn all_golden_fixtures_parse_ok() {
    let fixtures = [
        "alive",
        "setDamage",
        "getPos",
        "addAction",
        "forEach",
        "time",
        "while",
        "if",
        "a_%26%26_b",
        "waves",
        "createVehicle",
        "remoteExec",
        "params",
        "select",
        "count",
        "private",
        "spawn",
        "call",
        "player",
    ];
    let mut failures = Vec::new();
    for stem in fixtures {
        match parse_fixture(stem) {
            ParseOutcome::Ok(_) => {}
            other => failures.push(format!("{stem}: {other:?}")),
        }
    }
    assert!(
        failures.is_empty(),
        "fixtures failed to parse:\n{}",
        failures.join("\n")
    );
}

#[test]
fn yaml_roundtrip_shape() {
    let cmd = expect_ok("alive");
    let yaml = comref_extract::emit::render_command(&cmd);
    assert!(yaml.contains("name: alive"));
    let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("yaml");
    assert_eq!(parsed["name"].as_str(), Some("alive"));
}
