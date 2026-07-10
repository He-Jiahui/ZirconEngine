const PARENT_SOURCE: &str = include_str!("../tech_stack.rs");
const MANIFEST_INVENTORY_SOURCE: &str = include_str!("manifest_inventory.rs");
const GUARD_ANCHORS_SOURCE: &str = include_str!("guard_anchors.rs");
const BEHAVIOR_ANCHORS_SOURCE: &str = include_str!("behavior_anchors.rs");
const MIRROR_DOCS_SOURCE: &str = include_str!("mirror_docs.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

const FRAMEWORKS_02_PLAN: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md");
const FRAMEWORKS_02_OUTPUT_ARCHIVE: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
);
const RUNTIME_15_PLAN: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
);
const RUNTIME_15_OUTPUT_ARCHIVE: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
);
const RUNTIME_INDEX: &str =
    include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
const STRUCTURE_CONVENTION_PLAN: &str =
    include_str!("../../../../../docs/plans/engine-code-structure-convention.md");
const STRUCTURE_CONVENTION_OUTPUT_ARCHIVE: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md"
);
const REVIEW_FINDINGS_PLAN: &str =
    include_str!("../../../../../docs/plans/engine-code-review-findings-2026-06.md");
const REVIEW_FINDINGS_OUTPUT_ARCHIVE: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"
);
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
const SESSION_NOTE: &str = include_str!(
    "../../../../../.codex/sessions/20260612-0847-runtime-architecture-implementation.md"
);
const STATUS_ROW_DATA: &str = include_str!(
    "../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards/runtime_structure_tests/runtime_absorption_platform_rows.rs"
);
const STATUS_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps/core_route_rows.rs"
);
const DATE_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps/core_route_rows.rs"
);

#[test]
fn runtime_15_tech_stack_route_owner_is_folder_backed() {
    assert_contains_all(
        "tech_stack route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"tech_stack/behavior_anchors.rs\"]",
            "#[path = \"tech_stack/guard_anchors.rs\"]",
            "#[path = \"tech_stack/manifest_inventory.rs\"]",
            "#[path = \"tech_stack/mirror_docs.rs\"]",
            "#[path = \"tech_stack/split_layout.rs\"]",
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
        "tech_stack.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "EXPECTED_RUNTIME_01_MANIFESTS",
        "tech_stack_dependency_guard",
        "behavior_test_anchor",
        "mirror_docs_match_structure_audit_counts",
        "include_str!(",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "tech_stack.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owners_are_focused() {
    assert_contains_all(
        "manifest inventory child",
        MANIFEST_INVENTORY_SOURCE,
        &[
            "EXPECTED_RUNTIME_01_MANIFESTS",
            "assert_runtime_01_manifests_exist",
            "../zircon_plugins/physics/runtime/Cargo.toml",
        ],
    );
    assert_contains_all(
        "guard anchors child",
        GUARD_ANCHORS_SOURCE,
        &[
            "assert_runtime_01_guard_anchors",
            "tech_stack_dependency_guard.rs",
            "runtime_01_tech_stack_cargo_gate_stays_visible_until_dependency_validation",
        ],
    );
    assert_contains_all(
        "behavior anchors child",
        BEHAVIOR_ANCHORS_SOURCE,
        &[
            "assert_runtime_01_behavior_anchors",
            "shared_text_shaper_matches_public_layout_entrypoint",
            "unavailable_jolt_backend_does_not_fallback_to_builtin_scene_tick",
        ],
    );
    assert_contains_all(
        "mirror docs child",
        MIRROR_DOCS_SOURCE,
        &[
            "runtime_01_tech_stack_mirror_docs_match_structure_audit_counts",
            "assert_mirror_docs_match_structure_audit",
            "runtime tech-stack doc",
            "interface convergence",
        ],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 12),
        ("manifest inventory child", MANIFEST_INVENTORY_SOURCE, 40),
        ("guard anchors child", GUARD_ANCHORS_SOURCE, 70),
        ("behavior anchors child", BEHAVIOR_ANCHORS_SOURCE, 50),
        ("mirror docs child", MIRROR_DOCS_SOURCE, 90),
        ("split layout guard", SPLIT_LAYOUT_SOURCE, 240),
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
        ("Frameworks 02 output archive", FRAMEWORKS_02_OUTPUT_ARCHIVE),
        ("Runtime 15 output archive", RUNTIME_15_OUTPUT_ARCHIVE),
        (
            "structure convention output archive",
            STRUCTURE_CONVENTION_OUTPUT_ARCHIVE,
        ),
        (
            "review findings output archive",
            REVIEW_FINDINGS_OUTPUT_ARCHIVE,
        ),
        ("module convention doc", MODULE_CONVENTION_DOC),
        ("session note", SESSION_NOTE),
        ("status row data", STATUS_ROW_DATA),
        ("status map", STATUS_MAP),
    ] {
        assert!(
            source.contains("runtime_15_tech_stack_route_owner_split_static_passed_cargo_deferred"),
            "{label} should mirror the tech_stack route-owner split status"
        );
    }
    for (label, source, route_anchor) in [
        (
            "Frameworks 02 plan",
            FRAMEWORKS_02_PLAN,
            "02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
        ),
        (
            "Runtime 15 plan",
            RUNTIME_15_PLAN,
            "15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        ),
        (
            "runtime index",
            RUNTIME_INDEX,
            "15-code-structure-and-module-conventions.md",
        ),
        (
            "structure convention plan",
            STRUCTURE_CONVENTION_PLAN,
            "15/2026-07-09-engine-code-structure-output-records.md",
        ),
        (
            "review findings plan",
            REVIEW_FINDINGS_PLAN,
            "15/2026-07-09-engine-code-review-findings-output-records.md",
        ),
    ] {
        assert_contains_all(label, source, &[route_anchor, "此处仅展示当前现状的概述"]);
    }
    assert!(
        DATE_MAP.contains("Runtime 15 M3 tech-stack route-owner split"),
        "date map should mirror the tech_stack route-owner split slice"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "tech_stack/manifest_inventory.rs",
            "tech_stack/guard_anchors.rs",
            "tech_stack/behavior_anchors.rs",
            "tech_stack/mirror_docs.rs",
            "tech_stack/split_layout.rs",
            "runtime_15_tech_stack_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 output archive",
        FRAMEWORKS_02_OUTPUT_ARCHIVE,
        &[
            "frameworks_02_m3_tech_stack_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 tech-stack route-owner split",
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
