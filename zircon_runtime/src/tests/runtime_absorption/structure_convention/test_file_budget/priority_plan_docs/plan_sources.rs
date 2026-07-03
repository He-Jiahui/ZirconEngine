use super::*;

#[test]
fn runtime_15_priority_plan_docs_plan_sources_stay_cross_linked() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_plan_sources_are_cross_linked(label, path, &source);
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
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 priority plan docs plan-source cross-link guard",
                "runtime_15_priority_plan_docs_plan_source_cross_link_guard_static_passed_cargo_deferred",
                "docs/plans/engine-code-structure-convention.md",
                "docs/plans/engine-code-review-findings-2026-06.md",
                "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
                "structure_convention/test_file_budget/priority_plan_docs/plan_sources.rs",
                "runtime_15_priority_plan_docs_plan_sources_stay_cross_linked",
                "plan_sources:",
                "user:",
                "Cargo gate deferred",
            ],
        );
    }

    assert_contains_all(
        "status expected-slice map",
        &status_map,
        &[
            "Runtime 15 M3 priority plan docs plan-source cross-link guard",
            "runtime_15_priority_plan_docs_plan_source_cross_link_guard_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "date expected-slice map",
        &date_map,
        &[
            "Runtime 15 M3 priority plan docs plan-source cross-link guard",
            "2026-07-01",
        ],
    );
}

fn assert_priority_plan_doc_plan_sources_are_cross_linked(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);
    let plan_sources = frontmatter_section_items(&frontmatter, "plan_sources");
    assert!(
        plan_sources.iter().any(|item| item.starts_with("user:")),
        "{label} priority plan doc `{path}` should keep a user-request plan source"
    );
    assert!(
        plan_sources
            .iter()
            .any(|item| *item == "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"),
        "{label} priority plan doc `{path}` should cite Runtime 15 as its implementation-status source"
    );

    let required_companion = match path {
        "docs/plans/engine-code-structure-convention.md" => {
            "docs/plans/engine-code-review-findings-2026-06.md"
        }
        "docs/plans/engine-code-review-findings-2026-06.md" => {
            "docs/plans/engine-code-structure-convention.md"
        }
        _ => panic!("unexpected priority plan doc path `{path}`"),
    };
    assert!(
        plan_sources.iter().any(|item| *item == required_companion),
        "{label} priority plan doc `{path}` should cite companion priority plan source `{required_companion}`"
    );
}
