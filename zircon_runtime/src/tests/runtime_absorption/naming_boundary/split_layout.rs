const PARENT_SOURCE: &str = include_str!("../naming_boundary.rs");
const CLASSIFIERS_SOURCE: &str = include_str!("classifiers.rs");
const LEXICAL_SCAN_SOURCE: &str = include_str!("lexical_scan.rs");
const PRODUCTION_LINES_SOURCE: &str = include_str!("lexical_scan/production_lines.rs");
const SUPPORT_SOURCE: &str = include_str!("support.rs");
const STATUS_EVIDENCE_SOURCE: &str = include_str!("support/status_evidence.rs");
const TOP_LEVEL_SOURCE: &str = include_str!("top_level.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");
const RUNTIME_15_M2_SOURCE: &str = include_str!("runtime_15_m2.rs");
const ASSET_DYNAMIC_SOURCE: &str = include_str!("runtime_15_m2/asset_dynamic.rs");
const GRAPHICS_HYBRID_GI_SOURCE: &str = include_str!("runtime_15_m2/graphics/hybrid_gi.rs");

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
    "../plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/asset_budget_tests/naming_graphics_misc/root_route_rows.rs"
);
const STATUS_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/structure_route_maps/naming_boundary_rows.rs"
);
const DATE_MAP: &str = include_str!(
    "../plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/structure_route_maps/naming_boundary_rows.rs"
);

#[test]
fn runtime_15_naming_boundary_route_owner_is_folder_backed() {
    assert_contains_all(
        "parent route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"naming_boundary/classifiers.rs\"]",
            "#[path = \"naming_boundary/lexical_scan.rs\"]",
            "mod runtime_15_m2;",
            "#[path = \"naming_boundary/split_layout.rs\"]",
            "#[path = \"naming_boundary/support.rs\"]",
            "#[path = \"naming_boundary/top_level.rs\"]",
        ],
    );
    assert_parent_route_only();
    assert_child_owners_are_focused();
    assert_runtime_15_m2_imports_support_owner();
    assert_line_budget();
    assert_docs_and_status_mirror_split();
}

fn assert_parent_route_only() {
    assert!(
        !PARENT_SOURCE.contains("#[test]"),
        "naming_boundary.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "struct NamingReference",
        "fn runtime_editor_and_legacy_naming_is_classified_by_owner",
        "fn rust_source_files",
        "fn classify_editor_reference",
        "fn assert_contains_all",
        "fn read_repo_text",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "naming_boundary.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owners_are_focused() {
    assert_contains_all(
        "top-level guards",
        TOP_LEVEL_SOURCE,
        &[
            "runtime_editor_and_legacy_naming_is_classified_by_owner",
            "runtime_non_network_server_naming_is_classified_by_owner",
            "assert_no_unclassified_naming",
        ],
    );
    assert_contains_all(
        "classifiers",
        CLASSIFIERS_SOURCE,
        &[
            "allowed_server_context",
            "classify_server_reference",
            "classify_editor_reference",
            "classify_legacy_reference",
            "is_test_path",
        ],
    );
    assert_contains_all(
        "lexical scan",
        LEXICAL_SCAN_SOURCE,
        &[
            "NamingReference",
            "rust_source_files",
            "collect_naming_references",
            "collect_server_references",
            "token_has_server_component",
            "lexical_scan/production_lines.rs",
        ],
    );
    assert_contains_all(
        "support",
        SUPPORT_SOURCE,
        &[
            "assert_contains_all",
            "read_text",
            "read_repo_text",
            "support/status_evidence.rs",
        ],
    );
    assert_contains_all(
        "status evidence support",
        STATUS_EVIDENCE_SOURCE,
        &[
            "read_runtime_15_naming_status_rows",
            "read_runtime_test_children",
        ],
    );
}

fn assert_runtime_15_m2_imports_support_owner() {
    assert!(
        !ASSET_DYNAMIC_SOURCE.contains("use super::super::{"),
        "Runtime 15 M2 child owners should not import helpers from naming_boundary.rs"
    );
    assert!(
        !GRAPHICS_HYBRID_GI_SOURCE.contains("use super::super::super::{"),
        "nested Runtime 15 M2 child owners should not import helpers from naming_boundary.rs"
    );
    assert_contains_all(
        "Runtime 15 M2 support imports",
        ASSET_DYNAMIC_SOURCE,
        &["use super::super::support::{"],
    );
    assert_contains_all(
        "nested Runtime 15 M2 support imports",
        GRAPHICS_HYBRID_GI_SOURCE,
        &["use super::super::super::support::{"],
    );
    assert_contains_all(
        "Runtime 15 M2 route",
        RUNTIME_15_M2_SOURCE,
        &["mod asset_dynamic;", "mod asset_schema;", "mod ui;"],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 20),
        ("top-level guards", TOP_LEVEL_SOURCE, 120),
        ("classifiers", CLASSIFIERS_SOURCE, 190),
        ("lexical scan", LEXICAL_SCAN_SOURCE, 120),
        ("production-line scan", PRODUCTION_LINES_SOURCE, 80),
        ("support", SUPPORT_SOURCE, 50),
        ("status evidence support", STATUS_EVIDENCE_SOURCE, 80),
        ("split layout", SPLIT_LAYOUT_SOURCE, 240),
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
            source.contains(
                "runtime_15_naming_boundary_route_owner_split_static_passed_cargo_deferred"
            ),
            "{label} should mirror the naming_boundary route-owner split status"
        );
    }
    assert!(
        DATE_MAP.contains("Runtime 15 M3 naming-boundary route-owner split"),
        "date map should mirror the naming_boundary route-owner split slice"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "naming_boundary/top_level.rs",
            "naming_boundary/classifiers.rs",
            "naming_boundary/lexical_scan.rs",
            "naming_boundary/support.rs",
            "runtime_15_naming_boundary_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 plan",
        FRAMEWORKS_02_PLAN,
        &[
            "frameworks_02_m3_naming_boundary_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 naming-boundary route-owner split",
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
