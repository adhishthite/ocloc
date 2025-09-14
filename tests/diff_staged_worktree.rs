use std::fs;

#[test]
fn diff_staged_and_worktree_modes() {
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

    // commit a file
    let fpath = root.join("file.rs");
    fs::write(&fpath, "fn main() {}\n").unwrap();
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

    // modify file, stage it
    fs::write(&fpath, "// change\nfn main() {}\n").unwrap();
    assert!(
        std::process::Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(root)
            .status()
            .unwrap()
            .success()
    );

    // staged diff (HEAD vs index)
    let bin = env!("CARGO_BIN_EXE_ocloc");
    let staged = std::process::Command::new(bin)
        .arg("diff")
        .arg("--staged")
        .arg("--json")
        .current_dir(root)
        .output()
        .expect("run diff staged");
    assert!(staged.status.success());
    let v: serde_json::Value = serde_json::from_slice(&staged.stdout).unwrap();
    assert!(v["totals"]["total_net"].as_i64().unwrap() >= 0);

    // add an unstaged change (worktree diff should catch it vs index)
    fs::write(&fpath, "// another\n// change\nfn main() {}\n").unwrap();
    let work = std::process::Command::new(bin)
        .arg("diff")
        .arg("--working-tree")
        .arg("--json")
        .current_dir(root)
        .output()
        .expect("run diff worktree");
    assert!(work.status.success());
    let v2: serde_json::Value = serde_json::from_slice(&work.stdout).unwrap();
    assert!(v2["totals"]["total_net"].as_i64().unwrap() >= 0);
}
