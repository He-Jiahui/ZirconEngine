use super::*;

#[test]
fn runtime_15_priority_plan_docs_test_paths_stay_current() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_test_paths_exist(label, path, &source);
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/integrity_guards.rs",
    );
    let status_map = priority_plan_doc_status_map_source();
    let date_map = priority_plan_doc_date_map_source();

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("structure convention plan", structure_plan.as_str()),
        ("review findings plan", review_findings.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 priority plan docs test-path integrity guard",
                "runtime_15_priority_plan_docs_test_path_integrity_guard_static_passed_cargo_deferred",
                "docs/plans/engine-code-structure-convention.md",
                "docs/plans/engine-code-review-findings-2026-06.md",
                "structure_convention/test_file_budget/priority_plan_docs/test_paths.rs",
                "runtime_15_priority_plan_docs_test_paths_stay_current",
                "tests:",
                "Cargo gate deferred",
            ],
        );
    }

    assert_contains_all(
        "status expected-slice map",
        &status_map,
        &[
            "Runtime 15 M3 priority plan docs test-path integrity guard",
            "runtime_15_priority_plan_docs_test_path_integrity_guard_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "date expected-slice map",
        &date_map,
        &[
            "Runtime 15 M3 priority plan docs test-path integrity guard",
            "2026-07-01",
        ],
    );
}

fn assert_priority_plan_doc_test_paths_exist(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);
    let mut checked_paths = 0usize;
    let mut active_section_is_tests = false;

    for line in frontmatter {
        if !line.starts_with(' ') {
            active_section_is_tests = line
                .split_once(':')
                .map(|(section, _)| section == "tests")
                .unwrap_or(false);
            continue;
        }

        if !active_section_is_tests {
            continue;
        }

        let Some(item) = line.strip_prefix("  - ") else {
            continue;
        };
        let candidate = item
            .split("::")
            .next()
            .unwrap_or(item)
            .split(": ")
            .next()
            .unwrap_or(item)
            .trim();
        if !is_repo_path_like_test_entry(candidate) {
            continue;
        }

        assert!(
            repo_path(candidate).exists(),
            "{label} frontmatter `tests` entry should resolve to an existing repository path: {candidate}"
        );
        checked_paths += 1;
    }

    assert!(
        checked_paths > 0,
        "{label} priority plan doc `{path}` should expose at least one path-like tests entry"
    );
}

fn is_repo_path_like_test_entry(candidate: &str) -> bool {
    candidate.starts_with("zircon_")
        || candidate.starts_with("tools/")
        || candidate.starts_with("tools\\")
        || candidate.starts_with("docs/")
        || candidate.starts_with("docs\\")
        || candidate.starts_with(".github/")
        || candidate.starts_with(".github\\")
        || candidate.starts_with(".codex/")
        || candidate.starts_with(".codex\\")
}
