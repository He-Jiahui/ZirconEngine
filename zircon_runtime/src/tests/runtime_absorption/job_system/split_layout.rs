const PARENT_SOURCE: &str = include_str!("../job_system.rs");
const INVENTORY_SOURCE: &str = include_str!("inventory.rs");
const MIRROR_DOCS_SOURCE: &str = include_str!("mirror_docs.rs");
const SOURCE_HELPERS_SOURCE: &str = include_str!("source_helpers.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

const RUNTIME_11_PLAN: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/11-job-system-task-model.md");
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
const FRAMEWORKS_02_PLAN: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md");
const SESSION_NOTE: &str = include_str!(
    "../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
);
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
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md"),
    include_str!("../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md")
);

#[test]
fn runtime_15_job_system_route_owner_is_folder_backed() {
    assert_contains_all(
        "parent route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"job_system/inventory.rs\"]",
            "#[path = \"job_system/mirror_docs.rs\"]",
            "#[path = \"job_system/source_helpers.rs\"]",
            "#[path = \"job_system/split_layout.rs\"]",
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
        "job_system.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "JOB_SYSTEM_MODULE_MAX_LINES",
        "EXPECTED_JOB_SYSTEM_MODULES",
        "fn runtime_11_job_system_mirror_docs_match_structure_audit_counts",
        "fn collect_direct_rayon_paths",
        "fn rust_source_files",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "job_system.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owners_are_focused() {
    assert_contains_all(
        "inventory child",
        INVENTORY_SOURCE,
        &[
            "EXPECTED_JOB_SYSTEM_MODULES",
            "EXPECTED_DIRECT_RAYON_PATHS",
            "TASKS_MOD_DECLARATIONS",
            "BEHAVIOR_TEST_ANCHORS",
            "MIRROR_DOC_ANCHORS",
        ],
    );
    assert_contains_all(
        "mirror docs child",
        MIRROR_DOCS_SOURCE,
        &[
            "runtime_11_job_system_mirror_docs_match_structure_audit_counts",
            "collect_direct_rayon_paths(&runtime_root.join(\"src\"))",
            "assert_contains_all(doc_name, doc_source, MIRROR_DOC_ANCHORS)",
        ],
    );
    assert_contains_all(
        "source helpers child",
        SOURCE_HELPERS_SOURCE,
        &[
            "pub(super) fn collect_direct_rayon_paths",
            "fn rust_source_files",
            "fn line_mentions_rayon",
            "fn relative_path",
        ],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 16),
        ("inventory child", INVENTORY_SOURCE, 140),
        ("mirror docs child", MIRROR_DOCS_SOURCE, 170),
        ("source helpers child", SOURCE_HELPERS_SOURCE, 80),
        ("split layout guard", SPLIT_LAYOUT_SOURCE, 220),
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
        ("Runtime 11 plan", RUNTIME_11_PLAN),
        ("Runtime 15 plan", RUNTIME_15_PLAN),
        ("runtime index", RUNTIME_INDEX),
        ("structure convention plan", STRUCTURE_CONVENTION_PLAN),
        ("review findings plan", REVIEW_FINDINGS_PLAN),
        ("module convention doc", MODULE_CONVENTION_DOC),
        ("Frameworks 02 plan", FRAMEWORKS_02_PLAN),
        ("session note", SESSION_NOTE),
        ("status row data", STATUS_ROW_DATA),
        ("status map", STATUS_MAP),
    ] {
        assert!(
            source.contains("runtime_15_job_system_route_owner_split_static_passed_cargo_deferred")
                || NUMBERED_STATUS_RECORDS.contains(
                    "runtime_15_job_system_route_owner_split_static_passed_cargo_deferred"
                ),
            "{label} should mirror the job_system route-owner split status"
        );
    }
    assert!(
        DATE_MAP.contains("Runtime 15 M3 job-system route-owner split"),
        "date map should mirror the job_system route-owner split slice"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "job_system/inventory.rs",
            "job_system/mirror_docs.rs",
            "job_system/source_helpers.rs",
            "job_system/split_layout.rs",
            "runtime_15_job_system_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 plan",
        NUMBERED_STATUS_RECORDS,
        &[
            "frameworks_02_m3_job_system_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 job-system route-owner split",
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
