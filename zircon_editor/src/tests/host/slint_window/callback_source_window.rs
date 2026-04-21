#[test]
fn child_window_callback_wiring_tracks_source_window_for_pane_interactions() {
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));
    let helpers = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/helpers.rs"
    ));
    let viewport = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/viewport.rs"
    ));
    let hierarchy = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/hierarchy_pointer.rs"
    ));
    let tree = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/asset_tree_pointer.rs"
    ));
    let content = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/asset_content_pointer.rs"
    ));
    let references = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/asset_reference_pointer.rs"
    ));
    let detail = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/detail_scroll_pointer.rs"
    ));
    let inspector = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/inspector.rs"
    ));
    let assets = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/assets.rs"
    ));
    let pane_actions = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/pane_surface_actions.rs"
    ));
    let ui_asset_editor = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/ui_asset_editor.rs"
    ));
    let workbench_pointer = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/workbench_pointer.rs"
    ));

    assert!(
        helpers.contains(
            "fn resolve_callback_source_window_id(ui: &UiHostWindow) -> Option<MainPageId>"
        ),
        "child callback source helper should resolve the originating native window id"
    );
    for needle in [
        "resolve_callback_source_window_id(&source_ui)",
        ".with_callback_source_window(",
        "pane_surface_host.on_ui_asset_collection_event(",
        "pane_surface_host.on_ui_asset_detail_event(",
    ] {
        assert!(
            wiring.contains(needle),
            "callback wiring should track child window source for pane interactions via `{needle}`"
        );
    }

    for (name, source) in [
        ("viewport", viewport),
        ("hierarchy", hierarchy),
        ("asset tree", tree),
        ("asset content", content),
        ("asset references", references),
        ("detail scroll", detail),
        ("inspector controls", inspector),
        ("asset controls", assets),
        ("pane surface actions", pane_actions),
        ("ui asset editor", ui_asset_editor),
    ] {
        assert!(
            source.contains("self.focus_callback_source_window();"),
            "{name} interactions should focus the originating floating window before dispatch"
        );
    }

    for needle in [
        "pub(super) fn dispatch_ui_asset_collection_event(",
        "pub(super) fn dispatch_ui_asset_detail_event(",
        ".select_ui_asset_editor_binding_event_option(",
        ".select_ui_asset_editor_binding_action_kind(",
        ".select_ui_asset_editor_binding_payload(",
        ".upsert_ui_asset_editor_selected_binding_payload(",
        ".delete_ui_asset_editor_selected_binding_payload(",
    ] {
        assert!(
            ui_asset_editor.contains(needle),
            "ui asset editor host dispatch should include `{needle}`"
        );
    }

    for needle in [
        "self.note_focused_floating_window_surface(surface_key);",
        "self.note_focused_floating_window(Some(window_id));",
    ] {
        assert!(
            workbench_pointer.contains(needle),
            "floating header/tab focus should keep host callback-window focus state in sync via `{needle}`"
        );
    }
}
