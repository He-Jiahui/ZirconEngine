const PARENT_SOURCE: &str = include_str!("../core_spine_root_generated.rs");
const INVENTORY_SOURCE: &str = include_str!("inventory.rs");
const MIRROR_DOCS_SOURCE: &str = include_str!("mirror_docs.rs");
const GENERATED_BEHAVIOR_SOURCE: &str = include_str!("generated_behavior.rs");
const SOURCE_HELPERS_SOURCE: &str = include_str!("source_helpers.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

const RUNTIME_15_PLAN: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
);
const RUNTIME_INDEX: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md");
const STRUCTURE_CONVENTION_PLAN: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md");
const REVIEW_FINDINGS_PLAN: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md");
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
const FRAMEWORKS_02_PLAN: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md");
const SESSION_NOTE: &str = include_str!(
    "../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
);
const STATUS_ROW_DATA: &str = include_str!(
    "../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/runtime_structure_tests/root_route_rows.rs"
);
const STATUS_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation/lock_poison.rs"
);
const DATE_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation/lock_poison.rs"
);

#[test]
fn runtime_15_core_spine_root_generated_is_folder_backed() {
    assert_contains_all(
        "parent route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"core_spine_root_generated/generated_behavior.rs\"]",
            "#[path = \"core_spine_root_generated/inventory.rs\"]",
            "#[path = \"core_spine_root_generated/mirror_docs.rs\"]",
            "#[path = \"core_spine_root_generated/source_helpers.rs\"]",
            "#[path = \"core_spine_root_generated/split_layout.rs\"]",
        ],
    );
    assert_parent_route_only();

    assert_contains_all(
        "inventory child",
        INVENTORY_SOURCE,
        &[
            "EXPECTED_CORE_ROOT_ENTRIES",
            "EXPECTED_CORE_PUBLIC_MODULES",
            "RETIRED_CORE_ROOT_ENTRIES",
            "EXPECTED_RUNTIME_02_GUARD_TEST_ANCHORS",
            "MIRROR_DOCS",
        ],
    );
    assert_contains_all(
        "mirror docs child",
        MIRROR_DOCS_SOURCE,
        &[
            "runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts",
            "assert_runtime_02_guard_test_anchors",
            "generated behavior migration debt reappeared",
            "Runtime 02 audit source",
        ],
    );
    assert_contains_all(
        "generated behavior child",
        GENERATED_BEHAVIOR_SOURCE,
        &[
            "GeneratedBehaviorLocation",
            "generated_behavior_locations",
            "generated_behavior_labels_for_line",
            "generated_behavior_requires_migration",
            "behavior_labels",
        ],
    );
    assert_contains_all(
        "source helpers child",
        SOURCE_HELPERS_SOURCE,
        &[
            "core_root_entries",
            "public_modules",
            "crate_visible_graphics_reexport_count",
            "export_template_files",
            "rust_test_count",
            "read_source",
        ],
    );
    assert_line_budget();
    assert_docs_and_status_mirror_split();
}

fn assert_parent_route_only() {
    assert!(
        !PARENT_SOURCE.contains("#[test]"),
        "core_spine_root_generated.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "const EXPECTED_CORE_ROOT_ENTRIES",
        "fn runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts",
        "struct GeneratedBehaviorLocation",
        "fn core_root_entries",
        "fn read_source",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "core_spine_root_generated.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 20),
        ("inventory child", INVENTORY_SOURCE, 150),
        ("mirror docs child", MIRROR_DOCS_SOURCE, 210),
        ("generated behavior child", GENERATED_BEHAVIOR_SOURCE, 90),
        ("source helpers child", SOURCE_HELPERS_SOURCE, 130),
        ("split layout guard", SPLIT_LAYOUT_SOURCE, 200),
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
        ("Runtime 15 output archive", RUNTIME_15_PLAN),
        ("runtime-index output archive", RUNTIME_INDEX),
        (
            "structure-convention output archive",
            STRUCTURE_CONVENTION_PLAN,
        ),
        ("review-findings output archive", REVIEW_FINDINGS_PLAN),
        ("module convention doc", MODULE_CONVENTION_DOC),
        ("Frameworks 02 output archive", FRAMEWORKS_02_PLAN),
        ("session note", SESSION_NOTE),
        ("status row data", STATUS_ROW_DATA),
        ("status map", STATUS_MAP),
    ] {
        assert!(
            source.contains(
                "runtime_15_core_spine_root_generated_route_owner_split_static_passed_cargo_deferred"
            ),
            "{label} should mirror the core_spine_root_generated route-owner split status"
        );
    }
    assert!(
        DATE_MAP.contains("Runtime 15 M3 core spine root/generated route-owner split"),
        "date map should mirror the core_spine_root_generated route-owner split slice"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "core_spine_root_generated/inventory.rs",
            "core_spine_root_generated/mirror_docs.rs",
            "core_spine_root_generated/generated_behavior.rs",
            "core_spine_root_generated/source_helpers.rs",
            "runtime_15_core_spine_root_generated_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 output archive",
        FRAMEWORKS_02_PLAN,
        &[
            "frameworks_02_m3_core_spine_root_generated_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 core spine root/generated route-owner split",
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
