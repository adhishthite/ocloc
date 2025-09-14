use std::path::PathBuf;

#[test]
fn detects_shebang_languages_in_fixture() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shebang_repo");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_ocloc"))
        .arg(&root)
        .arg("--json")
        .output()
        .expect("run json");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("Python"));
    assert!(s.contains("Shell"));
}
