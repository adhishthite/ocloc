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

#[test]
fn analyze_reader_parity_with_analyze_file() {
    use std::io::Cursor;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("parity.rs");
    std::fs::write(&path, "// c1\nfn main() {}\n/* x */\n").unwrap();

    let file_counts = ocloc::analyzer::analyze_file(&path).unwrap();
    let data = std::fs::read(&path).unwrap();
    let reader_counts = ocloc::analyzer::analyze_reader(Cursor::new(data), &path).unwrap();
    assert_eq!(file_counts.total, reader_counts.total);
    assert_eq!(file_counts.code, reader_counts.code);
    assert_eq!(file_counts.comment, reader_counts.comment);
    assert_eq!(file_counts.blank, reader_counts.blank);
}
