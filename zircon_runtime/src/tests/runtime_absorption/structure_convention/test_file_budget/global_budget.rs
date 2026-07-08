use super::*;
use std::path::{Path, PathBuf};

const TEST_FILE_LINE_BUDGET: usize = 800;

#[test]
fn runtime_15_no_oversized_test_files() {
    let runtime_src_root = runtime_src_root();
    let test_files = test_file_line_counts(&runtime_src_root);
    assert!(
        !test_files.is_empty(),
        "Runtime 15 test-file budget guard should scan at least one runtime test file"
    );

    let oversized: Vec<_> = test_files
        .iter()
        .filter_map(|(path, line_count)| {
            (*line_count >= TEST_FILE_LINE_BUDGET).then(|| (path.clone(), *line_count))
        })
        .collect();
    assert!(
        oversized.is_empty(),
        "Runtime 15 test files should stay below {TEST_FILE_LINE_BUDGET} lines; oversized files: {}",
        format_oversized_files(&oversized)
    );

    let parent =
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/mod.rs");
    assert_contains_all(
        "test-file budget parent mounts global budget guard",
        &parent,
        &["mod global_budget;"],
    );

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests.rs",
    );
    let status_map = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/asset_budget_maps.rs",
        ),
    ]
    .join("\n");
    let date_map = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/asset_budget_maps.rs",
        ),
    ]
    .join("\n");

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 no oversized test files global gate",
                "runtime_15_no_oversized_test_files_global_gate_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/global_budget.rs",
                "TEST_FILE_LINE_BUDGET",
                "runtime_15_no_oversized_test_files",
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status/date maps record no-oversized test files global gate",
        &format!("{status_map}\n{date_map}"),
        &[
            "Runtime 15 M3 no oversized test files global gate",
            "runtime_15_no_oversized_test_files_global_gate_static_passed_cargo_deferred",
            "2026-06-27",
        ],
    );
}

fn runtime_src_root() -> PathBuf {
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir).join("src")
    } else {
        PathBuf::from("zircon_runtime").join("src")
    }
}

fn test_file_line_counts(src_root: &Path) -> Vec<(String, usize)> {
    let mut files = Vec::new();
    collect_test_file_line_counts(src_root, src_root, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn collect_test_file_line_counts(root: &Path, directory: &Path, files: &mut Vec<(String, usize)>) {
    let entries = std::fs::read_dir(directory).unwrap_or_else(|error| {
        panic!("failed to read directory {}: {error}", directory.display())
    });
    let mut paths: Vec<_> = entries
        .map(|entry| {
            entry
                .unwrap_or_else(|error| {
                    panic!(
                        "failed to read directory entry under {}: {error}",
                        directory.display()
                    )
                })
                .path()
        })
        .collect();
    paths.sort();

    for path in paths {
        if path.is_dir() {
            collect_test_file_line_counts(root, &path, files);
        } else if is_test_rust_file(&path) {
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            files.push((relative_test_path(root, &path), source.lines().count()));
        }
    }
}

fn is_test_rust_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let is_rust = path.extension().and_then(|ext| ext.to_str()) == Some("rs");
    is_rust
        && (file_name == "tests.rs"
            || file_name.ends_with("_tests.rs")
            || path
                .components()
                .any(|component| component.as_os_str().to_string_lossy().as_ref() == "tests"))
}

fn relative_test_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn format_oversized_files(files: &[(String, usize)]) -> String {
    files
        .iter()
        .map(|(path, line_count)| format!("{path} ({line_count} lines)"))
        .collect::<Vec<_>>()
        .join(", ")
}
