use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord, EditorAssetDetailsRecord,
    EditorAssetFolderRecord, EditorAssetReferenceRecord,
};
use zircon_runtime::asset::project::PreviewState;
use zircon_runtime::core::resource::{
    ResourceId, ResourceKind, ResourceLocator, ResourceRecord, ResourceState,
};

use crate::ui::workbench::project::AssetWorkspaceState;
use crate::ui::workbench::snapshot::{AssetSurfaceMode, AssetUtilityTab, AssetViewMode};

#[test]
fn asset_workspace_builds_folder_tree_and_visible_content_from_catalog() {
    let mut workspace = AssetWorkspaceState::default();
    workspace.sync_catalog(sample_catalog());
    workspace.select_folder("res://materials");
    workspace.select_asset(Some("11111111-1111-1111-1111-111111111111".to_string()));
    workspace.sync_selected_details(Some(sample_material_details()));
    workspace.sync_resources(vec![sample_resource_status(
        "res://materials/grid.material.toml",
        ResourceKind::Material,
        4,
        ResourceState::Ready,
    )]);

    let snapshot = workspace.build_snapshot(AssetSurfaceMode::Activity);

    assert_eq!(
        snapshot.selected_folder_id.as_deref(),
        Some("res://materials")
    );
    assert_eq!(
        snapshot.selected_asset_uuid.as_deref(),
        Some("11111111-1111-1111-1111-111111111111")
    );
    assert_eq!(snapshot.visible_folders.len(), 0);
    assert_eq!(snapshot.visible_assets.len(), 1);
    assert_eq!(
        snapshot.visible_assets[0].locator,
        "res://materials/grid.material.toml"
    );
    assert_eq!(
        snapshot.selection.references[0].locator,
        "res://textures/checker.png"
    );
    assert_eq!(snapshot.selection.resource_revision, Some(4));
}

#[test]
fn asset_workspace_shares_selection_but_keeps_surface_preferences_separate() {
    let mut workspace = AssetWorkspaceState::default();
    workspace.sync_catalog(sample_catalog());
    workspace.select_folder("res://scenes");
    workspace.select_asset(Some("22222222-2222-2222-2222-222222222222".to_string()));
    workspace.set_activity_view_mode(AssetViewMode::List);
    workspace.set_browser_view_mode(AssetViewMode::Thumbnail);
    workspace.set_activity_utility_tab(AssetUtilityTab::References);
    workspace.set_browser_utility_tab(AssetUtilityTab::Metadata);

    let activity = workspace.build_snapshot(AssetSurfaceMode::Activity);
    let browser = workspace.build_snapshot(AssetSurfaceMode::Explorer);

    assert_eq!(activity.selected_asset_uuid, browser.selected_asset_uuid);
    assert_eq!(activity.view_mode, AssetViewMode::List);
    assert_eq!(browser.view_mode, AssetViewMode::Thumbnail);
    assert_eq!(activity.utility_tab, AssetUtilityTab::References);
    assert_eq!(browser.utility_tab, AssetUtilityTab::Metadata);
}

#[test]
fn asset_workspace_reference_navigation_relocates_selection() {
    let mut workspace = AssetWorkspaceState::default();
    workspace.sync_catalog(sample_catalog());

    workspace.navigate_to_asset("11111111-1111-1111-1111-111111111111");

    assert_eq!(workspace.selected_folder_id(), "res://materials");
    assert_eq!(
        workspace.selected_asset_uuid(),
        Some("11111111-1111-1111-1111-111111111111")
    );
}

#[test]
fn asset_workspace_filters_physics_and_animation_asset_kinds() {
    let mut workspace = AssetWorkspaceState::default();
    workspace.sync_catalog(sample_catalog());

    workspace.select_folder("res://animation");
    workspace.set_kind_filter(Some(ResourceKind::AnimationSequence));
    let animation_snapshot = workspace.build_snapshot(AssetSurfaceMode::Activity);

    assert_eq!(animation_snapshot.visible_assets.len(), 1);
    assert_eq!(
        animation_snapshot.visible_assets[0].locator,
        "res://animation/hero.sequence.zranim"
    );
    assert_eq!(
        animation_snapshot.visible_assets[0].kind,
        ResourceKind::AnimationSequence
    );

    workspace.select_folder("res://physics/materials");
    workspace.set_kind_filter(Some(ResourceKind::PhysicsMaterial));
    let physics_snapshot = workspace.build_snapshot(AssetSurfaceMode::Activity);

    assert_eq!(physics_snapshot.visible_assets.len(), 1);
    assert_eq!(
        physics_snapshot.visible_assets[0].locator,
        "res://physics/materials/default.physics_material.toml"
    );
    assert_eq!(
        physics_snapshot.visible_assets[0].kind,
        ResourceKind::PhysicsMaterial
    );
}

pub(super) fn sample_catalog() -> EditorAssetCatalogSnapshotRecord {
    EditorAssetCatalogSnapshotRecord {
        project_name: "Sandbox".to_string(),
        project_root: "E:/Sandbox".to_string(),
        assets_root: "E:/Sandbox/assets".to_string(),
        library_root: "E:/Sandbox/library".to_string(),
        default_scene_uri: "res://scenes/main.scene.toml".to_string(),
        catalog_revision: 3,
        folders: vec![
            EditorAssetFolderRecord {
                folder_id: "res://".to_string(),
                parent_folder_id: None,
                locator_prefix: "res://".to_string(),
                display_name: "Assets".to_string(),
                child_folder_ids: vec![
                    "res://materials".to_string(),
                    "res://scenes".to_string(),
                    "res://textures".to_string(),
                    "res://animation".to_string(),
                    "res://physics".to_string(),
                ],
                direct_asset_uuids: Vec::new(),
                recursive_asset_count: 5,
            },
            EditorAssetFolderRecord {
                folder_id: "res://materials".to_string(),
                parent_folder_id: Some("res://".to_string()),
                locator_prefix: "res://materials".to_string(),
                display_name: "materials".to_string(),
                child_folder_ids: Vec::new(),
                direct_asset_uuids: vec!["11111111-1111-1111-1111-111111111111".to_string()],
                recursive_asset_count: 1,
            },
            EditorAssetFolderRecord {
                folder_id: "res://scenes".to_string(),
                parent_folder_id: Some("res://".to_string()),
                locator_prefix: "res://scenes".to_string(),
                display_name: "scenes".to_string(),
                child_folder_ids: Vec::new(),
                direct_asset_uuids: vec!["22222222-2222-2222-2222-222222222222".to_string()],
                recursive_asset_count: 1,
            },
            EditorAssetFolderRecord {
                folder_id: "res://textures".to_string(),
                parent_folder_id: Some("res://".to_string()),
                locator_prefix: "res://textures".to_string(),
                display_name: "textures".to_string(),
                child_folder_ids: Vec::new(),
                direct_asset_uuids: vec!["33333333-3333-3333-3333-333333333333".to_string()],
                recursive_asset_count: 1,
            },
            EditorAssetFolderRecord {
                folder_id: "res://animation".to_string(),
                parent_folder_id: Some("res://".to_string()),
                locator_prefix: "res://animation".to_string(),
                display_name: "animation".to_string(),
                child_folder_ids: Vec::new(),
                direct_asset_uuids: vec!["44444444-4444-4444-4444-444444444444".to_string()],
                recursive_asset_count: 1,
            },
            EditorAssetFolderRecord {
                folder_id: "res://physics".to_string(),
                parent_folder_id: Some("res://".to_string()),
                locator_prefix: "res://physics".to_string(),
                display_name: "physics".to_string(),
                child_folder_ids: vec!["res://physics/materials".to_string()],
                direct_asset_uuids: Vec::new(),
                recursive_asset_count: 1,
            },
            EditorAssetFolderRecord {
                folder_id: "res://physics/materials".to_string(),
                parent_folder_id: Some("res://physics".to_string()),
                locator_prefix: "res://physics/materials".to_string(),
                display_name: "materials".to_string(),
                child_folder_ids: Vec::new(),
                direct_asset_uuids: vec!["55555555-5555-5555-5555-555555555555".to_string()],
                recursive_asset_count: 1,
            },
        ],
        assets: vec![
            EditorAssetCatalogRecord {
                uuid: "11111111-1111-1111-1111-111111111111".to_string(),
                id: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa".to_string(),
                locator: "res://materials/grid.material.toml".to_string(),
                kind: ResourceKind::Material,
                display_name: "grid.material".to_string(),
                file_name: "grid.material.toml".to_string(),
                extension: "toml".to_string(),
                preview_state: PreviewState::Ready,
                meta_path: "E:/Sandbox/assets/materials/grid.material.toml.meta.toml".to_string(),
                preview_artifact_path: "E:/Sandbox/library/editor-previews/grid.png".to_string(),
                source_mtime_unix_ms: 10,
                source_hash: "mat".to_string(),
                dirty: false,
                diagnostics: Vec::new(),
                direct_reference_uuids: vec!["33333333-3333-3333-3333-333333333333".to_string()],
            },
            EditorAssetCatalogRecord {
                uuid: "22222222-2222-2222-2222-222222222222".to_string(),
                id: "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb".to_string(),
                locator: "res://scenes/main.scene.toml".to_string(),
                kind: ResourceKind::Scene,
                display_name: "main.scene".to_string(),
                file_name: "main.scene.toml".to_string(),
                extension: "toml".to_string(),
                preview_state: PreviewState::Dirty,
                meta_path: "E:/Sandbox/assets/scenes/main.scene.toml.meta.toml".to_string(),
                preview_artifact_path: "E:/Sandbox/library/editor-previews/main.png".to_string(),
                source_mtime_unix_ms: 20,
                source_hash: "scene".to_string(),
                dirty: true,
                diagnostics: vec!["preview dirty".to_string()],
                direct_reference_uuids: vec!["11111111-1111-1111-1111-111111111111".to_string()],
            },
            EditorAssetCatalogRecord {
                uuid: "33333333-3333-3333-3333-333333333333".to_string(),
                id: "cccccccc-cccc-cccc-cccc-cccccccccccc".to_string(),
                locator: "res://textures/checker.png".to_string(),
                kind: ResourceKind::Texture,
                display_name: "checker".to_string(),
                file_name: "checker.png".to_string(),
                extension: "png".to_string(),
                preview_state: PreviewState::Ready,
                meta_path: "E:/Sandbox/assets/textures/checker.png.meta.toml".to_string(),
                preview_artifact_path: "E:/Sandbox/library/editor-previews/checker.png".to_string(),
                source_mtime_unix_ms: 30,
                source_hash: "tex".to_string(),
                dirty: false,
                diagnostics: Vec::new(),
                direct_reference_uuids: Vec::new(),
            },
            EditorAssetCatalogRecord {
                uuid: "44444444-4444-4444-4444-444444444444".to_string(),
                id: "dddddddd-dddd-dddd-dddd-dddddddddddd".to_string(),
                locator: "res://animation/hero.sequence.zranim".to_string(),
                kind: ResourceKind::AnimationSequence,
                display_name: "hero.sequence".to_string(),
                file_name: "hero.sequence.zranim".to_string(),
                extension: "zranim".to_string(),
                preview_state: PreviewState::Dirty,
                meta_path: "E:/Sandbox/assets/animation/hero.sequence.zranim.meta.toml".to_string(),
                preview_artifact_path: "E:/Sandbox/library/editor-previews/hero-sequence.png"
                    .to_string(),
                source_mtime_unix_ms: 40,
                source_hash: "anim-seq".to_string(),
                dirty: true,
                diagnostics: vec!["sequence preview pending".to_string()],
                direct_reference_uuids: Vec::new(),
            },
            EditorAssetCatalogRecord {
                uuid: "55555555-5555-5555-5555-555555555555".to_string(),
                id: "eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee".to_string(),
                locator: "res://physics/materials/default.physics_material.toml".to_string(),
                kind: ResourceKind::PhysicsMaterial,
                display_name: "default.physics_material".to_string(),
                file_name: "default.physics_material.toml".to_string(),
                extension: "toml".to_string(),
                preview_state: PreviewState::Ready,
                meta_path:
                    "E:/Sandbox/assets/physics/materials/default.physics_material.toml.meta.toml"
                        .to_string(),
                preview_artifact_path: "E:/Sandbox/library/editor-previews/default-physics.png"
                    .to_string(),
                source_mtime_unix_ms: 50,
                source_hash: "phys".to_string(),
                dirty: false,
                diagnostics: Vec::new(),
                direct_reference_uuids: Vec::new(),
            },
        ],
    }
}

pub(super) fn sample_material_details() -> EditorAssetDetailsRecord {
    EditorAssetDetailsRecord {
        asset: sample_catalog().assets[0].clone(),
        direct_references: vec![EditorAssetReferenceRecord {
            uuid: "33333333-3333-3333-3333-333333333333".to_string(),
            locator: "res://textures/checker.png".to_string(),
            display_name: "checker".to_string(),
            kind: Some(ResourceKind::Texture),
            known_project_asset: true,
        }],
        referenced_by: vec![EditorAssetReferenceRecord {
            uuid: "22222222-2222-2222-2222-222222222222".to_string(),
            locator: "res://scenes/main.scene.toml".to_string(),
            display_name: "main.scene".to_string(),
            kind: Some(ResourceKind::Scene),
            known_project_asset: true,
        }],
        editor_adapter: Some("material.pbr".to_string()),
    }
}

pub(super) fn sample_resource_status(
    locator: &str,
    kind: ResourceKind,
    revision: u64,
    state: ResourceState,
) -> ResourceRecord {
    let locator = ResourceLocator::parse(locator).unwrap();
    ResourceRecord {
        id: ResourceId::from_locator(&locator),
        kind,
        primary_locator: locator,
        artifact_locator: None,
        revision,
        state,
        dependency_ids: Vec::new(),
        diagnostics: Vec::new(),
        source_hash: String::new(),
        importer_version: 0,
        config_hash: String::new(),
    }
}
