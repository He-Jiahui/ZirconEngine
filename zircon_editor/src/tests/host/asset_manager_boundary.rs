#[test]
fn editor_asset_boundary_lives_in_editor_crate() {
    let app_source = include_str!("../../ui/slint_host/app.rs");
    let host_lifecycle_source = include_str!("../../ui/slint_host/app/host_lifecycle.rs");
    let project_access_source = include_str!("../../core/host/manager/project_access.rs");
    let asset_workspace_source = include_str!("../../core/editing/asset_workspace.rs");
    let accessors_source = include_str!("../../core/editor_event/runtime/accessors.rs");
    let runtime_asset_root_source = include_str!("../../../../zircon_runtime/src/asset/mod.rs");
    let runtime_asset_module_source =
        include_str!("../../../../zircon_runtime/src/asset/module.rs");

    assert!(
        app_source.contains("use crate::core::host::asset_editor::{"),
        "editor app should import editor asset API from crate::core::host::asset_editor"
    );
    assert!(
        app_source.contains("EditorAssetChange"),
        "editor app should use editor-owned asset change DTOs"
    );
    assert!(
        app_source.contains("use zircon_runtime::asset::pipeline::manager::AssetManager;"),
        "editor app should use runtime asset manager via zircon_runtime::asset::pipeline::manager"
    );
    assert!(
        asset_workspace_source.contains("use crate::core::host::asset_editor::{"),
        "asset workspace state should import editor asset catalog types from crate::core::host::asset_editor"
    );
    assert!(
        accessors_source.contains("use crate::core::host::asset_editor::{"),
        "editor event runtime accessors should import editor asset snapshot types from crate::core::host::asset_editor"
    );
    assert!(
        host_lifecycle_source.contains("use crate::core::host::asset_editor::resolve_editor_asset_manager;"),
        "editor host lifecycle should resolve the editor asset server through crate::core::host::asset_editor"
    );
    assert!(
        host_lifecycle_source.contains("use zircon_runtime::asset::pipeline::manager::resolve_asset_manager;"),
        "editor host lifecycle should resolve the generic asset server through zircon_runtime::asset::pipeline::manager"
    );
    assert!(
        !host_lifecycle_source.contains("resolver.editor_asset()?"),
        "editor host lifecycle should not resolve editor asset API from zircon_runtime::core::manager::ManagerResolver"
    );
    assert!(
        !host_lifecycle_source.contains("resolver.asset()?"),
        "editor host lifecycle should not resolve generic asset API from zircon_runtime::core::manager::ManagerResolver"
    );
    assert!(
        project_access_source.contains("use zircon_runtime::asset::pipeline::manager::{resolve_asset_manager, AssetManager};"),
        "editor manager project access should resolve the asset server through zircon_runtime::asset::pipeline::manager"
    );
    assert!(
        !runtime_asset_root_source.contains("pub mod editor;"),
        "runtime asset root should no longer expose an editor namespace"
    );
    assert!(
        !runtime_asset_root_source.contains("pub(crate) use editor::{"),
        "runtime asset root should no longer re-export editor asset implementation"
    );
    assert!(
        !runtime_asset_module_source.contains("DefaultEditorAssetManager"),
        "runtime asset module should not register editor asset managers"
    );
    assert!(
        !runtime_asset_module_source.contains("EditorAssetManagerHandle"),
        "runtime asset module should not expose editor asset handles"
    );
}

#[test]
fn editor_host_uses_asset_owned_asset_change_stream() {
    let app_source = include_str!("../../ui/slint_host/app.rs");
    let backend_refresh_source = include_str!("../../ui/slint_host/app/backend_refresh.rs");
    let slint_asset_refresh_test_source = include_str!("slint_asset_refresh.rs");

    for source in [
        app_source,
        backend_refresh_source,
        slint_asset_refresh_test_source,
    ] {
        assert!(
            !source.contains("AssetChangeRecord"),
            "editor host sources should not depend on zircon_runtime::core::manager::AssetChangeRecord after asset boundary cleanup"
        );
    }

    for source in [
        app_source,
        backend_refresh_source,
        slint_asset_refresh_test_source,
    ] {
        assert!(
            source.contains("AssetChange"),
            "editor host sources should use zircon_runtime::asset::watch::AssetChange after asset boundary cleanup"
        );
    }
}

#[test]
fn editor_asset_workspace_uses_canonical_resource_kind() {
    let asset_workspace_source = include_str!("../../core/editing/asset_workspace.rs");
    let editor_state_asset_workspace_source =
        include_str!("../../core/editing/state/editor_state_asset_workspace.rs");
    let event_common_source = include_str!("../../core/editor_event/runtime/execution/common.rs");
    let resource_access_source = include_str!("../../core/host/resource_access.rs");
    let asset_item_snapshot_source =
        include_str!("../../ui/workbench/snapshot/asset/asset_item_snapshot.rs");
    let asset_reference_snapshot_source =
        include_str!("../../ui/workbench/snapshot/asset/asset_reference_snapshot.rs");
    let asset_selection_snapshot_source =
        include_str!("../../ui/workbench/snapshot/asset/asset_selection_snapshot.rs");
    let asset_workspace_snapshot_source =
        include_str!("../../ui/workbench/snapshot/asset/asset_workspace_snapshot.rs");

    for source in [
        asset_workspace_source,
        editor_state_asset_workspace_source,
        event_common_source,
        resource_access_source,
        asset_item_snapshot_source,
        asset_reference_snapshot_source,
        asset_selection_snapshot_source,
        asset_workspace_snapshot_source,
    ] {
        assert!(
            !source.contains("AssetRecordKind"),
            "editor asset workspace sources should use zircon_runtime::core::resource::ResourceKind instead of zircon_runtime::core::manager::AssetRecordKind"
        );
    }
}

#[test]
fn editor_asset_workspace_uses_canonical_resource_state() {
    let asset_workspace_source = include_str!("../../core/editing/asset_workspace.rs");
    let editor_state_asset_workspace_source =
        include_str!("../../core/editing/state/editor_state_asset_workspace.rs");
    let accessors_source = include_str!("../../core/editor_event/runtime/accessors.rs");
    let resource_access_source = include_str!("../../core/host/resource_access.rs");
    let asset_surface_source = include_str!("../../ui/layouts/views/asset_surface_presentation.rs");
    let asset_item_snapshot_source =
        include_str!("../../ui/workbench/snapshot/asset/asset_item_snapshot.rs");
    let asset_selection_snapshot_source =
        include_str!("../../ui/workbench/snapshot/asset/asset_selection_snapshot.rs");

    for source in [
        asset_workspace_source,
        editor_state_asset_workspace_source,
        accessors_source,
        resource_access_source,
        asset_surface_source,
        asset_item_snapshot_source,
        asset_selection_snapshot_source,
    ] {
        assert!(
            !source.contains("ResourceStateRecord"),
            "editor asset workspace sources should use zircon_runtime::core::resource::ResourceState instead of zircon_runtime::core::manager::ResourceStateRecord"
        );
    }

    for source in [
        resource_access_source,
        asset_surface_source,
        asset_item_snapshot_source,
        asset_selection_snapshot_source,
    ] {
        assert!(
            source.contains("ResourceState"),
            "editor asset workspace sources should refer to zircon_runtime::core::resource::ResourceState"
        );
    }
}

#[test]
fn editor_asset_workspace_uses_canonical_resource_record() {
    let asset_workspace_source = include_str!("../../core/editing/asset_workspace.rs");
    let editor_state_asset_workspace_source =
        include_str!("../../core/editing/state/editor_state_asset_workspace.rs");
    let accessors_source = include_str!("../../core/editor_event/runtime/accessors.rs");
    let resource_access_source = include_str!("../../core/host/resource_access.rs");
    let resource_access_test_source = include_str!("resource_access.rs");

    for source in [
        asset_workspace_source,
        editor_state_asset_workspace_source,
        accessors_source,
        resource_access_source,
        resource_access_test_source,
    ] {
        assert!(
            !source.contains("ResourceStatusRecord"),
            "editor sources should not depend on zircon_runtime::core::manager::ResourceStatusRecord after canonical ResourceRecord migration"
        );
    }

    for source in [asset_workspace_source, resource_access_test_source] {
        assert!(
            source.contains("ResourceRecord"),
            "editor sources should use zircon_runtime::core::resource::ResourceRecord after canonical ResourceRecord migration"
        );
    }
}

#[test]
fn editor_host_uses_canonical_resource_event() {
    let app_source = include_str!("../../ui/slint_host/app.rs");
    let backend_refresh_source = include_str!("../../ui/slint_host/app/backend_refresh.rs");
    let resource_access_test_source = include_str!("resource_access.rs");
    let slint_asset_refresh_test_source = include_str!("slint_asset_refresh.rs");

    for source in [
        app_source,
        backend_refresh_source,
        resource_access_test_source,
        slint_asset_refresh_test_source,
    ] {
        assert!(
            !source.contains("ResourceChangeRecord"),
            "editor host sources should not depend on zircon_runtime::core::manager::ResourceChangeRecord after canonical ResourceEvent migration"
        );
        assert!(
            !source.contains("ResourceChangeKind"),
            "editor host sources should not depend on zircon_runtime::core::manager::ResourceChangeKind after canonical ResourceEvent migration"
        );
    }

    for source in [
        app_source,
        backend_refresh_source,
        slint_asset_refresh_test_source,
    ] {
        assert!(
            source.contains("ResourceEvent"),
            "editor host sources should use zircon_runtime::core::resource::ResourceEvent after canonical ResourceEvent migration"
        );
    }
}
