const PARENT_SOURCE: &str = include_str!("../rayon_boundary.rs");
const CUTOVER_STATUS_SOURCE: &str = include_str!("cutover_status.rs");
const PRODUCTION_SCAN_SOURCE: &str = include_str!("production_scan.rs");
const SUPPORT_SOURCE: &str = include_str!("support.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

const FRAMEWORKS_02_PLAN: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md");
const RUNTIME_15_PLAN: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
);
const RUNTIME_INDEX: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
const STRUCTURE_CONVENTION_PLAN: &str =
    include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
const REVIEW_FINDINGS_PLAN: &str =
    include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
const STATUS_ROW_DATA: &str = include_str!(
    "../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/runtime_structure_tests/runtime_absorption_core_rows.rs"
);
const STATUS_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps/core_route_rows.rs"
);
const DATE_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps/core_route_rows.rs"
);
#[rustfmt::skip]
const NUMBERED_STATUS_RECORDS: &str = concat!(
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/11/2026-07-09-job-system-task-model-output-records.md"),
    include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"),
    include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"),
    include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
    include_str!("../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md")
);

#[test]
fn runtime_15_rayon_boundary_route_owner_is_folder_backed() {
    assert_contains_all(
        "rayon_boundary route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"rayon_boundary/cutover_status.rs\"]",
            "#[path = \"rayon_boundary/production_scan.rs\"]",
            "#[path = \"rayon_boundary/split_layout.rs\"]",
            "#[path = \"rayon_boundary/support.rs\"]",
        ],
    );
    assert_parent_route_only();
    assert_child_owners_are_focused();
    assert_line_budget();
    assert_docs_and_status_mirror_split();
}

fn assert_parent_route_only() {
    assert!(
        !PARENT_SOURCE.contains("#[test]"),
        "rayon_boundary.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "collect_rayon_references",
        "classify_rayon_reference",
        "RayonReference",
        "runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "rayon_boundary.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owners_are_focused() {
    assert_contains_all(
        "production scan child",
        PRODUCTION_SCAN_SOURCE,
        &[
            "rayon_is_only_reachable_through_core_task_primitives",
            "rayon_boundary_guard_rejects_unclassified_runtime_source",
            "classify_rayon_reference",
            "collect_rayon_references",
        ],
    );
    assert_contains_all(
        "cutover status child",
        CUTOVER_STATUS_SOURCE,
        &[
            "rayon_render_exception_cutover_is_recorded_in_runtime_11_m2_1_status",
            "runtime_11_m2_1_graphics_frustum_rayon_cutover_static_passed_cargo_pending",
            "direct_rayon_paths = 2",
            "parallel_frustum.rs",
        ],
    );
    assert_contains_all(
        "support child",
        SUPPORT_SOURCE,
        &[
            "pub(super) struct RayonReference",
            "collect_rayon_references",
            "classify_rayon_reference",
            "rust_source_files",
        ],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 10),
        ("production scan child", PRODUCTION_SCAN_SOURCE, 70),
        ("cutover status child", CUTOVER_STATUS_SOURCE, 80),
        ("support child", SUPPORT_SOURCE, 90),
        ("split layout guard", SPLIT_LAYOUT_SOURCE, 210),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{label} has {line_count} lines; expected at most {max_lines}"
        );
    }
}

fn assert_docs_and_status_mirror_split() {
    for (label, source) in [
        ("Frameworks 02 plan", FRAMEWORKS_02_PLAN),
        ("Runtime 15 plan", RUNTIME_15_PLAN),
        ("runtime index", RUNTIME_INDEX),
        ("structure convention plan", STRUCTURE_CONVENTION_PLAN),
        ("review findings plan", REVIEW_FINDINGS_PLAN),
        ("module convention doc", MODULE_CONVENTION_DOC),
        ("status row data", STATUS_ROW_DATA),
        ("status map", STATUS_MAP),
    ] {
        assert!(
            source.contains(
                "runtime_15_rayon_boundary_route_owner_split_static_passed_cargo_deferred"
            ) || NUMBERED_STATUS_RECORDS.contains(
                "runtime_15_rayon_boundary_route_owner_split_static_passed_cargo_deferred"
            ),
            "{label} should mirror the rayon_boundary route-owner split status"
        );
    }
    assert!(
        DATE_MAP.contains("Runtime 15 M3 rayon-boundary route-owner split"),
        "date map should mirror the rayon_boundary route-owner split slice"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "rayon_boundary/production_scan.rs",
            "rayon_boundary/cutover_status.rs",
            "rayon_boundary/support.rs",
            "rayon_boundary/split_layout.rs",
            "runtime_15_rayon_boundary_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 plan",
        NUMBERED_STATUS_RECORDS,
        &[
            "frameworks_02_m3_rayon_boundary_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 rayon-boundary route-owner split",
        ],
    );
}

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(
            source.contains(anchor),
            "{label} should contain split anchor `{anchor}`"
        );
    }
}
