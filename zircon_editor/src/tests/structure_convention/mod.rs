use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::Value;

static EDITOR_STRUCTURE_AUDIT: OnceLock<Value> = OnceLock::new();

#[test]
fn editor_ui_10_module_convention_audit_report_has_expected_shape() {
    let report = editor_structure_audit();
    let gate = &report["module_convention_gate"];
    assert!(
        gate["m1_gate_status"] == "migration-debt-present"
            || gate["m1_gate_status"] == "classified-and-clear",
        "unexpected editor module convention gate status: {gate:#?}"
    );
    assert!(
        gate["migration_debt_count"].is_u64(),
        "editor module convention gate should expose a numeric migration_debt_count: {gate:#?}"
    );
    assert!(
        gate["oversized_production_files"].is_array(),
        "editor module convention gate should expose oversized production file entries: {gate:#?}"
    );
    assert!(
        gate["oversized_test_file_count"].is_u64(),
        "editor module convention gate should expose a numeric oversized test file count: {gate:#?}"
    );
    assert!(
        gate["oversized_test_files"].is_array(),
        "editor module convention gate should expose oversized test file entries: {gate:#?}"
    );
    assert!(
        gate["oversized_test_file_exemptions"].is_array(),
        "editor module convention gate should expose explicit oversized test file exemptions: {gate:#?}"
    );
}

#[test]
fn editor_ui_10_no_oversized_test_files() {
    let gate = editor_structure_audit();
    let oversized = gate["module_convention_gate"]["oversized_test_files"]
        .as_array()
        .expect("EditorUI10 audit must return an oversized test-file array");

    assert!(
        oversized.is_empty(),
        "EditorUI10 test-file budget is a zero-tolerance gate; split these functional-test owners: {oversized:#?}"
    );
}

#[test]
fn editor_ui_10_visual_style_owner_tree_is_hard_cut_over() {
    let report = editor_structure_audit();
    let visual_style = &report["module_convention_gate"]["visual_style_owner_tree"];

    assert_eq!(
        visual_style["old_file_exists"], false,
        "visual-style hard cutover should delete the old single-file owner"
    );
    assert_eq!(
        visual_style["missing_owner_files"]
            .as_array()
            .expect("missing_owner_files should be an array")
            .len(),
        0,
        "visual-style owner tree should include all required owner files: {visual_style:#?}"
    );
}

fn editor_structure_audit() -> &'static Value {
    EDITOR_STRUCTURE_AUDIT.get_or_init(|| {
        let repo_root = repo_root();
        let audit_script = repo_root.join(
            ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py",
        );
        assert!(
            audit_script.exists(),
            "missing editor structure audit script at {}",
            audit_script.display()
        );
        run_editor_structure_audit(&audit_script, &repo_root)
    })
}

fn run_editor_structure_audit(audit_script: &Path, repo_root: &Path) -> Value {
    let output = run_python_audit(audit_script, repo_root);
    assert!(
        output.status.success(),
        "editor structure audit failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("editor structure audit JSON should parse")
}

fn run_python_audit(audit_script: &Path, repo_root: &Path) -> Output {
    for python in ["python", "python3"] {
        if let Ok(output) = Command::new(python)
            .arg(audit_script)
            .arg("--json")
            .arg("--repo-root")
            .arg(repo_root)
            .output()
        {
            return output;
        }
    }
    panic!("failed to launch python or python3 for editor structure audit");
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should have a repository parent")
        .to_path_buf()
}
