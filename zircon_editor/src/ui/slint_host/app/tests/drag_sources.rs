use super::*;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord, EditorAssetDetailsRecord,
    EditorAssetFolderRecord, EditorAssetReferenceRecord,
};
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::{
    AssetItemSnapshot, AssetReferenceSnapshot, AssetWorkspaceSnapshot,
};
use zircon_runtime::asset::project::PreviewState;
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::component::{
    UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata,
};

#[test]
fn hierarchy_pointer_down_arms_scene_instance_payload_for_instance_field_drop() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_hierarchy_drag_source_payload");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.hierarchy#1");

    pane_surface_host(&harness.root_ui).invoke_hierarchy_pointer_event(0, 1, 80.0, 40.0, 0.0, 0.0);

    let reference = {
        let host = harness.host.borrow();
        let payload = host
            .active_scene_drag_payload
            .as_ref()
            .expect("hierarchy row pointer down should arm a scene-instance payload");
        assert_eq!(payload.kind, UiDragPayloadKind::SceneInstance);
        assert!(payload.reference.starts_with("scene://"));
        assert!(payload.source_summary().is_some());
        payload.reference.clone()
    };

    {
        let mut host = harness.host.borrow_mut();
        host.dispatch_component_showcase_control_activated(
            "InstanceFieldDemo",
            "UiComponentShowcase/InstanceFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_scene_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("InstanceFieldDemo", "value")
            .as_deref(),
        Some(reference.as_str())
    );
}

#[test]
fn hierarchy_pointer_up_clears_scene_instance_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_hierarchy_drag_clear");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.hierarchy#1");

    pane_surface_host(&harness.root_ui).invoke_hierarchy_pointer_event(0, 1, 80.0, 40.0, 0.0, 0.0);
    assert!(harness.host.borrow().active_scene_drag_payload.is_some());

    pane_surface_host(&harness.root_ui).invoke_hierarchy_pointer_event(2, 1, 80.0, 40.0, 0.0, 0.0);
    assert!(harness.host.borrow().active_scene_drag_payload.is_none());
}

#[test]
fn object_field_drop_accepts_active_scene_instance_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_object_field_scene_payload_drop");
    {
        let mut host = harness.host.borrow_mut();
        host.active_scene_drag_payload = Some(UiDragPayload::new(
            UiDragPayloadKind::SceneInstance,
            "scene://node/42",
        ));
        host.dispatch_component_showcase_control_activated(
            "ObjectFieldDemo",
            "UiComponentShowcase/ObjectFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_scene_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("ObjectFieldDemo", "value")
            .as_deref(),
        Some("scene://node/42")
    );
}

#[test]
fn asset_field_drop_rejects_active_scene_instance_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_asset_field_rejects_scene_payload");
    {
        let mut host = harness.host.borrow_mut();
        host.active_scene_drag_payload = Some(UiDragPayload::new(
            UiDragPayloadKind::SceneInstance,
            "scene://node/42",
        ));
        host.dispatch_component_showcase_control_activated(
            "AssetFieldDemo",
            "UiComponentShowcase/AssetFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_scene_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("AssetFieldDemo", "value")
            .as_deref(),
        Some("res://textures/grid.albedo.png")
    );

    let projection = host
        .component_showcase_runtime
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = host
        .component_showcase_runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    let host_projection = host
        .component_showcase_runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();
    let node = host_projection
        .node_by_control_id("AssetFieldDemo")
        .expect("AssetFieldDemo should be projected after rejected drop");
    assert_eq!(node.validation_level.as_deref(), Some("error"));
    assert_eq!(
        node.validation_message.as_deref(),
        Some("rejected drop payload `scene-instance` for AssetField")
    );
}

#[test]
fn instance_field_drop_rejects_active_asset_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_instance_field_rejects_asset_payload");
    {
        let mut host = harness.host.borrow_mut();
        host.active_asset_drag_payload = Some(UiDragPayload::new(
            UiDragPayloadKind::Asset,
            "res://textures/grid.albedo.png",
        ));
        host.dispatch_component_showcase_control_activated(
            "InstanceFieldDemo",
            "UiComponentShowcase/InstanceFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_asset_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("InstanceFieldDemo", "value")
            .as_deref(),
        Some("scene://Root/CameraRig")
    );

    let projection = host
        .component_showcase_runtime
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = host
        .component_showcase_runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    let host_projection = host
        .component_showcase_runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();
    let node = host_projection
        .node_by_control_id("InstanceFieldDemo")
        .expect("InstanceFieldDemo should be projected after rejected drop");
    assert_eq!(node.validation_level.as_deref(), Some("error"));
    assert_eq!(
        node.validation_message.as_deref(),
        Some("rejected drop payload `asset` for InstanceField")
    );
}

#[test]
fn object_field_drop_consumes_active_object_drag_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_object_field_object_payload_drop");
    {
        let mut host = harness.host.borrow_mut();
        host.active_object_drag_payload = Some(
            UiDragPayload::new(UiDragPayloadKind::Object, "object://scene/node/7").with_source(
                UiDragSourceMetadata {
                    source_surface: "inspector".to_string(),
                    source_control_id: "InspectorHeaderPanel".to_string(),
                    locator: Some("object://scene/node/7".to_string()),
                    display_name: Some("Runtime Demo Camera".to_string()),
                    asset_kind: Some("Scene Object".to_string()),
                    ..UiDragSourceMetadata::default()
                },
            ),
        );
        host.dispatch_component_showcase_control_activated(
            "ObjectFieldDemo",
            "UiComponentShowcase/ObjectFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_object_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("ObjectFieldDemo", "value")
            .as_deref(),
        Some("object://scene/node/7")
    );
    let projection = host
        .component_showcase_runtime
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = host
        .component_showcase_runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    let host_projection = host
        .component_showcase_runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();
    assert_eq!(
        host_projection
            .node_by_control_id("ObjectFieldDemo")
            .and_then(|node| node.drop_source_summary.as_deref()),
        Some("Scene Object: Runtime Demo Camera")
    );
}

#[test]
fn inspector_pointer_down_arms_active_object_payload_for_object_field_drop() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_inspector_object_drag_source_payload");
    harness.activate_workbench_page();

    let (expected_reference, expected_summary) = {
        let host = harness.host.borrow();
        let inspector = host
            .runtime
            .editor_snapshot()
            .inspector
            .expect("default test scene should have a selected inspector object");
        (
            format!("object://scene/node/{}", inspector.id),
            format!("Scene Object: {}", inspector.name),
        )
    };

    pane_surface_host(&harness.root_ui)
        .invoke_inspector_reference_pointer_event(0, 1, 12.0, 10.0, 260.0, 180.0);

    {
        let host = harness.host.borrow();
        let payload = host
            .active_object_drag_payload
            .as_ref()
            .expect("inspector header pointer down should arm an object payload");
        assert_eq!(payload.kind, UiDragPayloadKind::Object);
        assert_eq!(payload.reference, expected_reference);
        assert_eq!(
            payload.source_summary().as_deref(),
            Some(expected_summary.as_str())
        );
        let source = payload.source.as_ref().expect("object source metadata");
        assert_eq!(source.source_surface, "inspector");
        assert_eq!(source.source_control_id, "InspectorHeaderPanel");
    }

    {
        let mut host = harness.host.borrow_mut();
        host.dispatch_component_showcase_control_activated(
            "ObjectFieldDemo",
            "UiComponentShowcase/ObjectFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_object_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("ObjectFieldDemo", "value")
            .as_deref(),
        Some(expected_reference.as_str())
    );
}

#[test]
fn inspector_pointer_up_clears_active_object_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_inspector_object_drag_clear");
    harness.activate_workbench_page();

    pane_surface_host(&harness.root_ui)
        .invoke_inspector_reference_pointer_event(0, 1, 12.0, 10.0, 260.0, 180.0);
    assert!(harness.host.borrow().active_object_drag_payload.is_some());

    pane_surface_host(&harness.root_ui)
        .invoke_inspector_reference_pointer_event(2, 1, 12.0, 10.0, 260.0, 180.0);
    assert!(harness.host.borrow().active_object_drag_payload.is_none());
}

#[test]
fn asset_drag_payload_resolves_visible_asset_metadata() {
    let mut snapshot = AssetWorkspaceSnapshot::default();
    snapshot.visible_assets.push(AssetItemSnapshot {
        uuid: "asset-uuid-1".to_string(),
        locator: "res://textures/grid.albedo.png".to_string(),
        display_name: "Grid Albedo".to_string(),
        file_name: "grid.albedo.png".to_string(),
        extension: "png".to_string(),
        kind: ResourceKind::Texture,
        preview_artifact_path: String::new(),
        dirty: false,
        diagnostics: Vec::new(),
        selected: false,
        resource_state: None,
        resource_revision: None,
    });

    let payload = super::super::asset_drag_payload::asset_drag_payload_from_snapshot(
        "browser",
        "asset-uuid-1",
        &snapshot,
    )
    .expect("visible asset should create a drag payload");

    assert_eq!(payload.reference, "res://textures/grid.albedo.png");
    assert_eq!(payload.kind, UiDragPayloadKind::Asset);
    assert_eq!(
        payload.source_summary().as_deref(),
        Some("Texture: Grid Albedo")
    );
    let source = payload.source.as_ref().expect("source metadata");
    assert_eq!(source.source_surface, "browser");
    assert_eq!(source.source_control_id, "AssetBrowserContentPanel");
    assert_eq!(source.asset_uuid.as_deref(), Some("asset-uuid-1"));
    assert_eq!(
        source.locator.as_deref(),
        Some("res://textures/grid.albedo.png")
    );
    assert_eq!(source.display_name.as_deref(), Some("Grid Albedo"));
    assert_eq!(source.asset_kind.as_deref(), Some("Texture"));
    assert_eq!(source.extension.as_deref(), Some("png"));

    let activity_payload = super::super::asset_drag_payload::asset_drag_payload_from_snapshot(
        "activity",
        "asset-uuid-1",
        &snapshot,
    )
    .expect("visible activity asset should create a drag payload");
    let activity_source = activity_payload.source.as_ref().expect("source metadata");
    assert_eq!(activity_source.source_surface, "activity");
    assert_eq!(
        activity_source.source_control_id,
        "AssetsActivityContentPanel"
    );

    assert!(
        super::super::asset_drag_payload::asset_drag_payload_from_snapshot(
            "browser",
            "missing-uuid",
            &snapshot,
        )
        .is_none()
    );
}

#[test]
fn asset_drag_payload_resolves_reference_panel_metadata() {
    let mut snapshot = AssetWorkspaceSnapshot::default();
    snapshot.selection.references.push(AssetReferenceSnapshot {
        uuid: "material-uuid-1".to_string(),
        locator: "res://materials/runtime_demo.mat".to_string(),
        display_name: "Runtime Demo".to_string(),
        kind: Some(ResourceKind::Material),
        known_project_asset: true,
    });
    snapshot.selection.used_by.push(AssetReferenceSnapshot {
        uuid: "external-uuid-1".to_string(),
        locator: "file:///vendor/texture.png".to_string(),
        display_name: "External Texture".to_string(),
        kind: Some(ResourceKind::Texture),
        known_project_asset: false,
    });

    let payload = super::super::asset_drag_payload::asset_drag_payload_from_reference(
        "browser",
        "references",
        "material-uuid-1",
        &snapshot,
    )
    .expect("known project reference should create a drag payload");

    assert_eq!(payload.kind, UiDragPayloadKind::Asset);
    assert_eq!(payload.reference, "res://materials/runtime_demo.mat");
    assert_eq!(
        payload.source_summary().as_deref(),
        Some("Material: Runtime Demo")
    );
    let source = payload.source.as_ref().expect("source metadata");
    assert_eq!(source.source_surface, "browser.references");
    assert_eq!(source.source_control_id, "AssetBrowserReferenceLeftPanel");
    assert_eq!(source.asset_uuid.as_deref(), Some("material-uuid-1"));
    assert_eq!(
        source.locator.as_deref(),
        Some("res://materials/runtime_demo.mat")
    );
    assert_eq!(source.display_name.as_deref(), Some("Runtime Demo"));
    assert_eq!(source.asset_kind.as_deref(), Some("Material"));
    assert_eq!(source.extension.as_deref(), Some("mat"));

    assert!(
        super::super::asset_drag_payload::asset_drag_payload_from_reference(
            "browser",
            "used_by",
            "external-uuid-1",
            &snapshot,
        )
        .is_none(),
        "external references should not become project asset drag payloads"
    );
    assert!(
        super::super::asset_drag_payload::asset_drag_payload_from_reference(
            "browser",
            "unknown",
            "material-uuid-1",
            &snapshot,
        )
        .is_none()
    );
}

#[test]
fn asset_field_drop_consumes_active_asset_drag_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_asset_field_real_payload_drop");
    {
        let mut host = harness.host.borrow_mut();
        host.active_asset_drag_payload = Some(
            UiDragPayload::new(UiDragPayloadKind::Asset, "res://textures/grid.albedo.png")
                .with_source(UiDragSourceMetadata::asset(
                    "browser",
                    "AssetBrowserContentPanel",
                    "asset-uuid-1",
                    "res://textures/grid.albedo.png",
                    "Grid Albedo",
                    "Texture",
                    "png",
                )),
        );
        host.dispatch_component_showcase_control_activated(
            "AssetFieldDemo",
            "UiComponentShowcase/AssetFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_asset_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("AssetFieldDemo", "value")
            .as_deref(),
        Some("res://textures/grid.albedo.png")
    );
}

#[test]
fn asset_field_drop_without_active_payload_uses_showcase_default_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_asset_field_default_payload_drop");
    {
        let mut host = harness.host.borrow_mut();
        assert!(host.active_asset_drag_payload.is_none());
        host.dispatch_component_showcase_control_activated(
            "AssetFieldDemo",
            "UiComponentShowcase/AssetFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_asset_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("AssetFieldDemo", "value")
            .as_deref(),
        Some("res://materials/runtime_demo.mat")
    );
}

fn asset_drag_source_catalog() -> EditorAssetCatalogSnapshotRecord {
    EditorAssetCatalogSnapshotRecord {
        project_name: "Sandbox".to_string(),
        project_root: "E:/Sandbox".to_string(),
        assets_root: "E:/Sandbox/assets".to_string(),
        library_root: "E:/Sandbox/library".to_string(),
        default_scene_uri: "res://scenes/main.scene.toml".to_string(),
        catalog_revision: 1,
        folders: vec![EditorAssetFolderRecord {
            folder_id: "res://".to_string(),
            parent_folder_id: None,
            locator_prefix: "res://".to_string(),
            display_name: "Assets".to_string(),
            child_folder_ids: Vec::new(),
            direct_asset_uuids: vec!["asset-uuid-1".to_string()],
            recursive_asset_count: 1,
        }],
        assets: vec![EditorAssetCatalogRecord {
            uuid: "asset-uuid-1".to_string(),
            id: "asset-id-1".to_string(),
            locator: "res://grid.albedo.png".to_string(),
            kind: ResourceKind::Texture,
            display_name: "Grid Albedo".to_string(),
            file_name: "grid.albedo.png".to_string(),
            extension: "png".to_string(),
            preview_state: PreviewState::Ready,
            meta_path: "E:/Sandbox/assets/grid.albedo.png.meta.toml".to_string(),
            preview_artifact_path: "E:/Sandbox/library/editor-previews/grid.png".to_string(),
            source_mtime_unix_ms: 1,
            source_hash: "grid".to_string(),
            dirty: false,
            diagnostics: Vec::new(),
            direct_reference_uuids: Vec::new(),
        }],
    }
}

fn asset_drag_source_catalog_with_reference() -> (
    EditorAssetCatalogSnapshotRecord,
    EditorAssetCatalogRecord,
    EditorAssetCatalogRecord,
) {
    let mut catalog = asset_drag_source_catalog();
    catalog.folders[0]
        .direct_asset_uuids
        .push("asset-uuid-2".to_string());
    catalog.folders[0].recursive_asset_count = 2;
    catalog.assets[0]
        .direct_reference_uuids
        .push("asset-uuid-2".to_string());
    let material_asset = EditorAssetCatalogRecord {
        uuid: "asset-uuid-2".to_string(),
        id: "asset-id-2".to_string(),
        locator: "res://materials/runtime_demo.mat".to_string(),
        kind: ResourceKind::Material,
        display_name: "Runtime Demo".to_string(),
        file_name: "runtime_demo.mat".to_string(),
        extension: "mat".to_string(),
        preview_state: PreviewState::Ready,
        meta_path: "E:/Sandbox/assets/materials/runtime_demo.mat.meta.toml".to_string(),
        preview_artifact_path: "E:/Sandbox/library/editor-previews/runtime_demo.png".to_string(),
        source_mtime_unix_ms: 2,
        source_hash: "runtime-demo".to_string(),
        dirty: false,
        diagnostics: Vec::new(),
        direct_reference_uuids: Vec::new(),
    };
    let source_asset = catalog.assets[0].clone();
    catalog.assets.push(material_asset.clone());
    (catalog, source_asset, material_asset)
}

#[test]
fn asset_content_pointer_down_arms_active_asset_drag_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_asset_drag_source_payload");
    let _asset_browser = harness.open_view("editor.asset_browser");

    {
        let mut host = harness.host.borrow_mut();
        host.runtime.sync_asset_catalog(asset_drag_source_catalog());
        host.mark_layout_dirty();
        host.refresh_ui();
    }

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "browser".into(),
        0,
        2,
        96.0,
        96.0,
        0.0,
        0.0,
    );
    assert!(
        harness.host.borrow().active_asset_drag_payload.is_none(),
        "right-button pointer down should not arm an active payload"
    );

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "browser".into(),
        0,
        1,
        96.0,
        96.0,
        0.0,
        0.0,
    );

    let host = harness.host.borrow();
    let payload = host
        .active_asset_drag_payload
        .as_ref()
        .expect("asset row pointer down should arm an active payload");
    assert_eq!(payload.kind, UiDragPayloadKind::Asset);
    assert!(payload.reference.starts_with("res://"));
    assert!(payload.source_summary().is_some());
    drop(host);

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "browser".into(),
        2,
        1,
        96.0,
        96.0,
        0.0,
        0.0,
    );
    assert!(
        harness.host.borrow().active_asset_drag_payload.is_none(),
        "left-button pointer up should clear the active payload"
    );
}

#[test]
fn asset_reference_pointer_down_arms_active_asset_drag_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_asset_reference_drag_source_payload");
    let _asset_browser = harness.open_view("editor.asset_browser");
    let (catalog, source_asset, reference_asset) = asset_drag_source_catalog_with_reference();

    {
        let mut host = harness.host.borrow_mut();
        host.runtime.sync_asset_catalog(catalog);
        host.mark_layout_dirty();
        host.refresh_ui();
    }

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_clicked(
        "browser".into(),
        96.0,
        96.0,
        0.0,
        0.0,
    );

    {
        let mut host = harness.host.borrow_mut();
        host.runtime
            .sync_asset_details(Some(EditorAssetDetailsRecord {
                asset: source_asset,
                direct_references: vec![EditorAssetReferenceRecord {
                    uuid: reference_asset.uuid.clone(),
                    locator: reference_asset.locator.clone(),
                    display_name: reference_asset.display_name.clone(),
                    kind: Some(reference_asset.kind),
                    known_project_asset: true,
                }],
                referenced_by: Vec::new(),
                editor_adapter: None,
            }));
        host.mark_layout_dirty();
        host.refresh_ui();
    }

    pane_surface_host(&harness.root_ui).invoke_asset_reference_pointer_event(
        "browser".into(),
        "references".into(),
        0,
        2,
        16.0,
        44.0,
        260.0,
        160.0,
    );
    assert!(
        harness.host.borrow().active_asset_drag_payload.is_none(),
        "right-button reference pointer down should not arm an active payload"
    );

    pane_surface_host(&harness.root_ui).invoke_asset_reference_pointer_event(
        "browser".into(),
        "references".into(),
        0,
        1,
        16.0,
        44.0,
        260.0,
        160.0,
    );

    let host = harness.host.borrow();
    let payload = host
        .active_asset_drag_payload
        .as_ref()
        .expect("known reference row pointer down should arm an active payload");
    assert_eq!(payload.kind, UiDragPayloadKind::Asset);
    assert_eq!(payload.reference, "res://materials/runtime_demo.mat");
    assert_eq!(
        payload.source_summary().as_deref(),
        Some("Material: Runtime Demo")
    );
    let source = payload.source.as_ref().expect("source metadata");
    assert_eq!(source.source_surface, "browser.references");
    assert_eq!(source.source_control_id, "AssetBrowserReferenceLeftPanel");
    drop(host);

    pane_surface_host(&harness.root_ui).invoke_asset_reference_pointer_event(
        "browser".into(),
        "references".into(),
        2,
        1,
        16.0,
        44.0,
        260.0,
        160.0,
    );
    assert!(
        harness.host.borrow().active_asset_drag_payload.is_none(),
        "left-button reference pointer up should clear the active payload"
    );
}

#[test]
fn asset_browser_pointer_drop_applies_real_payload_to_showcase_asset_field() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_asset_browser_real_payload_drop");
    let _asset_browser = harness.open_view("editor.asset_browser");

    {
        let mut host = harness.host.borrow_mut();
        host.runtime.sync_asset_catalog(asset_drag_source_catalog());
        host.mark_layout_dirty();
        host.refresh_ui();
    }

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "browser".into(),
        0,
        1,
        96.0,
        96.0,
        0.0,
        0.0,
    );

    {
        let mut host = harness.host.borrow_mut();
        let payload = host
            .active_asset_drag_payload
            .as_ref()
            .expect("visible asset row pointer down should arm an active payload");
        assert_eq!(payload.reference, "res://grid.albedo.png");
        assert_eq!(
            payload.source_summary().as_deref(),
            Some("Texture: Grid Albedo")
        );

        host.dispatch_component_showcase_control_activated(
            "AssetFieldDemo",
            "UiComponentShowcase/AssetFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_asset_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("AssetFieldDemo", "value")
            .as_deref(),
        Some("res://grid.albedo.png")
    );
    let projection = host
        .component_showcase_runtime
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = host
        .component_showcase_runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    let host_projection = host
        .component_showcase_runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();
    assert_eq!(
        host_projection
            .node_by_control_id("AssetFieldDemo")
            .and_then(|node| node.drop_source_summary.as_deref()),
        Some("Texture: Grid Albedo")
    );
}

#[test]
fn asset_content_pointer_unknown_surface_clears_active_asset_drag_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_asset_drag_unknown_surface_clear");
    let _asset_browser = harness.open_view("editor.asset_browser");

    {
        let mut host = harness.host.borrow_mut();
        host.runtime.sync_asset_catalog(asset_drag_source_catalog());
        host.mark_layout_dirty();
        host.refresh_ui();
    }

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "browser".into(),
        0,
        1,
        96.0,
        96.0,
        0.0,
        0.0,
    );
    assert!(harness.host.borrow().active_asset_drag_payload.is_some());

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "unknown".into(),
        0,
        1,
        96.0,
        96.0,
        0.0,
        0.0,
    );
    assert!(
        harness.host.borrow().active_asset_drag_payload.is_none(),
        "unknown asset surface should clear stale active payload"
    );
}
