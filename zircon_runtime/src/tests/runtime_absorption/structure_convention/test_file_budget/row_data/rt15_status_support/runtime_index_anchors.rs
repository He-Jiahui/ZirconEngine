use super::*;

const RUNTIME_INDEX_ANCHOR_CHILD_SPLIT_STATUS_NAME: &str =
    "Runtime 15 M3 status-support runtime-index anchor row-data child split";
const RUNTIME_INDEX_ANCHOR_CHILD_SPLIT_STATUS_ID: &str =
    "runtime_15_status_support_runtime_index_anchor_row_data_child_split_static_passed_cargo_deferred";
const RUNTIME_INDEX_ANCHOR_CHILD_SPLIT_GUARD_NAME: &str =
    "runtime_15_status_support_runtime_index_anchor_rows_are_child_owned";

#[test]
fn runtime_15_status_support_runtime_index_anchor_rows_are_child_owned() {
    let parent = read_runtime_src(STATUS_SUPPORT_RUNTIME_INDEX_ANCHORS_PATH);
    let child_rows = RUNTIME_INDEX_ANCHOR_CHILDREN
        .iter()
        .map(|(_, path, _)| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "status-support runtime-index anchor parent mounts focused children",
        &parent,
        &[
            "mod cargo_attempt;",
            "mod index_baseline;",
            "mod plan_status_children;",
            "mod runtime_status_anchors;",
            "mod support_inventory;",
            "index_baseline::SUBPLAN_MAP_SYNC",
            "runtime_status_anchors::RUNTIME_07_SCENE_ASSET_STATUS_ANCHOR_SYNC",
            "cargo_attempt::RUNTIME_CARGO_ATTEMPT_STATUS_ANCHOR_SYNC",
            "plan_status_children::INDEX_TABLES_CHILD_OWNER_SPLIT",
            "support_inventory::SUPPORT_INVENTORY_REVIEW_SYNC",
        ],
    );
    for forbidden in [
        "runtime_15_runtime_index_subplan_map_01_15_sync_static_passed_cargo_deferred",
        "Runtime 07 scene asset owner split",
        "cargo_recheck_timeout_no_result",
        "runtime_absorption/code_review_findings.rs",
    ] {
        assert!(
            !parent.contains(forbidden),
            "runtime_index_anchors.rs should route row anchors instead of owning `{forbidden}`"
        );
    }
    for (module_name, child_path, representative_row) in RUNTIME_INDEX_ANCHOR_CHILDREN {
        let module_mount = format!("mod {module_name};");
        assert!(
            parent.contains(&module_mount),
            "runtime-index anchor parent should mount {module_mount}"
        );
        let child_source = read_runtime_src(child_path);
        assert_contains_all(
            child_path,
            &child_source,
            &[*representative_row, "type Slice = super::Slice;"],
        );
        assert!(
            child_rows.contains(representative_row),
            "runtime-index anchor child rows should include {representative_row}"
        );
    }
}

#[test]
fn runtime_15_status_support_runtime_index_anchor_child_split_status_is_current() {
    let status_row_data = EXPECTED_SLICE_STATUS_SUPPORT_MAPS_CHILDREN
        .iter()
        .map(|path| read_runtime_src(path))
        .collect::<Vec<_>>()
        .join("\n");
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/status_support_map_rows.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps/status_support_map_rows.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("status row data", status_row_data.as_str()),
        ("status map", status_map.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("engine structure convention", structure_convention.as_str()),
        ("review findings", review_findings.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                RUNTIME_INDEX_ANCHOR_CHILD_SPLIT_STATUS_NAME,
                RUNTIME_INDEX_ANCHOR_CHILD_SPLIT_STATUS_ID,
                RUNTIME_INDEX_ANCHOR_CHILD_SPLIT_GUARD_NAME,
            ],
        );
    }
    assert_contains_all(
        "date map records current runtime-index anchor child split date",
        &date_map,
        &[
            RUNTIME_INDEX_ANCHOR_CHILD_SPLIT_STATUS_NAME,
            "Some(\"2026-07-05\")",
        ],
    );
}
