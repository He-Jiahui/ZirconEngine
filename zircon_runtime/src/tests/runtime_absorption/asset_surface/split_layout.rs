const PARENT_SOURCE: &str = include_str!("../asset_surface.rs");
const REGISTRATION_SOURCE: &str = include_str!("registration.rs");
const NAMESPACE_SURFACE_SOURCE: &str = include_str!("namespace_surface.rs");
const FACADE_QUERY_SOURCE: &str = include_str!("facade_query.rs");
const SUPPORT_SOURCE: &str = include_str!("support.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

const RUNTIME_15_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
);
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
const FRAMEWORKS_02_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
);

#[test]
fn runtime_15_asset_surface_route_owner_is_folder_backed() {
    assert_contains_all(
        "asset_surface route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"asset_surface/facade_query.rs\"]",
            "#[path = \"asset_surface/namespace_surface.rs\"]",
            "#[path = \"asset_surface/registration.rs\"]",
            "#[path = \"asset_surface/support.rs\"]",
            "#[path = \"asset_surface/split_layout.rs\"]",
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
        "asset_surface.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "AssetModule",
        "runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free",
        "fn runtime_asset_surface_keeps_project_and_watch_under_namespaces",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "asset_surface.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owners_are_focused() {
    assert_contains_all(
        "registration child",
        REGISTRATION_SOURCE,
        &[
            "asset_module_registration_is_absorbed_into_runtime_asset_surface",
            "AssetModule",
            "standalone zircon_asset crate should be removed",
        ],
    );
    assert_contains_all(
        "namespace child",
        NAMESPACE_SURFACE_SOURCE,
        &[
            "runtime_asset_surface_keeps_project_and_watch_under_namespaces",
            "pub mod project;",
            "pub mod watch;",
            "pub use zircon_asset::ProjectAssetManager;",
        ],
    );
    assert_contains_all(
        "facade query child",
        FACADE_QUERY_SOURCE,
        &[
            "runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free",
            "ProjectAssetManager::readiness_report<TAsset>(handle)` is read-only.",
            "asset server vocabulary",
        ],
    );
    assert_contains_all(
        "support child",
        SUPPORT_SOURCE,
        &[
            "pub(super) fn read_runtime_file",
            "pub(super) fn read_workspace_file",
            "pub(super) fn assert_contains_all",
            "pub(super) fn assert_not_contains_all",
        ],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 12),
        ("registration child", REGISTRATION_SOURCE, 50),
        ("namespace child", NAMESPACE_SURFACE_SOURCE, 80),
        ("facade query child", FACADE_QUERY_SOURCE, 150),
        ("support child", SUPPORT_SOURCE, 60),
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
    assert!(
        RUNTIME_15_OUTPUT_RECORDS
            .contains("runtime_15_asset_surface_route_owner_split_static_passed_cargo_deferred"),
        "Runtime 15 output records should own the asset_surface route-owner split status"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "asset_surface/facade_query.rs",
            "asset_surface/namespace_surface.rs",
            "asset_surface/registration.rs",
            "asset_surface/support.rs",
            "asset_surface/split_layout.rs",
            "runtime_15_asset_surface_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 output records",
        FRAMEWORKS_02_OUTPUT_RECORDS,
        &[
            "frameworks_02_m3_asset_surface_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 asset-surface route-owner split",
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
