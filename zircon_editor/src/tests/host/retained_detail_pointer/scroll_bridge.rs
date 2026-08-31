use crate::ui::retained_host::console_content_extent;
use crate::ui::retained_host::detail_pointer::{
    asset_details_content_extent, asset_details_scroll_layout, console_scroll_layout,
    inspector_content_extent, inspector_scroll_layout, ScrollSurfacePointerBridge,
    ScrollSurfacePointerRoute, ScrollSurfacePointerState,
};
use crate::ui::workbench::snapshot::AssetSelectionSnapshot;
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};

#[test]
fn shared_console_scroll_surface_bridge_uses_shared_scroll_state() {
    let mut bridge = ScrollSurfacePointerBridge::new();
    let layout = console_scroll_layout(
        UiSize::new(320.0, 56.0),
        console_content_extent(
            "compile started\nmesh cache rebuilt\npreview extraction queued\nimport summary refreshed",
        ),
    );
    bridge.sync(layout.clone(), ScrollSurfacePointerState::default());

    let scrolled = bridge.handle_scroll(UiPoint::new(124.0, 42.0), 48.0);
    assert_eq!(scrolled.route, Some(ScrollSurfacePointerRoute::Viewport));
    assert!(scrolled.state.scroll_offset > 0.0);
    assert!(scrolled.changed);

    bridge.sync(layout, scrolled.state);
    let clamped = bridge.handle_scroll(UiPoint::new(124.0, 42.0), 4096.0);
    assert!(clamped.state.scroll_offset >= scrolled.state.scroll_offset);
}

#[test]
fn console_scroll_clamps_to_the_projected_output_viewport_end() {
    let mut bridge = ScrollSurfacePointerBridge::new();
    let layout = console_scroll_layout(
        UiSize::new(284.0, 146.0),
        console_content_extent("0\n1\n2\n3\n4\n5\n6\n7\n8\n9"),
    );
    bridge.sync(layout, ScrollSurfacePointerState::default());

    let clamped = bridge.handle_scroll(UiPoint::new(12.0, 16.0), 4096.0);

    assert_eq!(clamped.state.scroll_offset, 34.0);
}

#[test]
fn shared_scroll_surface_bridge_skips_rebuild_for_unchanged_layout_and_state() {
    let mut bridge = ScrollSurfacePointerBridge::new();
    let layout = console_scroll_layout(
        UiSize::new(320.0, 56.0),
        console_content_extent(
            "compile started\nmesh cache rebuilt\npreview extraction queued\nimport summary refreshed",
        ),
    );
    let state = ScrollSurfacePointerState::default();

    assert!(bridge.sync(layout.clone(), state.clone()));
    assert!(!bridge.sync(layout, state));
}

#[test]
fn shared_asset_details_scroll_surface_accounts_for_diagnostics_panel() {
    let mut selection = AssetSelectionSnapshot {
        uuid: Some("11111111-1111-1111-1111-111111111111".to_string()),
        display_name: "grid.material".to_string(),
        locator: "res://materials/grid.zmaterial".to_string(),
        kind: None,
        asset_type: crate::ui::workbench::snapshot::AssetTypeProjectionSnapshot::default(),
        preview_artifact_path: "E:/Sandbox/.zircon/cache/editor-previews/grid.png".to_string(),
        meta_path: "E:/Sandbox/assets/materials/grid.zmaterial.zmeta".to_string(),
        toolkit_view_id: "editor.material".to_string(),
        toolkit_open_operation: "material.editor.open".to_string(),
        context_commands: Vec::new(),
        package_id: None,
        asset_unit: "single".to_string(),
        included_files: Vec::new(),
        subassets: Vec::new(),
        diagnostics: Vec::new(),
        resource_state: None,
        resource_revision: Some(7),
        references: Vec::new(),
        used_by: Vec::new(),
    };
    let base_extent = asset_details_content_extent(&selection);
    selection.diagnostics = vec!["preview artifact mismatch".to_string()];
    let diagnostics_extent = asset_details_content_extent(&selection);

    assert!(diagnostics_extent > base_extent);

    let mut bridge = ScrollSurfacePointerBridge::new();
    let layout = asset_details_scroll_layout(UiSize::new(320.0, 220.0), &selection);
    bridge.sync(layout, ScrollSurfacePointerState::default());

    let scrolled = bridge.handle_scroll(UiPoint::new(96.0, 148.0), 120.0);
    assert_eq!(scrolled.route, Some(ScrollSurfacePointerRoute::Viewport));
    assert!(scrolled.state.scroll_offset > 0.0);
}

#[test]
fn shared_inspector_scroll_surface_uses_shared_scroll_state() {
    assert!(inspector_content_extent() > 0.0);

    let mut bridge = ScrollSurfacePointerBridge::new();
    let layout = inspector_scroll_layout(UiSize::new(240.0, 96.0));
    bridge.sync(layout.clone(), ScrollSurfacePointerState::default());

    let scrolled = bridge.handle_scroll(UiPoint::new(108.0, 44.0), 120.0);
    assert_eq!(scrolled.route, Some(ScrollSurfacePointerRoute::Viewport));
    assert!(scrolled.state.scroll_offset > 0.0);

    bridge.sync(layout, scrolled.state);
    let clamped = bridge.handle_scroll(UiPoint::new(108.0, 44.0), 4096.0);
    assert!(clamped.state.scroll_offset >= scrolled.state.scroll_offset);
}

#[test]
fn direct_scroll_receipt_rejects_header_and_unchanged_boundary_input() {
    let selection = AssetSelectionSnapshot {
        display_name: "grid.material".to_string(),
        locator: "res://materials/grid.zmaterial".to_string(),
        ..AssetSelectionSnapshot::default()
    };
    let mut bridge = ScrollSurfacePointerBridge::new();
    bridge.sync(
        asset_details_scroll_layout(UiSize::new(320.0, 220.0), &selection),
        ScrollSurfacePointerState::default(),
    );

    let header = bridge.handle_scroll(UiPoint::new(96.0, 20.0), 120.0);
    assert_eq!(header.route, None);
    assert!(!header.changed);
    assert_eq!(header.state.scroll_offset, 0.0);

    let zero = bridge.handle_scroll(UiPoint::new(96.0, 148.0), 0.0);
    assert_eq!(zero.route, Some(ScrollSurfacePointerRoute::Viewport));
    assert!(!zero.changed);

    let tail = bridge.handle_scroll(UiPoint::new(96.0, 148.0), 4096.0);
    assert!(tail.changed);
    let clamped = bridge.handle_scroll(UiPoint::new(96.0, 148.0), 4096.0);
    assert!(!clamped.changed);
    assert_eq!(clamped.state.scroll_offset, tail.state.scroll_offset);
}
