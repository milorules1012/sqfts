fn main() {
    let src = r#"private _x: number = 1;
_vehicle setDamage "boom";
"#;
    let src_lf = src.replace("\r\n", "\n");
    let e = sqfts_syntax::erase(&src_lf, &sqfts_syntax::EraseOptions::default()).unwrap();
    for off in [0usize, 10] {
        let o = e.span_map.to_original(off);
        println!(
            "erased {off} -> original {o} snip={:?}",
            src_lf.get(o..o + 12.min(src_lf.len().saturating_sub(o)))
        );
    }
    println!("erased at 0..10={:?}", e.text.get(0..10));
    for a in &e.annotations {
        println!("ann {:?} span={:?}", a.kind, a.span);
    }
}
