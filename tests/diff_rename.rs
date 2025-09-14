use std::fs;
use std::io::Write;

#[test]
fn diff_reports_renames() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // init
    assert!(
        std::process::Command::new("git")
            .arg("-c")
            .arg("init.defaultBranch=main")
            .arg("init")
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );

    // Create a text file and commit
    let txt = root.join("doc.txt");
    fs::write(&txt, "hello world\n").unwrap();
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
                "initial",
            ])
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );

    // Rename to markdown and commit
    let md = root.join("doc.md");
    assert!(
        std::process::Command::new("git")
            .args(["mv", "doc.txt", "doc.md"])
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );
    // Optionally modify content
    let mut f = fs::OpenOptions::new().append(true).open(&md).unwrap();
    writeln!(f, "more").unwrap();
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
                "rename",
            ])
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );

    let bin = env!("CARGO_BIN_EXE_ocloc");
    let out = std::process::Command::new(bin)
        .arg("diff")
        .arg("--base")
        .arg("HEAD~1")
        .arg("--head")
        .arg("HEAD")
        .arg("--json")
        .current_dir(root)
        .output()
        .expect("run diff");
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    let files_renamed = v["files_renamed"].as_u64().unwrap();
    assert!(files_renamed >= 1);
}
