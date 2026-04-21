use crate::ui::slint_host::detail_pointer::{
    asset_details_content_extent, asset_details_scroll_layout, console_content_extent,
    console_scroll_layout, inspector_content_extent, inspector_scroll_layout,
    ScrollSurfacePointerBridge, ScrollSurfacePointerRoute, ScrollSurfacePointerState,
};
use crate::ui::workbench::snapshot::AssetSelectionSnapshot;
use zircon_runtime::ui::layout::{UiPoint, UiSize};

#[test]
fn shared_console_scroll_surface_bridge_uses_shared_scroll_state() {
    let mut bridge =
        ScrollSurfacePointerBridge::new("zircon.editor.console.pointer", "editor.console");
    let layout = console_scroll_layout(
        UiSize::new(320.0, 56.0),
        console_content_extent(
            "compile started\nmesh cache rebuilt\npreview extraction queued\nimport summary refreshed",
            320.0,
            false,
            "",
        ),
    );
    bridge.sync(layout.clone(), ScrollSurfacePointerState::default());

    let scrolled = bridge
        .handle_scroll(UiPoint::new(124.0, 42.0), 48.0)
        .expect("console scroll surface should accept shared scroll input");
    assert_eq!(scrolled.route, Some(ScrollSurfacePointerRoute::Viewport));
    assert!(scrolled.state.scroll_offset > 0.0);

    bridge.sync(layout, scrolled.state.clone());
    let clamped = bridge
        .handle_scroll(UiPoint::new(124.0, 42.0), 4096.0)
        .expect("console scroll surface should clamp overscroll");
    assert!(clamped.state.scroll_offset >= scrolled.state.scroll_offset);
}

#[test]
fn shared_asset_details_scroll_surface_accounts_for_diagnostics_panel() {
    let mut selection = AssetSelectionSnapshot {
        uuid: Some("11111111-1111-1111-1111-111111111111".to_string()),
        display_name: "grid.material".to_string(),
        locator: "res://materials/grid.material.toml".to_string(),
        kind: None,
        preview_artifact_path: "E:/Sandbox/library/editor-previews/grid.png".to_string(),
        meta_path: "E:/Sandbox/assets/materials/grid.material.toml.meta.toml".to_string(),
        adapter_key: "zircon.asset.material".to_string(),
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

    let mut bridge = ScrollSurfacePointerBridge::new(
        "zircon.editor.asset_details.pointer",
        "editor.asset_details",
    );
    let layout = asset_details_scroll_layout(UiSize::new(320.0, 220.0), &selection);
    bridge.sync(layout, ScrollSurfacePointerState::default());

    let scrolled = bridge
        .handle_scroll(UiPoint::new(96.0, 148.0), 120.0)
        .expect("asset details rail should accept shared scroll input");
    assert_eq!(scrolled.route, Some(ScrollSurfacePointerRoute::Viewport));
    assert!(scrolled.state.scroll_offset > 0.0);
}

#[test]
fn shared_inspector_scroll_surface_uses_shared_scroll_state() {
    assert!(inspector_content_extent() > 0.0);

    let mut bridge =
        ScrollSurfacePointerBridge::new("zircon.editor.inspector.pointer", "editor.inspector");
    let layout = inspector_scroll_layout(UiSize::new(240.0, 96.0));
    bridge.sync(layout.clone(), ScrollSurfacePointerState::default());

    let scrolled = bridge
        .handle_scroll(UiPoint::new(108.0, 44.0), 120.0)
        .expect("inspector pane should accept shared scroll input");
    assert_eq!(scrolled.route, Some(ScrollSurfacePointerRoute::Viewport));
    assert!(scrolled.state.scroll_offset > 0.0);

    bridge.sync(layout, scrolled.state.clone());
    let clamped = bridge
        .handle_scroll(UiPoint::new(108.0, 44.0), 4096.0)
        .expect("inspector pane should clamp overscroll");
    assert!(clamped.state.scroll_offset >= scrolled.state.scroll_offset);
}
