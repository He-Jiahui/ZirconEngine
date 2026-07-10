use super::*;

const STATUS_NAME: &str = "Runtime 15 M3 module-convention status row-data owner child split";
const STATUS_ID: &str =
    "runtime_15_module_convention_status_row_data_owner_child_split_static_passed_cargo_deferred";
const GUARD_NAME: &str = "runtime_15_module_convention_status_row_data_owner_is_child_backed";
const STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/lock_poison_module_maps.rs";
const DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/lock_poison_module_maps.rs";

pub(super) fn assert_module_convention_status_status_mirrors_are_current() {
    let status_rows = read_runtime_src(MODULE_CONVENTION_STATUS_ROW_DATA_OWNER_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_MAP_PATH);
    let date_map = read_runtime_src(DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_plan = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    let status_anchors = [
        STATUS_NAME,
        STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status/status_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status/frontmatter_and_gate_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status/structure_guard_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status/audit_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status/row_data_owner.rs",
        "MODULE_CONVENTION_STATUS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        "RUNTIME_15_M3_MODULE_CONVENTION_STATUS_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02 plan", frameworks_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("module-convention status rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all("status map", &status_map, &[STATUS_NAME, STATUS_ID]);
    assert_contains_all("date map", &date_map, &[STATUS_NAME, "2026-07-07"]);
}
