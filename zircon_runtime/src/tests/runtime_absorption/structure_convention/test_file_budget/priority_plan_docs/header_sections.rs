use super::*;

#[test]
fn runtime_15_priority_plan_docs_required_header_sections_stay_complete() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_required_header_sections(label, path, &source);
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
                "Runtime 15 M3 priority plan docs required header sections guard",
                "runtime_15_priority_plan_docs_required_header_sections_guard_static_passed_cargo_deferred",
                "docs/plans/engine-code-structure-convention.md",
                "docs/plans/engine-code-review-findings-2026-06.md",
                "structure_convention/test_file_budget/priority_plan_docs/header_sections.rs",
                "runtime_15_priority_plan_docs_required_header_sections_stay_complete",
                "related_code:",
                "implementation_files:",
                "plan_sources:",
                "tests:",
                "doc_type:",
                "status:",
                "Cargo gate deferred",
            ],
        );
    }

    assert_contains_all(
        "status expected-slice map",
        &status_map,
        &[
            "Runtime 15 M3 priority plan docs required header sections guard",
            "runtime_15_priority_plan_docs_required_header_sections_guard_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "date expected-slice map",
        &date_map,
        &[
            "Runtime 15 M3 priority plan docs required header sections guard",
            "2026-07-01",
        ],
    );
}

fn assert_priority_plan_doc_required_header_sections(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);
    let required_order = [
        "related_code",
        "implementation_files",
        "plan_sources",
        "tests",
        "doc_type",
        "status",
    ];
    let top_level_keys: Vec<&str> = frontmatter
        .iter()
        .filter(|line| !line.starts_with(' '))
        .filter_map(|line| line.split_once(':').map(|(key, _)| key))
        .collect();

    for key in required_order {
        assert!(
            top_level_keys.contains(&key),
            "{label} priority plan doc `{path}` should keep `{key}:` in frontmatter"
        );
    }

    let required_positions: Vec<usize> = required_order
        .iter()
        .map(|key| {
            top_level_keys
                .iter()
                .position(|candidate| candidate == key)
                .unwrap_or_else(|| {
                    panic!("{label} priority plan doc `{path}` missing required key `{key}:`")
                })
        })
        .collect();
    assert!(
        required_positions.windows(2).all(|window| window[0] < window[1]),
        "{label} priority plan doc `{path}` should keep required frontmatter keys in docs lookup order: {:?}",
        required_order
    );

    for section in [
        "related_code",
        "implementation_files",
        "plan_sources",
        "tests",
    ] {
        let item_count = frontmatter_section_items(&frontmatter, section).len();
        assert!(
            item_count > 0,
            "{label} priority plan doc `{path}` should keep non-empty `{section}:` frontmatter"
        );
    }

    for scalar in ["doc_type", "status"] {
        let value = frontmatter_scalar(&frontmatter, scalar);
        assert!(
            value.is_some(),
            "{label} priority plan doc `{path}` should keep non-empty `{scalar}:` frontmatter"
        );
    }
}
