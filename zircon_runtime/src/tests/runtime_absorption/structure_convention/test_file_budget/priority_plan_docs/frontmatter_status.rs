use super::*;

#[test]
fn runtime_15_priority_plan_docs_frontmatter_status_stays_current() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_frontmatter_status(label, path, &source);
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
    let status_map = priority_plan_doc_status_map_source();
    let date_map = priority_plan_doc_date_map_source();

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
                "Runtime 15 M3 priority plan docs frontmatter status guard",
                "runtime_15_priority_plan_docs_frontmatter_status_guard_static_passed_cargo_deferred",
                "docs/plans/engine-code-structure-convention.md",
                "docs/plans/engine-code-review-findings-2026-06.md",
                "structure_convention/test_file_budget/priority_plan_docs/frontmatter_status.rs",
                "runtime_15_priority_plan_docs_frontmatter_status_stays_current",
                "doc_type: convention-authority",
                "doc_type: review-findings",
                "status: in_progress",
                "Cargo gate deferred",
            ],
        );
    }

    assert_contains_all(
        "status expected-slice map",
        &status_map,
        &[
            "Runtime 15 M3 priority plan docs frontmatter status guard",
            "runtime_15_priority_plan_docs_frontmatter_status_guard_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "date expected-slice map",
        &date_map,
        &[
            "Runtime 15 M3 priority plan docs frontmatter status guard",
            "2026-07-01",
        ],
    );
}

fn assert_priority_plan_doc_frontmatter_status(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);
    let doc_type = frontmatter_scalar(&frontmatter, "doc_type")
        .unwrap_or_else(|| panic!("{label} priority plan doc `{path}` should declare doc_type"));
    let status = frontmatter_scalar(&frontmatter, "status")
        .unwrap_or_else(|| panic!("{label} priority plan doc `{path}` should declare status"));

    let expected_doc_type = match path {
        "docs/plans/engine-code-structure-convention.md" => "convention-authority",
        "docs/plans/engine-code-review-findings-2026-06.md" => "review-findings",
        _ => panic!("unexpected priority plan doc path `{path}`"),
    };
    assert_eq!(
        doc_type, expected_doc_type,
        "{label} priority plan doc `{path}` should keep doc_type `{expected_doc_type}`"
    );
    assert_eq!(
        status, "in_progress",
        "{label} priority plan doc `{path}` should stay in_progress while Runtime 15 records Cargo-deferred implementation slices"
    );
}
