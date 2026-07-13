const PARENT_SOURCE: &str = include_str!("../resource_foundation.rs");
const RUNTIME_SURFACE_SOURCE: &str = include_str!("runtime_surface.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

const FRAMEWORKS_02_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
);
const RUNTIME_15_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
);
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");

#[test]
fn runtime_15_resource_foundation_route_owner_is_folder_backed() {
    assert_contains_all(
        "resource_foundation route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"resource_foundation/runtime_surface.rs\"]",
            "#[path = \"resource_foundation/split_layout.rs\"]",
        ],
    );
    assert_parent_route_only();
    assert_child_owner_is_focused();
    assert_line_budget();
    assert_docs_and_status_mirror_split();
}

fn assert_parent_route_only() {
    assert!(
        !PARENT_SOURCE.contains("#[test]"),
        "resource_foundation.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "ResourceInspectorAdapterKey",
        "RuntimeResourceState",
        "std::fs::read_to_string",
        "include_str!(",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "resource_foundation.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owner_is_focused() {
    assert_contains_all(
        "runtime surface child",
        RUNTIME_SURFACE_SOURCE,
        &[
            "runtime_resource_foundation_keeps_editor_inspector_surface_internal",
            "src/core/resource/mod.rs",
            "RuntimeResourceState",
            "ResourceInspectorAdapterKey",
        ],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 6),
        ("runtime surface child", RUNTIME_SURFACE_SOURCE, 40),
        ("split layout guard", SPLIT_LAYOUT_SOURCE, 180),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{label} has {line_count} lines; expected at most {max_lines}"
        );
    }
}

fn assert_docs_and_status_mirror_split() {
    assert!(
        RUNTIME_15_OUTPUT_RECORDS.contains(
            "runtime_15_resource_foundation_route_owner_split_static_passed_cargo_deferred"
        ),
        "Runtime 15 output records should own the resource_foundation route-owner split status"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "resource_foundation/runtime_surface.rs",
            "resource_foundation/split_layout.rs",
            "runtime_15_resource_foundation_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 output records",
        FRAMEWORKS_02_OUTPUT_RECORDS,
        &[
            "frameworks_02_m3_resource_foundation_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 resource-foundation route-owner split",
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
