use crate::host::slint_host::detail_pointer::{
    asset_details_content_extent, asset_details_scroll_layout, console_content_extent,
    console_scroll_layout, inspector_content_extent, inspector_scroll_layout,
    ScrollSurfacePointerBridge, ScrollSurfacePointerRoute, ScrollSurfacePointerState,
};
use crate::host::slint_host::scroll_surface_host::ScrollSurfaceHostState;
use crate::workbench::snapshot::AssetSelectionSnapshot;
use zircon_ui::{UiPoint, UiSize};

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

#[test]
fn scroll_surface_host_state_tracks_size_and_shared_scroll_offset() {
    let mut host =
        ScrollSurfaceHostState::new("zircon.editor.inspector.pointer", "editor.inspector");
    host.set_size(UiSize::new(240.0, 96.0));
    host.sync(inspector_scroll_layout(host.size()));

    host.handle_scroll(UiPoint::new(108.0, 44.0), 120.0)
        .expect("host state should route inspector scroll through shared surface");

    assert!(host.scroll_offset() > 0.0);
}

#[test]
fn inspector_surface_controls_use_generic_template_callbacks_instead_of_legacy_business_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let panes = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/panes.slint"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/host/slint_host/app/callback_wiring.rs"
    ));

    for needle in [
        "callback inspector_name_edited(",
        "callback inspector_parent_edited(",
        "callback inspector_x_edited(",
        "callback inspector_y_edited(",
        "callback inspector_z_edited(",
        "callback inspector_apply()",
        "callback delete_selected()",
        "inspector_name_edited(value) =>",
        "inspector_parent_edited(value) =>",
        "inspector_x_edited(value) =>",
        "inspector_y_edited(value) =>",
        "inspector_z_edited(value) =>",
        "inspector_apply() =>",
        "delete_selected() =>",
    ] {
        assert!(
            !workbench.contains(needle),
            "workbench shell still exposes legacy inspector callback `{needle}`"
        );
    }

    for needle in [
        "callback inspector_name_edited(value: string);",
        "callback inspector_parent_edited(value: string);",
        "callback inspector_x_edited(value: string);",
        "callback inspector_y_edited(value: string);",
        "callback inspector_z_edited(value: string);",
        "callback inspector_apply();",
        "callback delete_selected();",
        "edited(value) => { root.inspector_name_edited(value); }",
        "edited(value) => { root.inspector_parent_edited(value); }",
        "edited(value) => { root.inspector_x_edited(value); }",
        "edited(value) => { root.inspector_y_edited(value); }",
        "edited(value) => { root.inspector_z_edited(value); }",
        "clicked => { root.inspector_apply(); }",
        "clicked => { root.delete_selected(); }",
    ] {
        assert!(
            !panes.contains(needle),
            "inspector pane still exposes legacy direct control callback `{needle}`"
        );
    }

    for needle in [
        "ui.on_inspector_name_edited(",
        "ui.on_inspector_parent_edited(",
        "ui.on_inspector_x_edited(",
        "ui.on_inspector_y_edited(",
        "ui.on_inspector_z_edited(",
        "ui.on_inspector_apply(",
        "ui.on_delete_selected(",
    ] {
        assert!(
            !wiring.contains(needle),
            "slint host wiring still registers legacy inspector callback `{needle}`"
        );
    }
}

#[test]
fn shared_detail_scroll_surfaces_do_not_leave_slint_scrollview_as_authority() {
    let panes = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/panes.slint"
    ));
    let assets = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/assets.slint"
    ));

    for needle in [
        "import { LineEdit, ScrollView } from \"std-widgets.slint\";",
        "ScrollView {\n        width: parent.width;\n        height: parent.height;\n        viewport-y: root.scroll_px * 1px;",
    ] {
        assert!(
            !panes.contains(needle),
            "console pane still leaves Slint ScrollView as scroll authority via `{needle}`"
        );
    }

    for needle in [
        "import { LineEdit, ScrollView } from \"std-widgets.slint\";",
        "ScrollView {\n        x: 0px;\n        y: root.header_height + 1px;",
    ] {
        assert!(
            !assets.contains(needle),
            "asset details rail still leaves Slint ScrollView as scroll authority via `{needle}`"
        );
    }
}
