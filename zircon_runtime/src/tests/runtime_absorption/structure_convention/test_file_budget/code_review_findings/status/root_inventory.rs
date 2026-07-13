use super::*;

#[test]
fn runtime_15_code_review_findings_status_docs_root_inventory_is_child_owned() {
    let parent = read_runtime_src(STATUS_DOC_PARENT_PATH);
    let status_rows = review_guard_status_rows_source();
    let status_map = review_guard_status_map_source();
    let date_map = review_guard_date_map_source();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (module_name, child_path, anchor) in STATUS_DOC_ROOT_CHILDREN {
        let path_attr = format!("#[path = \"status/{module_name}.rs\"]");
        assert_contains_all(
            "status-doc parent mounts root-inventory child",
            &parent,
            &[path_attr.as_str(), *module_name],
        );

        let child_source = read_runtime_src(child_path);
        assert_contains_all(child_path, &child_source, &[*anchor]);
        assert!(
            child_source.lines().count() < STATUS_DOC_ROOT_CHILD_LINE_BUDGET,
            "{child_path} should stay below its root inventory child budget"
        );
    }

    for forbidden in [
        "pub(super) const STATUS_DOC_PARENT_PATH:",
        "pub(super) const STATUS_DOC_SOURCE_ANCHORS_OWNER:",
        "pub(super) const REVIEW_GUARD_STATUS_ROWS_PATH:",
        "const REVIEW_GUARD_STATUS_ROW_SOURCE_PATHS:",
        "pub(super) const STATUS_DOC_SOURCE_ANCHORS_SLICE:",
        "pub(super) const STATUS_DOC_CHILDREN:",
        "pub(super) fn status_doc_child_sources",
        "pub(super) fn status_doc_child_source_blob",
    ] {
        assert!(
            !parent.contains(forbidden),
            "status-doc parent should delegate root inventory anchor `{forbidden}`"
        );
    }

    let status_anchors = [
        STATUS_DOC_ROOT_INVENTORY_SLICE,
        STATUS_DOC_ROOT_INVENTORY_STATUS,
        STATUS_DOC_PARENT_PATH,
        STATUS_DOC_ROOT_PATHS_CHILD,
        STATUS_DOC_ROOT_STATUSES_CHILD,
        STATUS_DOC_ROOT_ROW_SOURCES_CHILD,
        STATUS_DOC_ROOT_CHILDREN_CHILD,
        STATUS_DOC_ROOT_INVENTORY_CHILD,
        STATUS_DOC_ROOT_INVENTORY_GUARD,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("status-doc row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 review status map records status-doc root inventory split",
        &status_map,
        &[
            STATUS_DOC_ROOT_INVENTORY_SLICE,
            STATUS_DOC_ROOT_INVENTORY_STATUS,
        ],
    );
    assert_contains_all(
        "M3 review date map records status-doc root inventory split",
        &date_map,
        &[
            STATUS_DOC_ROOT_INVENTORY_SLICE,
            STATUS_DOC_ROOT_INVENTORY_DATE,
        ],
    );
}
