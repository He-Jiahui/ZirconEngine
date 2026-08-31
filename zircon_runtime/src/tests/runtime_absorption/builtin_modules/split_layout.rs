const PARENT_SOURCE: &str = include_str!("../builtin_modules.rs");
const CORE_SPINE_SOURCE: &str = include_str!("core_spine.rs");
const PLUGIN_SELECTION_SOURCE: &str = include_str!("plugin_selection.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

const FRAMEWORKS_02_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
);
const RUNTIME_15_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
);
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");

#[test]
fn runtime_15_builtin_modules_route_owner_is_folder_backed() {
    assert_contains_all(
        "builtin_modules route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"builtin_modules/core_spine.rs\"]",
            "#[path = \"builtin_modules/plugin_selection.rs\"]",
            "#[path = \"builtin_modules/split_layout.rs\"]",
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
        "builtin_modules.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "client_runtime_composition_includes_core_and_required_plugins",
        "required_unavailable_runtime_plugin_is_reported_as_fatal_missing",
        "RuntimePluginId::VirtualGeometry",
        "ProjectPluginManifest",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "builtin_modules.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owners_are_focused() {
    assert_contains_all(
        "core spine child",
        CORE_SPINE_SOURCE,
        &[
            "client_runtime_composition_includes_core_and_required_plugins",
            "client_runtime_composition_keeps_plugins_after_core_spine",
            "FOUNDATION_MODULE_NAME",
            "GRAPHICS_MODULE_NAME",
            "SCRIPT_MODULE_NAME",
        ],
    );
    assert_contains_all(
        "plugin selection child",
        PLUGIN_SELECTION_SOURCE,
        &[
            "required_unavailable_runtime_plugin_is_reported_as_fatal_missing",
            "optional_unavailable_runtime_plugin_stays_warning_only",
            "physics_animation_manifest_entries_require_linked_external_plugins",
            "RuntimePluginId::VirtualGeometry",
            "ProjectPluginManifest",
        ],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 8),
        ("core spine child", CORE_SPINE_SOURCE, 90),
        ("plugin selection child", PLUGIN_SELECTION_SOURCE, 90),
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
        RUNTIME_15_OUTPUT_RECORDS
            .contains("runtime_15_builtin_modules_route_owner_split_static_passed_cargo_deferred"),
        "Runtime 15 output records should own the builtin_modules route-owner split status"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "builtin_modules/core_spine.rs",
            "builtin_modules/plugin_selection.rs",
            "builtin_modules/split_layout.rs",
            "runtime_15_builtin_modules_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 output records",
        FRAMEWORKS_02_OUTPUT_RECORDS,
        &[
            "frameworks_02_m3_builtin_modules_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 builtin-modules route-owner split",
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
