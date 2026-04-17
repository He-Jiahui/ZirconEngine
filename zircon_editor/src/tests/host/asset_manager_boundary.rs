#[test]
fn editor_asset_boundary_lives_in_asset_crate() {
    let app_source = include_str!("../../host/slint_host/app.rs");
    let host_lifecycle_source = include_str!("../../host/slint_host/app/host_lifecycle.rs");
    let asset_workspace_source = include_str!("../../editing/asset_workspace.rs");
    let accessors_source = include_str!("../../editor_event/runtime/accessors.rs");

    assert!(
        app_source.contains("use zircon_asset::{"),
        "editor app should import editor asset API from zircon_asset"
    );
    assert!(
        app_source.contains("EditorAssetChangeRecord"),
        "editor app should use asset-owned EditorAssetChangeRecord"
    );
    assert!(
        asset_workspace_source.contains("use zircon_asset::{"),
        "asset workspace state should import editor asset catalog types from zircon_asset"
    );
    assert!(
        accessors_source.contains("use zircon_asset::{"),
        "editor event runtime accessors should import editor asset snapshot types from zircon_asset"
    );
    assert!(
        host_lifecycle_source.contains("resolve_editor_asset_manager"),
        "editor host lifecycle should resolve the editor asset server through zircon_asset"
    );
    assert!(
        !host_lifecycle_source.contains("resolver.editor_asset()?"),
        "editor host lifecycle should not resolve editor asset API from zircon_manager::ManagerResolver"
    );
}

#[test]
fn editor_asset_workspace_uses_canonical_resource_kind() {
    let asset_workspace_source = include_str!("../../editing/asset_workspace.rs");
    let editor_state_asset_workspace_source =
        include_str!("../../editing/state/editor_state_asset_workspace.rs");
    let event_common_source = include_str!("../../editor_event/runtime/execution/common.rs");
    let resource_access_source = include_str!("../../host/resource_access.rs");
    let asset_item_snapshot_source =
        include_str!("../../workbench/snapshot/asset/asset_item_snapshot.rs");
    let asset_reference_snapshot_source =
        include_str!("../../workbench/snapshot/asset/asset_reference_snapshot.rs");
    let asset_selection_snapshot_source =
        include_str!("../../workbench/snapshot/asset/asset_selection_snapshot.rs");
    let asset_workspace_snapshot_source =
        include_str!("../../workbench/snapshot/asset/asset_workspace_snapshot.rs");

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
            "editor asset workspace sources should use zircon_resource::ResourceKind instead of zircon_manager::AssetRecordKind"
        );
    }
}

#[test]
fn editor_asset_workspace_uses_canonical_resource_state() {
    let asset_workspace_source = include_str!("../../editing/asset_workspace.rs");
    let editor_state_asset_workspace_source =
        include_str!("../../editing/state/editor_state_asset_workspace.rs");
    let accessors_source = include_str!("../../editor_event/runtime/accessors.rs");
    let resource_access_source = include_str!("../../host/resource_access.rs");
    let asset_surface_source = include_str!("../../host/slint_host/ui/asset_surface_presentation.rs");
    let asset_item_snapshot_source =
        include_str!("../../workbench/snapshot/asset/asset_item_snapshot.rs");
    let asset_selection_snapshot_source =
        include_str!("../../workbench/snapshot/asset/asset_selection_snapshot.rs");

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
            "editor asset workspace sources should use zircon_resource::ResourceState instead of zircon_manager::ResourceStateRecord"
        );
    }

    for source in [
        asset_workspace_source,
        resource_access_source,
        asset_surface_source,
        asset_item_snapshot_source,
        asset_selection_snapshot_source,
    ] {
        assert!(
            source.contains("ResourceState"),
            "editor asset workspace sources should refer to zircon_resource::ResourceState"
        );
    }
}
