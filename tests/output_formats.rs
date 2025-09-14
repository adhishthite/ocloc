use std::path::PathBuf;

#[test]
fn json_and_csv_outputs_work() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/simple_repo");
    // JSON
    let out_json = std::process::Command::new(env!("CARGO_BIN_EXE_ocloc"))
        .arg(&root)
        .arg("--json")
        .output()
        .expect("run json");
    assert!(out_json.status.success());
    let s = String::from_utf8_lossy(&out_json.stdout);
    assert!(s.trim_start().starts_with("{"));
    assert!(s.contains("\"languages\""));
    assert!(s.contains("\"totals\""));

    // CSV
    let out_csv = std::process::Command::new(env!("CARGO_BIN_EXE_ocloc"))
        .arg(&root)
        .arg("--csv")
        .output()
        .expect("run csv");
    assert!(out_csv.status.success());
    let s = String::from_utf8_lossy(&out_csv.stdout);
    assert!(s.starts_with("language,files,code,comment,blank,total"));
}
