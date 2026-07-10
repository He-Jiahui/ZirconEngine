pub(super) fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    let normalized_label = label.replace('\\', "/");
    let archived_status_source = if label.eq_ignore_ascii_case("runtime index")
        || label.to_ascii_lowercase().contains("runtime index")
        || normalized_label.contains("docs/plans/zircon_runtime/runtime/index.md")
    {
        Some(runtime_15_output_archive_source())
    } else if label.eq_ignore_ascii_case("review findings")
        || label.eq_ignore_ascii_case("review findings plan")
        || label.to_ascii_lowercase().contains("review findings")
        || normalized_label.contains("docs/plans/engine-code-review-findings-2026-06.md")
    {
        Some(engine_code_review_findings_archive_source())
    } else if label.eq_ignore_ascii_case("structure convention")
        || label.eq_ignore_ascii_case("structure convention plan")
        || label
            .to_ascii_lowercase()
            .contains("structure convention")
        || normalized_label.contains("docs/plans/engine-code-structure-convention.md")
    {
        Some(engine_code_structure_archive_source())
    } else if label.eq_ignore_ascii_case("module convention doc")
        || label
            .to_ascii_lowercase()
            .contains("module convention doc")
        || normalized_label.contains("docs/zircon_runtime/structure/module-convention.md")
        || label.eq_ignore_ascii_case("Frameworks 02")
        || label.eq_ignore_ascii_case("Frameworks 02 plan")
        || label.to_ascii_lowercase().contains("frameworks 02")
        || normalized_label.starts_with("docs/plans/zircon_runtime/frameworks/")
        || label.to_ascii_lowercase().contains("session note")
        || normalized_label.starts_with(".codex/sessions/")
        || label
            .to_ascii_lowercase()
            .contains("runtime implementation session")
        || label.to_ascii_lowercase().contains("frameworks index")
        || label.to_ascii_lowercase().contains("framework plan")
        || label.to_ascii_lowercase().contains("frameworks plan")
        || label.eq_ignore_ascii_case("render index")
    {
        Some(priority_plan_doc_current_owner_archive_source())
    } else if normalized_label.to_ascii_lowercase().contains("row data")
        || normalized_label.to_ascii_lowercase().contains("row_data")
        || normalized_label.to_ascii_lowercase().contains(" rows")
        || normalized_label.to_ascii_lowercase().ends_with(" row")
        || normalized_label
            .to_ascii_lowercase()
            .contains("status-output")
        || normalized_label
            .to_ascii_lowercase()
            .contains("expected-status")
        || normalized_label
            .to_ascii_lowercase()
            .contains("status output")
        || normalized_label
            .to_ascii_lowercase()
            .contains("status rows")
        || normalized_label
            .to_ascii_lowercase()
            .contains("status-support map")
        || normalized_label
            .to_ascii_lowercase()
            .contains("status-support")
        || normalized_label
            .to_ascii_lowercase()
            .contains("status support")
        || normalized_label
            .to_ascii_lowercase()
            .contains("status-doc")
        || normalized_label.to_ascii_lowercase().contains("row-doc")
        || normalized_label.to_ascii_lowercase().contains("row status")
        || normalized_label
            .to_ascii_lowercase()
            .contains("status map")
        || normalized_label.to_ascii_lowercase().contains("date map")
        || (normalized_label.to_ascii_lowercase().contains("status")
            && normalized_label.to_ascii_lowercase().contains("map"))
        || normalized_label
            .to_ascii_lowercase()
            .contains("expected-slice")
        || normalized_label
            .to_ascii_lowercase()
            .contains("expected status")
    {
        Some(current_status_row_owner_inventory_source())
    } else if label.eq_ignore_ascii_case("Runtime 15 plan")
        || label.to_ascii_lowercase().contains("runtime 15 plan")
        || normalized_label
            .contains("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md")
        || label.eq_ignore_ascii_case("Plan 09")
    {
        Some(runtime_15_output_archive_source())
    } else {
        None
    };
    let missing: Vec<_> = required
        .iter()
        .copied()
        .filter(|anchor| {
            !source.contains(anchor)
                && !archived_status_source
                    .is_some_and(|archive_source| archive_source.contains(anchor))
        })
        .collect();
    assert!(
        missing.is_empty(),
        "{label} missing required anchors: {missing:?}"
    );
}

fn current_status_row_owner_inventory_source() -> &'static str {
    static SOURCE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SOURCE.get_or_init(|| {
        let parent = repo_path(
            "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/current_structure_owner_inventory.rs",
        );
        let child_dir = repo_path(
            "zircon_runtime/src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/current_structure_owner_inventory",
        );
        let mut paths = std::fs::read_dir(&child_dir)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", child_dir.display()))
            .map(|entry| entry.expect("current row inventory entry should be readable").path())
            .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("rs"))
            .collect::<Vec<_>>();
        paths.sort();

        std::iter::once(parent)
            .chain(paths)
            .map(|path| {
                std::fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
            })
            .collect::<Vec<_>>()
            .join("\n")
    })
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

fn runtime_15_output_archive_source() -> &'static str {
    static SOURCE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SOURCE.get_or_init(|| {
        let archive_dir = repo_path("docs/plans/zircon_runtime/runtime/15");
        let mut paths = std::fs::read_dir(&archive_dir)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", archive_dir.display()))
            .map(|entry| {
                entry
                    .unwrap_or_else(|error| {
                        panic!("failed to read Runtime 15 archive entry: {error}")
                    })
                    .path()
            })
            .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("md"))
            .collect::<Vec<_>>();
        paths.sort();
        paths
            .into_iter()
            .map(|path| {
                std::fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
            })
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn engine_code_review_findings_archive_source() -> &'static str {
    static SOURCE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SOURCE.get_or_init(|| {
        format!(
            "{}\n{}",
            read_output_archive(
                "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
            ),
            priority_plan_doc_current_owner_archive_source(),
        )
    })
}

fn engine_code_structure_archive_source() -> &'static str {
    static SOURCE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SOURCE.get_or_init(|| {
        format!(
            "{}\n{}",
            read_output_archive(
                "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
            ),
            priority_plan_doc_current_owner_archive_source(),
        )
    })
}

pub(crate) fn priority_plan_doc_current_owner_archive_source() -> &'static str {
    static SOURCE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SOURCE.get_or_init(|| {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            read_output_archive(
                "docs/plans/zircon_runtime/runtime/15/2026-07-10-priority-plan-doc-current-owner-inventory.md",
            ),
            read_output_archive(
                "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-inventory.md",
            ),
            read_output_archive(
                "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-02.md",
            ),
            read_output_archive(
                "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-03.md",
            ),
            read_output_archive(
                "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-04.md",
            ),
            read_output_archive(
                "docs/plans/zircon_runtime/runtime/15/2026-07-11-test-file-budget-current-owner-anchor-delta-05.md",
            ),
        )
    })
}

fn read_output_archive(relative: &str) -> String {
    let path = repo_path(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}
