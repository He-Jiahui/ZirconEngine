pub(super) fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    assert_contains_all_exact(label, source, required);
}

pub(super) fn assert_contains_all_exact(label: &str, source: &str, required: &[&str]) {
    let missing: Vec<_> = required
        .iter()
        .copied()
        .filter(|anchor| !source.contains(anchor))
        .collect();
    assert!(
        missing.is_empty(),
        "{label} missing required anchors: {missing:?}"
    );
}

pub(super) fn runtime_src_path(relative: &str) -> std::path::PathBuf {
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        std::path::PathBuf::from(manifest_dir)
            .join("src")
            .join(relative)
    } else {
        std::path::PathBuf::from("zircon_runtime")
            .join("src")
            .join(relative)
    }
}

pub(super) fn repo_path(relative: &str) -> std::path::PathBuf {
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        std::path::PathBuf::from(manifest_dir)
            .parent()
            .expect("zircon_runtime manifest should live under repository root")
            .join(relative)
    } else {
        std::path::PathBuf::from(relative)
    }
}

pub(crate) fn priority_plan_doc_current_owner_archive_source() -> &'static str {
    static SOURCE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SOURCE.get_or_init(|| {
        [
            "docs/plans/zircon_runtime/runtime/15/2026-07-10-priority-plan-doc-current-owner-inventory.md",
            "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-inventory.md",
            "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-02.md",
            "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-03.md",
            "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-04.md",
            "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-05.md",
        ]
        .into_iter()
        .map(read_output_archive)
        .collect::<Vec<_>>()
        .join("\n")
    })
}

fn read_output_archive(relative: &str) -> String {
    let path = repo_path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}
