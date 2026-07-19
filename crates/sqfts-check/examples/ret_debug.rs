fn main() {
    use sqfts_check::{check_source, CheckFlags, DeclarationSet, StsCode};
    use sqfts_db::CommandDb;
    let db = CommandDb::load_default().unwrap();
    let decls = DeclarationSet::default();
    let flags = CheckFlags::default();
    let src = r#"private _a: code(unit: object): boolean = { 
	if (alive player) then {
		3
	} else {
		4
	}
 };
"#;
    let result = check_source(src, "t.sqfts", &db, &decls, &flags).unwrap();
    for d in &result.diagnostics {
        println!("{:?} {}", d.code, d.message);
    }
    if result.diagnostics.is_empty() {
        println!("(no diagnostics)");
    }
}
