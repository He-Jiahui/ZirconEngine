use std::sync::Arc;
use std::time::Instant;

use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
    EditorAssetFolderRecord,
};
use crate::ui::workbench::snapshot::{AssetSurfaceMode, AssetUtilityTab, AssetViewMode};
use zircon_runtime::asset::project::PreviewState;
use zircon_runtime::core::resource::ResourceManagementGeneration;
use zircon_runtime_interface::resource::ResourceKind;

use super::{parent_folder_id_for_locator, AssetWorkspaceState};

#[test]
fn stable_resource_generation_skips_asset_projection_invalidation() {
    let mut workspace = AssetWorkspaceState::default();
    let generation = Arc::new(ResourceManagementGeneration::default());

    assert!(workspace.sync_resources(generation.clone()));
    assert!(!workspace.sync_resources(generation));
}

#[test]
fn idempotent_asset_controls_report_no_state_change() {
    let mut workspace = AssetWorkspaceState::default();

    assert!(!workspace.set_search_query(""));
    assert!(workspace.set_search_query("cube"));
    assert!(!workspace.set_search_query("cube"));
    assert!(!workspace.set_kind_filter(None));
    assert!(workspace.set_kind_filter(Some(ResourceKind::Mesh)));
    assert!(!workspace.set_kind_filter(Some(ResourceKind::Mesh)));
    assert!(!workspace.set_activity_view_mode(AssetViewMode::List));
    assert!(workspace.set_activity_view_mode(AssetViewMode::Thumbnail));
    assert!(!workspace.set_activity_view_mode(AssetViewMode::Thumbnail));
    assert!(!workspace.set_browser_utility_tab(AssetUtilityTab::Preview));
    assert!(workspace.set_browser_utility_tab(AssetUtilityTab::Metadata));
    assert!(!workspace.set_browser_utility_tab(AssetUtilityTab::Metadata));
}

#[test]
fn workspace_projection_generation_advances_once_for_each_exact_input_change() {
    let mut workspace = AssetWorkspaceState::default();
    workspace.sync_catalog(Arc::new(
        EditorAssetCatalogGeneration::from_snapshot_record(
            EditorAssetCatalogSnapshotRecord::default(),
            4,
        ),
    ));

    let initial = workspace
        .build_snapshot(AssetSurfaceMode::Activity)
        .catalog_revision;
    let stable = workspace
        .build_snapshot(AssetSurfaceMode::Activity)
        .catalog_revision;

    workspace.sync_catalog(Arc::new(
        EditorAssetCatalogGeneration::from_snapshot_record(
            EditorAssetCatalogSnapshotRecord::default(),
            5,
        ),
    ));
    let advanced = workspace
        .build_snapshot(AssetSurfaceMode::Activity)
        .catalog_revision;

    assert_eq!(stable, initial);
    assert_eq!(advanced, initial.wrapping_add(1));
    assert_ne!(advanced, initial);
    let legacy_hasher = ["Default", "Hasher"].concat();
    assert!(!include_str!("../asset_workspace_state.rs").contains(&legacy_hasher));
}

#[test]
#[should_panic(expected = "asset workspace projection generation exhausted")]
fn workspace_projection_generation_exhaustion_never_reuses_identity() {
    let workspace = AssetWorkspaceState::default();
    workspace.projection_generation.set(u64::MAX);
    let catalog = EditorAssetCatalogGeneration::default();

    let _ = workspace.asset_workspace_projection_generation(&catalog);
}

#[test]
#[ignore = "release-only current-source performance evidence"]
fn stable_asset_workspace_snapshot_scale_profile() {
    const ITERATIONS: usize = 32;
    const MARKER: &str = "EDITOR09_ASSET_WORKSPACE_STABLE_SNAPSHOT_PROFILE_V1";

    for item_count in [1, 1_000, 10_000] {
        let mut workspace = AssetWorkspaceState::default();
        workspace.sync_catalog(Arc::new(
            EditorAssetCatalogGeneration::from_snapshot_record(scale_catalog(item_count), 1),
        ));

        let warm = workspace.build_surface_snapshots();
        assert_eq!(warm.0.visible_assets.len(), item_count);
        assert!(warm
            .0
            .visible_assets
            .shares_items_with(&warm.1.visible_assets));

        let started = Instant::now();
        for _ in 0..ITERATIONS {
            std::hint::black_box(workspace.build_snapshot(AssetSurfaceMode::Activity));
        }
        let activity_only_ns = started.elapsed().as_nanos();

        let started = Instant::now();
        for _ in 0..ITERATIONS {
            let (activity, explorer) = workspace.build_surface_snapshots();
            assert!(activity
                .visible_assets
                .shares_items_with(&explorer.visible_assets));
            std::hint::black_box((activity, explorer));
        }
        let dual_surface_ns = started.elapsed().as_nanos();

        println!(
            "{MARKER} items={item_count} iterations={ITERATIONS} \
             activity_only_total_ns={activity_only_ns} \
             activity_only_ns_per_op={} dual_surface_total_ns={dual_surface_ns} \
             dual_surface_ns_per_op={}",
            activity_only_ns / ITERATIONS as u128,
            dual_surface_ns / ITERATIONS as u128,
        );
    }
}

fn scale_catalog(item_count: usize) -> EditorAssetCatalogSnapshotRecord {
    let child_folder_ids = (0..item_count)
        .map(|index| format!("res://folder-{index:05}"))
        .collect::<Vec<_>>();
    let direct_asset_uuids = (0..item_count)
        .map(|index| format!("asset-{index:05}"))
        .collect::<Vec<_>>();
    let mut folders = Vec::with_capacity(item_count.saturating_add(1));
    folders.push(EditorAssetFolderRecord {
        folder_id: "res://".to_string(),
        parent_folder_id: None,
        locator_prefix: "res://".to_string(),
        display_name: "Assets".to_string(),
        child_folder_ids: child_folder_ids.clone(),
        direct_asset_uuids: direct_asset_uuids.clone(),
        recursive_asset_count: item_count,
    });
    folders.extend(
        child_folder_ids
            .into_iter()
            .enumerate()
            .map(|(index, folder_id)| EditorAssetFolderRecord {
                locator_prefix: folder_id.clone(),
                folder_id,
                parent_folder_id: Some("res://".to_string()),
                display_name: format!("folder-{index:05}"),
                child_folder_ids: Vec::new(),
                direct_asset_uuids: Vec::new(),
                recursive_asset_count: 0,
            }),
    );
    let assets = direct_asset_uuids
        .into_iter()
        .enumerate()
        .map(|(index, uuid)| {
            let file_name = format!("asset-{index:05}.png");
            EditorAssetCatalogRecord {
                id: format!("source-{index:05}"),
                locator: format!("res://{file_name}"),
                display_name: format!("asset-{index:05}"),
                file_name,
                extension: "png".to_string(),
                preview_state: PreviewState::Ready,
                meta_path: format!("E:/Profile/assets/asset-{index:05}.png.zmeta"),
                preview_artifact_path: format!(
                    "E:/Profile/.zircon/cache/editor-previews/asset-{index:05}.png"
                ),
                source_mtime_unix_ms: index as u64,
                source_hash: format!("hash-{index:05}"),
                dirty: false,
                diagnostics: Vec::new(),
                direct_reference_uuids: Vec::new(),
                uuid,
                kind: ResourceKind::Texture,
            }
        })
        .collect();

    EditorAssetCatalogSnapshotRecord {
        project_name: "AssetWorkspaceProfile".to_string(),
        project_root: "E:/Profile".to_string(),
        assets_root: "E:/Profile/assets".to_string(),
        cache_root: "E:/Profile/.zircon/cache".to_string(),
        default_scene_uri: "res://main.scene.toml".to_string(),
        catalog_revision: 1,
        folders,
        assets,
    }
}

#[test]
fn asset_snapshot_and_catalog_patch_each_normalize_search_once_and_stream_parent_paths() {
    let source = include_str!("../asset_workspace_state.rs");
    let snapshot_start = source
        .find("    pub fn build_snapshot(")
        .expect("asset snapshot builder");
    let snapshot_end = source[snapshot_start..]
        .find("\n    pub(crate) fn build_surface_snapshots(")
        .map(|offset| snapshot_start + offset)
        .expect("asset surface snapshot boundary");
    let snapshot_source = &source[snapshot_start..snapshot_end];
    assert_eq!(
        snapshot_source
            .matches("self.search_query.to_ascii_lowercase()")
            .count(),
        1
    );
    let patch_start = source
        .find("    fn patch_catalog_item_generation(")
        .expect("catalog item patcher");
    let patch_end = source[patch_start..]
        .find("\n    fn patch_resource_item_generation(")
        .map(|offset| patch_start + offset)
        .expect("resource item patch boundary");
    let patch_source = &source[patch_start..patch_end];
    assert_eq!(
        patch_source
            .matches("self.search_query.to_ascii_lowercase()")
            .count(),
        1
    );
    assert!(!source.contains("split('/').collect"));

    assert_eq!(parent_folder_id_for_locator("res://mesh.glb"), "res://");
    assert_eq!(
        parent_folder_id_for_locator("res://models/props/mesh.glb"),
        "res://models/props"
    );
    assert_eq!(
        parent_folder_id_for_locator("package://tools/mesh.glb"),
        "package://tools"
    );
    assert_eq!(
        parent_folder_id_for_locator("package://tools/models/mesh.glb"),
        "package://tools/models"
    );
}

#[test]
fn dual_asset_surfaces_share_one_projection_build() {
    let source = include_str!("../../snapshot/data/editor_state_snapshot_build.rs");
    assert!(source.contains("build_surface_snapshots()"));
    assert!(!source.contains(".build_snapshot(AssetSurfaceMode::"));
}

#[test]
fn asset_workspace_state_root_stays_within_owner_review_budget() {
    let source = include_str!("../asset_workspace_state.rs");
    assert!(
        source.lines().count() <= 800,
        "asset workspace state root exceeded the 800-line owner review budget"
    );
    assert!(!source.contains("mod performance_tests {"));
}
