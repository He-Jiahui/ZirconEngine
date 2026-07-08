use super::*;

#[test]
fn runtime_15_foundation_row_data_priority_doc_frontmatter_records_stale_count_guard() {
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/status_followups.rs",
    );
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    for (label, source) in [
        ("structure convention", structure_convention.as_str()),
        ("review findings", review_findings.as_str()),
    ] {
        let frontmatter = priority_doc_frontmatter(label, source);
        assert_contains_all(
            label,
            &frontmatter,
            &[
                ROW_COUNT_CHILD_FRONTMATTER_PATH,
                STATUS_SUPPORT_ROW_DATA_AND_BUDGET_FRONTMATTER_PATH,
                STATUS_SUPPORT_STATUS_MAP_FRONTMATTER_PATH,
                STATUS_SUPPORT_DATE_MAP_FRONTMATTER_PATH,
                "related_code:",
                "implementation_files:",
                "tests:",
                STALE_COUNT_PROSE_GUARD_NAME,
                STALE_COUNT_PROSE_GUARD_ID,
                PRIORITY_DOC_FRONTMATTER_SYNC_NAME,
                PRIORITY_DOC_FRONTMATTER_SYNC_ID,
                "runtime_15_foundation_row_data_docs_record_current_row_count",
                "runtime_15_foundation_row_data_priority_doc_frontmatter_records_stale_count_guard",
            ],
        );
    }

    for (label, source) in [
        ("structure convention", structure_convention.as_str()),
        ("review findings", review_findings.as_str()),
        ("priority-plan-doc status rows", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                STALE_COUNT_PROSE_GUARD_NAME,
                STALE_COUNT_PROSE_GUARD_ID,
                PRIORITY_DOC_FRONTMATTER_SYNC_NAME,
                PRIORITY_DOC_FRONTMATTER_SYNC_ID,
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status map",
        &status_map,
        &[
            PRIORITY_DOC_FRONTMATTER_SYNC_NAME,
            PRIORITY_DOC_FRONTMATTER_SYNC_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 date map",
        &date_map,
        &[PRIORITY_DOC_FRONTMATTER_SYNC_NAME, "2026-07-03"],
    );
}

fn priority_doc_frontmatter(label: &str, source: &str) -> String {
    let mut lines = source.lines();
    assert_eq!(
        lines.next(),
        Some("---"),
        "{label} priority plan doc should start with YAML frontmatter"
    );

    let mut frontmatter = String::new();
    for line in lines {
        if line == "---" {
            return frontmatter;
        }
        frontmatter.push_str(line);
        frontmatter.push('\n');
    }

    panic!("{label} priority plan doc should close YAML frontmatter");
}
