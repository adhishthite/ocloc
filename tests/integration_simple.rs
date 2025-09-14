use std::path::PathBuf;

#[test]
fn runs_on_simple_fixture() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/simple_repo");
    assert!(root.exists());

    // For now, we just invoke main binary through run() is not trivial from tests without cmd; so
    // test traversal and analyzer through lib API indirectly by running a small count via CLI as a smoke test
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_ocloc"))
        .arg(root)
        .arg("--json")
        .output()
        .expect("run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // schema check
    assert!(stdout.contains("\"totals\""));
    assert!(stdout.contains("\"languages\""));
    assert!(stdout.contains("Rust"));
    assert!(stdout.contains("Python"));
}
