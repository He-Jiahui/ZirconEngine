use super::*;

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
        asset_type: crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::from_resource_kind(
            ResourceKind::Texture,
        ),
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
        asset_type: Some(AssetTypeProjectionSnapshot::from_resource_kind(
            ResourceKind::Material,
        )),
        known_project_asset: true,
    });
    snapshot.selection.used_by.push(AssetReferenceSnapshot {
        uuid: "external-uuid-1".to_string(),
        locator: "file:///vendor/texture.png".to_string(),
        display_name: "External Texture".to_string(),
        kind: Some(ResourceKind::Texture),
        asset_type: Some(AssetTypeProjectionSnapshot::from_resource_kind(
            ResourceKind::Texture,
        )),
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

    let harness = ChildWindowHostHarness::new("zircon_retained_asset_field_real_payload_drop");
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

    let harness = ChildWindowHostHarness::new("zircon_retained_asset_field_default_payload_drop");
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
