//! Integration tests for `sqfts build` / identity.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sqfts"))
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn build_erases_annotations() {
    let dir = tempfile_dir();
    let src = fixtures().join("fn_impoundVehicle.sqfts");
    std::fs::copy(&src, dir.join("fn_impoundVehicle.sqfts")).unwrap();
    std::fs::write(
        dir.join("sqfts.toml"),
        r#"sources = ["."]
out_dir = "out_sqf"
"#,
    )
    .unwrap();

    let status = bin()
        .arg("build")
        .arg(&dir)
        .status()
        .expect("run sqfts build");
    assert!(status.success());

    let out = std::fs::read_to_string(dir.join("out_sqf/fn_impoundVehicle.sqf")).unwrap();
    assert!(!out.contains(": object"));
    assert!(!out.contains("type feeTier"));
    assert!(out.contains("\"_vehicle\""));
    assert!(out.contains("[\"_fee\", 0]"));
}

#[test]
fn check_resolves_relative_include() {
    let root = fixtures().join("include_mission");
    let output = bin()
        .arg("check")
        .arg(&root)
        .output()
        .expect("run sqfts check");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        output.status.success() || !combined.contains("preprocessor:"),
        "relative #include should resolve; got status={:?} out={combined}",
        output.status
    );
    assert!(
        !combined.contains("IncludeNotFound") && !combined.contains("not found"),
        "unexpected include failure: {combined}"
    );
}

#[test]
fn build_erases_define_body_annotations() {
    let dir = tempfile_dir();
    let src = fixtures().join("macro_typed.sqfts");
    std::fs::copy(&src, dir.join("macro_typed.sqfts")).unwrap();
    std::fs::write(
        dir.join("sqfts.toml"),
        r#"sources = ["."]
out_dir = "out_sqf"
"#,
    )
    .unwrap();

    let status = bin()
        .arg("build")
        .arg(&dir)
        .status()
        .expect("run sqfts build");
    assert!(status.success());

    let out = std::fs::read_to_string(dir.join("out_sqf/macro_typed.sqf")).unwrap();
    assert!(
        out.contains("#define TYPED private _x = 1;"),
        "define body should be erased, got: {out:?}"
    );
    assert!(!out.contains(": number"));
}

#[test]
fn identity_plain_sqf_via_syntax() {
    // E1: plain SQF files that happen to be scanned as .sqfts without annotations
    // stay byte-identical. Corpus mass-test is env-gated below.
    let plain = std::fs::read_to_string(fixtures().join("plain.sqf")).unwrap();
    let erased = sqfts_syntax::erase(&plain, &sqfts_syntax::EraseOptions::default()).unwrap();
    assert_eq!(erased.text, plain);
}

#[test]
fn corpus_identity_when_env_set() {
    let Some(root) = std::env::var_os("SQFTS_TEST_CORPUS") else {
        eprintln!("SQFTS_TEST_CORPUS unset — skipping corpus identity");
        return;
    };
    let root = PathBuf::from(root);
    assert!(
        root.is_dir(),
        "SQFTS_TEST_CORPUS is not a directory: {}",
        root.display()
    );

    let mut checked = 0usize;
    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("sqf") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        // Skip files with null bytes / non-UTF8 already handled by read
        let erased = sqfts_syntax::erase(&src, &sqfts_syntax::EraseOptions::default())
            .unwrap_or_else(|e| panic!("erase failed on {}: {e}", path.display()));
        assert_eq!(
            erased.text,
            src,
            "E1 identity failed for {}",
            path.display()
        );
        checked += 1;
        if checked >= 5000 {
            break; // safety cap
        }
    }
    assert!(checked > 0, "no .sqf files found under corpus");
    eprintln!("corpus identity OK on {checked} files");
}

fn tempfile_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sqfts-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
