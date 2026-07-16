const PARENT_SOURCE: &str = include_str!("../root_surface.rs");
const DOCS_SOURCE: &str = include_str!("docs.rs");
const GRAPHICS_ALIAS_SOURCE: &str = include_str!("graphics_alias.rs");
const INVENTORY_SOURCE: &str = include_str!("inventory.rs");
const PUBLIC_SURFACE_SOURCE: &str = include_str!("public_surface.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");
const CORE_SPINE_ROOT_GENERATED_INVENTORY_SOURCE: &str =
    include_str!("../core_spine_root_generated/inventory.rs");
const CORE_SPINE_ROOT_GENERATED_MIRROR_DOCS_SOURCE: &str =
    include_str!("../core_spine_root_generated/mirror_docs.rs");

const RUNTIME_15_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
);
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
const FRAMEWORKS_02_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
);

#[test]
fn runtime_15_root_surface_route_owner_is_folder_backed() {
    assert_contains_all(
        "parent route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"root_surface/docs.rs\"]",
            "#[path = \"root_surface/graphics_alias.rs\"]",
            "#[path = \"root_surface/inventory.rs\"]",
            "#[path = \"root_surface/public_surface.rs\"]",
            "#[path = \"root_surface/split_layout.rs\"]",
        ],
    );
    assert_parent_route_only();
    assert_child_owners_are_focused();
    assert_core_spine_generated_mirror_scans_child_owners();
    assert_line_budget();
    assert_docs_and_status_mirror_split();
}

fn assert_parent_route_only() {
    assert!(
        !PARENT_SOURCE.contains("#[test]"),
        "root_surface.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "const LIB_RS",
        "fn runtime_crate_root_public_surface_stays_curated",
        "fn graphics_alias_debt_is_removed_from_runtime_root",
        "fn core_spine_and_root_surface_docs_stay_in_sync",
        "fn root_surface_m1_gate_matches_runtime_14_module_family_seats",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "root_surface.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owners_are_focused() {
    assert_contains_all(
        "inventory child",
        INVENTORY_SOURCE,
        &[
            "LIB_RS",
            "PRELUDE_RS",
            "CORE_MOD_RS",
            "ROOT_SURFACE_DOC",
            "ROOT_SURFACE_M1_DOC",
            "INTERFACE_CONVERGENCE_DOC",
            "RUNTIME_02_PLAN",
            "RUNTIME_INDEX",
        ],
    );
    assert_contains_all(
        "public surface child",
        PUBLIC_SURFACE_SOURCE,
        &[
            "runtime_crate_root_public_surface_stays_curated",
            "public_modules",
            "root-surface cutover",
        ],
    );
    assert_contains_all(
        "graphics alias child",
        GRAPHICS_ALIAS_SOURCE,
        &[
            "graphics_alias_debt_is_removed_from_runtime_root",
            "graphics_type_alias_debt_symbols_are_only_available_through_graphics_namespace",
            "crate-visible graphics alias debt 0/0",
        ],
    );
    assert_contains_all(
        "docs child",
        DOCS_SOURCE,
        &[
            "core_spine_and_root_surface_docs_stay_in_sync",
            "root_surface_m1_gate_matches_runtime_14_module_family_seats",
            "root_surface_interface_convergence_mirror_uses_current_audit_counts",
        ],
    );
}

fn assert_core_spine_generated_mirror_scans_child_owners() {
    assert_contains_all(
        "core_spine_root_generated inventory",
        CORE_SPINE_ROOT_GENERATED_INVENTORY_SOURCE,
        &[
            "zircon_runtime/src/tests/runtime_absorption/root_surface/public_surface.rs",
            "zircon_runtime/src/tests/runtime_absorption/root_surface/graphics_alias.rs",
            "zircon_runtime/src/tests/runtime_absorption/root_surface/docs.rs",
        ],
    );
    assert_contains_all(
        "core_spine_root_generated mirror docs",
        CORE_SPINE_ROOT_GENERATED_MIRROR_DOCS_SOURCE,
        &[
            "root_surface_guard_files",
            "rust_test_count_in_files(runtime_root, &root_surface_guard_files)",
        ],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 20),
        ("inventory child", INVENTORY_SOURCE, 30),
        ("public surface child", PUBLIC_SURFACE_SOURCE, 110),
        ("graphics alias child", GRAPHICS_ALIAS_SOURCE, 130),
        ("docs child", DOCS_SOURCE, 150),
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
    assert!(
        RUNTIME_15_OUTPUT_RECORDS
            .contains("runtime_15_root_surface_route_owner_split_static_passed_cargo_deferred"),
        "Runtime 15 output records should own the root_surface route-owner split status"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "root_surface/public_surface.rs",
            "root_surface/graphics_alias.rs",
            "root_surface/docs.rs",
            "root_surface/inventory.rs",
            "runtime_15_root_surface_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 output records",
        FRAMEWORKS_02_OUTPUT_RECORDS,
        &[
            "frameworks_02_m3_root_surface_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 root-surface route-owner split",
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
