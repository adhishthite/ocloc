use std::fs;
use std::io::Write;

#[test]
fn diff_mode_reports_added_and_modified() {
    // Prepare a temporary git repo
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // init repo
    assert!(
        std::process::Command::new("git")
            .arg("-c")
            .arg("init.defaultBranch=main")
            .arg("init")
            .current_dir(root)
            .status()
            .expect("git init")
            .success()
    );

    // Add a Rust file and commit
    let a_rs = root.join("a.rs");
    fs::write(&a_rs, "// first\nfn main() {}\n").unwrap();
    assert!(
        std::process::Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        std::process::Command::new("git")
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                "initial"
            ])
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );

    // Modify a.rs and add a new Python file
    let mut f = fs::OpenOptions::new().append(true).open(&a_rs).unwrap();
    writeln!(f, "// added line").unwrap();
    let b_py = root.join("b.py");
    fs::write(&b_py, "print(123)\n").unwrap();
    assert!(
        std::process::Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        std::process::Command::new("git")
            .args([
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                "update"
            ])
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );

    // Run diff between HEAD~1 and HEAD
    let bin = env!("CARGO_BIN_EXE_ocloc");
    let out = std::process::Command::new(bin)
        .arg("diff")
        .arg("--base")
        .arg("HEAD~1")
        .arg("--head")
        .arg("HEAD")
        .arg("--json")
        .arg("--by-file")
        .current_dir(root)
        .output()
        .expect("run diff");
    assert!(
        out.status.success(),
        "diff command failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);

    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    // Totals should indicate a net positive addition
    let total_net = v["totals"]["total_net"].as_i64().unwrap();
    assert!(total_net > 0);
    // There should be at least one Rust and one Python entry in languages
    let langs = v["languages"].as_object().unwrap();
    assert!(langs.keys().any(|k| k == "Rust"));
    assert!(langs.keys().any(|k| k == "Python"));
}
