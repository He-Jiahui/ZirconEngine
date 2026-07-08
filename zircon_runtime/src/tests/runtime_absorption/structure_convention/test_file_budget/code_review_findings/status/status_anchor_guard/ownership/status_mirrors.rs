use super::*;

fn status_anchor_child_ownership_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}

#[test]
fn runtime_15_code_review_findings_status_docs_status_anchor_child_ownership_is_child_backed() {
    let parent = read_runtime_src(STATUS_DOC_STATUS_ANCHOR_GUARD_CHILD_OWNERSHIP_CHILD);
    let child_blob = status_anchor_child_ownership_child_source_blob();
    let status_rows = review_guard_status_rows_source();
    let status_map = review_guard_status_map_source();
    let date_map = review_guard_date_map_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (module_name, child_path, anchor) in STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_CHILDREN {
        let path_attr = format!("#[path = \"ownership/{module_name}.rs\"]");
        assert_contains_all(
            "status-anchor child-ownership route mounts child",
            &parent,
            &[path_attr.as_str(), *module_name],
        );
        assert_contains_all(child_path, &read_runtime_src(child_path), &[*anchor]);
    }
    for forbidden in [
        "fn assert_status_doc_mounts_status_anchor_child_owner",
        "const MOVED_STATUS_ANCHORS",
        "fn assert_status_anchor_route_forwards_children",
        "fn assert_status_anchor_line_budgets",
    ] {
        assert!(
            !parent.contains(forbidden),
            "child_ownership.rs should delegate `{forbidden}` to child modules"
        );
    }
    let status_anchors = [
        STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_SPLIT_NAME,
        STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_SPLIT_ID,
        STATUS_DOC_STATUS_ANCHOR_GUARD_CHILD_OWNERSHIP_CHILD,
        STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_SPLIT_GUARD,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("status row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime implementation session", session_note.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
        for (_, child_path, _) in STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_CHILDREN {
            assert!(
                source.contains(child_path),
                "{label} should mirror status-anchor child-ownership child {child_path}"
            );
        }
    }
    assert_contains_all(
        "review-guard status map records status-anchor child-ownership child split",
        &status_map,
        &[
            STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_SPLIT_NAME,
            STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_SPLIT_ID,
        ],
    );
    assert_contains_all(
        "review-guard date map records status-anchor child-ownership child split",
        &date_map,
        &[
            STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_SPLIT_NAME,
            STATUS_DOC_STATUS_ANCHOR_CHILD_OWNERSHIP_SPLIT_DATE,
        ],
    );
    assert!(child_blob.contains("assert_moved_status_anchors_stay_child_owned"));
}
