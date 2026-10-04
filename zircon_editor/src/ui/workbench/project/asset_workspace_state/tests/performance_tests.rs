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

use super::{
    contains_ascii_case_insensitive, locator_parent_matches_folder, parent_folder_id_for_locator,
    AssetWorkspaceItemProjectionInput, AssetWorkspaceState,
};

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
fn search_query_normalization_is_cached_when_the_query_changes() {
    let mut workspace = AssetWorkspaceState::default();
    assert_eq!(workspace.normalized_search_query, "");

    assert!(workspace.set_search_query("RoBoT"));
    assert_eq!(workspace.normalized_search_query, "robot");
    assert!(!workspace.set_search_query("RoBoT"));
    assert_eq!(workspace.normalized_search_query, "robot");

    assert!(workspace.set_search_query("MATERIAL"));
    assert_eq!(workspace.normalized_search_query, "material");
}

#[test]
fn asset_filter_matching_preserves_ascii_case_insensitive_contains_semantics() {
    assert!(contains_ascii_case_insensitive("Robot Arm", "robot"));
    assert!(contains_ascii_case_insensitive(
        "res://MESH/ROBOT.GLB",
        "mesh/robot"
    ));
    assert!(contains_ascii_case_insensitive("模型/材质", "模型"));
    assert!(contains_ascii_case_insensitive("anything", ""));
    assert!(!contains_ascii_case_insensitive("Robot Arm", "material"));
}

#[test]
fn borrowed_filter_matching_preserves_utf8_substring_boundaries() {
    let cases = [
        ("aé中", "é"),
        ("aé中", "中"),
        ("Aé", "aé"),
        ("e\u{301}", "e\u{301}"),
        ("é中", "中é"),
    ];

    for (haystack, needle) in cases {
        assert_eq!(
            contains_ascii_case_insensitive(haystack, needle),
            haystack
                .to_ascii_lowercase()
                .contains(&needle.to_ascii_lowercase()),
            "haystack={haystack:?} needle={needle:?}"
        );
    }
}

#[test]
fn asset_filter_and_folder_tree_paths_avoid_per_candidate_lowercase_allocations() {
    let source = include_str!("../../asset_workspace_state.rs");
    let folder_start = source
        .find("fn folder_matches_search(")
        .expect("folder filter helper");
    let asset_start = source
        .find("fn asset_matches_filters(")
        .expect("asset filter helper");
    let helper_start = source
        .find("fn contains_ascii_case_insensitive(")
        .expect("borrowed ascii matcher");
    let filter_source = &source[folder_start..helper_start];
    assert!(filter_source.contains("contains_ascii_case_insensitive"));
    assert_eq!(filter_source.matches("to_ascii_lowercase()").count(), 0);
    assert!(source[asset_start..helper_start].contains("asset.display_name"));
    assert!(source[asset_start..helper_start].contains("asset.file_name"));
    assert!(source[asset_start..helper_start].contains("asset.locator"));
}

#[test]
fn borrowed_folder_matching_preserves_locator_parent_semantics() {
    let cases = [
        ("res://mesh.glb", "res://", true),
        ("res://mesh.glb", "res://models", false),
        ("res://models/props/mesh.glb", "res://models/props", true),
        ("package://tools/mesh.glb", "package://tools", true),
        (
            "package://tools/models/mesh.glb",
            "package://tools/models",
            true,
        ),
        ("package://tool", "package://tool", true),
        ("package://tool", "package://", false),
        ("mesh.glb", "res://", true),
        ("mesh.glb", "", false),
    ];

    for (locator, folder_id, expected) in cases {
        assert_eq!(
            locator_parent_matches_folder(locator, folder_id),
            expected,
            "locator={locator:?} folder={folder_id:?}"
        );
    }
}

#[test]
fn borrowed_folder_matching_avoids_owned_parent_projection_in_asset_filter() {
    let source = include_str!("../../asset_workspace_state.rs");
    let start = source
        .find("fn asset_belongs_to_folder(")
        .expect("asset folder predicate");
    let end = source[start..]
        .find("\nfn parent_folder_id_for_locator(")
        .map(|offset| start + offset)
        .expect("owned parent projection boundary");
    let predicate = &source[start..end];

    assert!(predicate.contains("locator_parent_matches_folder"));
    assert!(!predicate.contains("parent_folder_id_for_locator"));
    assert!(predicate.contains("rsplit_once('/')"));
}

#[test]
fn folder_tree_projection_reserves_catalog_capacity_before_materialization() {
    let source = include_str!("../../asset_workspace_state.rs");
    let start = source
        .find("fn build_folder_tree(")
        .expect("folder tree builder");
    let end = source[start..]
        .find("\nfn append_folder_branch(")
        .map(|offset| start + offset)
        .expect("folder tree branch boundary");
    let implementation = &source[start..end];
    assert!(implementation.contains("with_capacity(folders.len())"));
    assert!(implementation.contains("Vec::with_capacity(folders.len())"));
}

#[test]
fn cached_item_projection_input_matches_borrowed_workspace_state() {
    let input = AssetWorkspaceItemProjectionInput {
        projection_generation: 42,
        selected_folder_id: "res://models".to_string(),
        search_query: "robot".to_string(),
        kind_filter: Some(ResourceKind::Mesh),
    };

    assert!(input.matches_current(42, "res://models", "robot", Some(ResourceKind::Mesh),));
    assert!(!input.matches_current(43, "res://models", "robot", Some(ResourceKind::Mesh),));
    assert!(!input.matches_current(42, "res://textures", "robot", Some(ResourceKind::Mesh),));
    assert!(!input.matches_current(42, "res://models", "material", Some(ResourceKind::Mesh),));
    assert!(!input.matches_current(42, "res://models", "robot", Some(ResourceKind::Texture),));
}

#[test]
fn stable_item_projection_cache_checks_borrowed_key_before_cloning_it() {
    let source = include_str!("../../asset_workspace_state.rs");
    let visible_start = source
        .find("    fn visible_asset_generation(")
        .expect("visible asset generation");
    let visible_end = source[visible_start..]
        .find("\n    fn patch_catalog_item_generation(")
        .map(|offset| visible_start + offset)
        .expect("catalog patch boundary");
    let visible = &source[visible_start..visible_end];
    let cache_check = visible
        .find(".matches_current(")
        .expect("borrowed cache key comparison");
    let owned_input = visible
        .find("let input = AssetWorkspaceItemProjectionInput")
        .expect("owned cache key rebuild");

    assert!(cache_check < owned_input);
    assert!(visible.contains("&self.selected_folder_id"));
    assert!(visible.contains("&self.search_query"));
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
    assert!(!include_str!("../../asset_workspace_state.rs").contains(&legacy_hasher));
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
fn asset_snapshot_and_catalog_patch_reuse_normalized_search_and_stream_parent_paths() {
    let source = include_str!("../../asset_workspace_state.rs");
    let snapshot_start = source
        .find("    pub fn build_snapshot(")
        .expect("asset snapshot builder");
    let snapshot_end = source[snapshot_start..]
        .find("\n    pub(crate) fn build_surface_snapshots(")
        .map(|offset| snapshot_start + offset)
        .expect("asset surface snapshot boundary");
    let snapshot_source = &source[snapshot_start..snapshot_end];
    assert_eq!(snapshot_source.matches("to_ascii_lowercase()").count(), 0);
    assert!(snapshot_source.contains("self.normalized_search_query.as_str()"));
    let patch_start = source
        .find("    fn patch_catalog_item_generation(")
        .expect("catalog item patcher");
    let patch_end = source[patch_start..]
        .find("\n    fn patch_resource_item_generation(")
        .map(|offset| patch_start + offset)
        .expect("resource item patch boundary");
    let patch_source = &source[patch_start..patch_end];
    assert_eq!(patch_source.matches("to_ascii_lowercase()").count(), 0);
    assert!(patch_source.contains("self.normalized_search_query.as_str()"));
    assert!(patch_source
        .contains("let mut replacements = Vec::with_capacity(changed_asset_uuids.len());"));
    let resource_patch_source = &source[patch_end..];
    assert!(resource_patch_source
        .contains("let mut replacements = Vec::with_capacity(changed_locators.len());"));
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
    let source = include_str!("../../../snapshot/data/editor_state_snapshot_build.rs");
    assert!(source.contains("build_surface_snapshots()"));
    assert!(!source.contains(".build_snapshot(AssetSurfaceMode::"));
}

#[test]
fn asset_workspace_state_root_stays_within_owner_review_budget() {
    let source = include_str!("../../asset_workspace_state.rs");
    assert!(
        source.lines().count() <= 800,
        "asset workspace state root exceeded the 800-line owner review budget"
    );
    assert!(!source.contains("mod performance_tests {"));
}
