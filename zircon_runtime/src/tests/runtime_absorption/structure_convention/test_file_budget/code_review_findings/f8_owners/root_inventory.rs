use super::*;

#[test]
fn runtime_15_f8_child_owner_root_inventory_is_child_owned() {
    let parent = read_runtime_src(STRUCTURE_GUARD_OWNER);
    let status_rows = f8_status_row_source();
    let status_map = format!(
        "{}\n{}",
        read_runtime_src(REVIEW_GUARD_STATUS_MAP),
        read_runtime_src(REVIEW_GUARD_F8_STATUS_MAP)
    );
    let date_map = format!(
        "{}\n{}",
        read_runtime_src(REVIEW_GUARD_DATE_MAP),
        read_runtime_src(REVIEW_GUARD_F8_DATE_MAP)
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (module_name, child_path, anchor) in F8_ROOT_CHILDREN {
        let path_attr = format!("#[path = \"f8_owners/{module_name}.rs\"]");
        assert_contains_all(
            "F8 structure guard parent mounts root-inventory child",
            &parent,
            &[path_attr.as_str(), *module_name],
        );

        let child_source = read_runtime_src(child_path);
        assert_contains_all(child_path, &child_source, &[*anchor]);
        assert!(
            child_source.lines().count() < F8_ROOT_CHILD_LINE_BUDGET,
            "{child_path} should stay below its root inventory child budget"
        );
    }

    for forbidden in [
        "pub(super) struct F8ReviewSources",
        "pub(super) const STRUCTURE_GUARD_OWNER:",
        "pub(super) const REVIEW_GUARDS:",
        "pub(super) const FOLDER_BACKED_CHILDREN:",
        "pub(super) fn read_f8_review_sources",
        "pub(super) fn folder_backed_child_sources",
    ] {
        assert!(
            !parent.contains(forbidden),
            "F8 structure guard parent should delegate root inventory anchor `{forbidden}`"
        );
    }

    let status_anchors = [
        F8_ROOT_INVENTORY_SLICE,
        F8_ROOT_INVENTORY_STATUS,
        STRUCTURE_GUARD_OWNER,
        F8_ROOT_PATHS_CHILD,
        F8_ROOT_STATUSES_CHILD,
        F8_ROOT_CHILD_ROWS_CHILD,
        F8_ROOT_SOURCES_CHILD,
        F8_ROOT_INVENTORY_CHILD,
        F8_ROOT_INVENTORY_GUARD,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("F8 row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime architecture session note", session_note.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 review status map records F8 child-owner root inventory split",
        &status_map,
        &[F8_ROOT_INVENTORY_SLICE, F8_ROOT_INVENTORY_STATUS],
    );
    assert_contains_all(
        "M3 review date map records F8 child-owner root inventory split",
        &date_map,
        &[F8_ROOT_INVENTORY_SLICE, F8_ROOT_INVENTORY_DATE],
    );
}
