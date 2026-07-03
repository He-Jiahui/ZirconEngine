use super::*;

#[test]
fn runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_frontmatter_entries_are_unique(label, path, &source);
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/integrity_guards.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("structure convention plan", structure_plan.as_str()),
        ("review findings plan", review_findings.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        (
            "priority-plan-doc status-output row data",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 priority plan docs frontmatter uniqueness guard",
                "runtime_15_priority_plan_docs_frontmatter_uniqueness_guard_static_passed_cargo_deferred",
                "docs/plans/engine-code-structure-convention.md",
                "docs/plans/engine-code-review-findings-2026-06.md",
                "structure_convention/test_file_budget/priority_plan_docs/frontmatter_uniqueness.rs",
                "runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries",
                "duplicate-free frontmatter",
                "related_code",
                "implementation_files",
                "plan_sources",
                "tests",
                "Cargo gate deferred",
            ],
        );
    }

    assert_contains_all(
        "status expected-slice map",
        &status_map,
        &[
            "Runtime 15 M3 priority plan docs frontmatter uniqueness guard",
            "runtime_15_priority_plan_docs_frontmatter_uniqueness_guard_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "date expected-slice map",
        &date_map,
        &[
            "Runtime 15 M3 priority plan docs frontmatter uniqueness guard",
            "2026-07-03",
        ],
    );
}

fn assert_priority_plan_doc_frontmatter_entries_are_unique(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);

    for section in [
        "related_code",
        "implementation_files",
        "plan_sources",
        "tests",
    ] {
        let entries = frontmatter_section_items(&frontmatter, section);
        let mut seen = std::collections::BTreeSet::new();
        let mut duplicates = Vec::new();

        for entry in entries {
            if !seen.insert(entry) {
                duplicates.push(entry);
            }
        }

        assert!(
            duplicates.is_empty(),
            "{label} priority plan doc `{path}` should keep `{section}:` duplicate-free; duplicates: {:?}",
            duplicates
        );
    }
}
