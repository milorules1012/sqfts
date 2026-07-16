fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: cargo run -p sqfts-check --example diag_spans -- <path-to-.sqfts>");
    let src = std::fs::read_to_string(&path).unwrap();
    let src_lf = src.replace("\r\n", "\n");
    let e = sqfts_syntax::erase(&src_lf, &Default::default()).unwrap();
    // Simulate: processed span 224..233
    let erased_start = 767usize;
    let erased_end = 776usize;
    let lf_start = e.span_map.to_original(erased_start);
    let lf_end = e.span_map.to_original(erased_end);
    println!(
        "lf range {lf_start}..{lf_end} = {:?}",
        &src_lf[lf_start..lf_end]
    );
    let crlf_start = lf_start + src_lf[..lf_start].bytes().filter(|&b| b == b'\n').count();
    let crlf_end = lf_end + src_lf[..lf_end].bytes().filter(|&b| b == b'\n').count();
    println!(
        "crlf range {crlf_start}..{crlf_end} = {:?}",
        &src[crlf_start..crlf_end]
    );

    // What does check_source actually produce now?
    let db = sqfts_db::CommandDb::load_default().unwrap();
    let r = sqfts_check::check_source(
        &src,
        "x.sqfts",
        &db,
        &Default::default(),
        &Default::default(),
    )
    .unwrap();
    for d in r.diagnostics {
        println!("DIAG {:?} {:?}", d.message, d.span);
    }
}
