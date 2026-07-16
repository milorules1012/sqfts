fn main() {
    use hemtt_common::config::{PDriveOption, PreprocessorOptions};
    use hemtt_preprocessor::Processor;
    use hemtt_sqf::parser::database::Database as HemttDb;
    use hemtt_sqf::{Expression, Statement};
    use hemtt_workspace::Workspace;
    use std::io::Write;

    let src = r#"private _x: number = 1;
_vehicle setDamage "boom";
"#;
    let src_lf = src.replace("\r\n", "\n");
    let e = sqfts_syntax::erase(&src_lf, &sqfts_syntax::EraseOptions::default()).unwrap();

    let workspace = Workspace::builder()
        .memory()
        .finish(None, false, &PDriveOption::Disallow)
        .unwrap();
    let path = workspace.join("check.sqf").unwrap();
    {
        let mut f = path.create_file().unwrap();
        f.write_all(e.text.as_bytes()).unwrap();
    }
    let processed = Processor::run(&path, &PreprocessorOptions::default()).unwrap();
    let stmts = hemtt_sqf::parser::run(&HemttDb::a3(false), &processed).unwrap();

    fn find_damage2(expr: &Expression) -> Option<std::ops::Range<usize>> {
        match expr {
            Expression::BinaryCommand(cmd, l, r, span) => {
                if cmd.as_str().eq_ignore_ascii_case("setDamage") {
                    let rs = format!("{r:?}");
                    if rs.contains("boom") {
                        return Some(span.clone());
                    }
                }
                find_damage2(l).or_else(|| find_damage2(r))
            }
            Expression::UnaryCommand(_, a, _) => find_damage2(a),
            Expression::Array(els, _) | Expression::ConsumeableArray(els, _) => {
                els.iter().find_map(find_damage2)
            }
            _ => None,
        }
    }
    for st in stmts.content() {
        let expr = match st {
            Statement::Expression(e, _)
            | Statement::AssignLocal(_, e, _)
            | Statement::AssignGlobal(_, e, _) => e,
        };
        if let Some(span) = find_damage2(expr) {
            println!("AST span={span:?}");
            for which in [span.start, span.end] {
                let maps = processed.mappings(which);
                let m = maps.last().unwrap();
                let p_start = m.processed_start().offset();
                let p_end = m.processed_end().offset();
                let o_start = m.original().start().offset();
                let o_end = m.original().end().offset();
                let delta = which.saturating_sub(p_start);
                let o_len = o_end.saturating_sub(o_start);
                let erased_off = o_start + delta.min(o_len);
                let orig = e.span_map.to_original(erased_off);
                println!(
                    "  off {which}: tok={:?} p={p_start}..{p_end} o={o_start}..{o_end} -> erased {erased_off} -> orig {orig}",
                    m.token().symbol()
                );
            }
        }
    }
}
