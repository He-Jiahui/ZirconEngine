use super::*;
use crate::ui::workbench::snapshot::{
    AssetFolderSnapshot, AssetItemSnapshot, AssetReferenceSnapshot, AssetViewMode,
    AssetWorkspaceSnapshot,
};
use zircon_runtime_interface::resource::ResourceKind;

fn snapshot(count: usize, prefix: &str) -> AssetWorkspaceSnapshot {
    let folders = (0..count)
        .map(|index| AssetFolderSnapshot {
            folder_id: format!("res://{prefix}/folder-{index}"),
            ..AssetFolderSnapshot::default()
        })
        .collect::<Vec<_>>();
    let mut snapshot = AssetWorkspaceSnapshot {
        catalog_revision: count as u64,
        view_mode: AssetViewMode::List,
        folder_tree: folders.clone(),
        visible_folders: folders.into(),
        visible_assets: (0..count)
            .map(|index| {
                let uuid = format!("{prefix}-{index}");
                AssetItemSnapshot {
                    locator: format!("res://{prefix}/{index}.texture"),
                    display_name: uuid.clone(),
                    file_name: format!("{index}.texture"),
                    extension: "texture".to_owned(),
                    kind: ResourceKind::Texture,
                    asset_type: Default::default(),
                    preview_artifact_path: String::new(),
                    dirty: false,
                    diagnostics: Vec::new(),
                    selected: false,
                    resource_state: None,
                    resource_revision: None,
                    uuid,
                }
            })
            .collect(),
        ..AssetWorkspaceSnapshot::default()
    };
    snapshot.selection.references = (0..count)
        .map(|index| AssetReferenceSnapshot {
            uuid: format!("{prefix}-reference-{index}"),
            locator: format!("res://{prefix}/reference-{index}.texture"),
            known_project_asset: true,
            ..AssetReferenceSnapshot::default()
        })
        .collect();
    snapshot.selection.used_by = snapshot.selection.references.clone();
    snapshot
}

fn set_sizes(surface: &mut AssetSurfacePointerState, size: UiSize) {
    surface.tree_size = size;
    surface.content_size = size;
    surface.references.size = size;
    surface.used_by.size = size;
}

fn surface_with_offset(offset: f32) -> AssetSurfacePointerState {
    let mut surface = AssetSurfacePointerState::new();
    set_sizes(&mut surface, UiSize::new(400.0, 256.0));
    surface.tree_state.scroll_offset = offset;
    surface.content_state.scroll_offset = offset;
    surface.references.state.scroll_offset = offset;
    surface.used_by.state.scroll_offset = offset;
    surface
}

fn offsets(surface: &AssetSurfacePointerState) -> [f32; 4] {
    [
        surface.tree_state.scroll_offset,
        surface.content_state.scroll_offset,
        surface.references.state.scroll_offset,
        surface.used_by.state.scroll_offset,
    ]
}

fn geometry_generations(surface: &AssetSurfacePointerState) -> [u64; 4] {
    [
        surface.tree_bridge.surface_authority_generation_for_test(),
        surface
            .content_bridge
            .surface_authority_generation_for_test(),
        surface
            .references
            .bridge
            .surface_authority_generation_for_test(),
        surface
            .used_by
            .bridge
            .surface_authority_generation_for_test(),
    ]
}

#[test]
fn snapshot_shrink_writes_all_four_clamped_offsets_to_the_paint_owner() {
    for profile in [
        AssetContentSurfaceProfile::Activity,
        AssetContentSurfaceProfile::Browser,
    ] {
        let mut surface = surface_with_offset(1_000_000.0);
        surface.sync_snapshot_pointer_layouts(&snapshot(64, "large"), profile);
        assert!(offsets(&surface).into_iter().all(|offset| offset > 0.0));

        surface.sync_snapshot_pointer_layouts(&snapshot(1, "small"), profile);

        assert_eq!(offsets(&surface), [0.0; 4]);
        assert_eq!(surface.tree_state, surface.tree_bridge.pointer_state());
        assert_eq!(
            surface.content_state,
            surface.content_bridge.pointer_state()
        );
        assert_eq!(
            surface.references.state,
            surface.references.bridge.pointer_state()
        );
        assert_eq!(
            surface.used_by.state,
            surface.used_by.bridge.pointer_state()
        );
    }
}

#[test]
fn empty_snapshot_clears_offsets_and_drops_previous_asset_hit_targets() {
    let profile = AssetContentSurfaceProfile::Browser;
    let mut surface = surface_with_offset(1_000_000.0);
    surface.sync_snapshot_pointer_layouts(&snapshot(64, "old"), profile);

    let empty = snapshot(0, "empty");
    surface.sync_snapshot_pointer_layouts(&empty, profile);
    surface.sync_snapshot_pointer_layouts(&empty, profile);

    assert_eq!(offsets(&surface), [0.0; 4]);
    assert!(!matches!(
        surface.content_bridge.route_at(UiPoint::new(24.0, 24.0)),
        Some(AssetPointerContentRoute::Item { .. })
    ));
    assert!(surface.snapshot.as_ref().unwrap().visible_assets.is_empty());
}

#[test]
fn snapshot_sync_with_larger_panes_publishes_the_new_scroll_bounds() {
    let profile = AssetContentSurfaceProfile::Browser;
    let snapshot = snapshot(64, "asset");
    let mut surface = surface_with_offset(1_000_000.0);
    surface.sync_snapshot_pointer_layouts(&snapshot, profile);
    assert!(offsets(&surface).into_iter().all(|offset| offset > 0.0));

    set_sizes(&mut surface, UiSize::new(400.0, 10_000.0));
    surface.sync_snapshot_pointer_layouts(&snapshot, profile);

    assert_eq!(offsets(&surface), [0.0; 4]);
}

#[test]
fn metadata_refresh_preserves_manual_scroll_and_geometry_generation() {
    let profile = AssetContentSurfaceProfile::Browser;
    let mut snapshot = snapshot(64, "asset");
    let mut surface = surface_with_offset(64.0);
    surface.sync_snapshot_pointer_layouts(&snapshot, profile);
    let generations = geometry_generations(&surface);
    snapshot.catalog_revision += 1;
    snapshot.project_name = "Updated project label".to_owned();

    surface.sync_snapshot_pointer_layouts(&snapshot, profile);

    assert_eq!(offsets(&surface), [64.0; 4]);
    assert_eq!(geometry_generations(&surface), generations);
    assert_eq!(
        surface.snapshot.as_ref().unwrap().catalog_revision,
        snapshot.catalog_revision
    );
}

#[test]
fn thumbnail_hit_after_shrink_uses_the_current_row_and_published_offset() {
    let profile = AssetContentSurfaceProfile::Browser;
    let mut surface = surface_with_offset(1_000_000.0);
    let mut previous = snapshot(64, "previous");
    previous.view_mode = AssetViewMode::Thumbnail;
    surface.sync_snapshot_pointer_layouts(&previous, profile);
    let mut current = snapshot(1, "current");
    current.view_mode = AssetViewMode::Thumbnail;

    surface.sync_snapshot_pointer_layouts(&current, profile);

    assert_eq!(surface.content_state.scroll_offset, 0.0);
    assert!(matches!(
        surface.content_bridge.route_at(UiPoint::new(24.0, 24.0)),
        Some(AssetPointerContentRoute::Item { asset_uuid, .. }) if asset_uuid == "current-0"
    ));
}
