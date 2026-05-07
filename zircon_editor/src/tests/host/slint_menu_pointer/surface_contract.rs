fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_menu_pointer_layout_sync_replaces_direct_menu_button_frame_getters() {
    let pointer_layout = source("src/ui/slint_host/app/pointer_layout.rs");
    let pointer_builder =
        source("src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs");

    for getter in [
        "get_file_menu_button_frame()",
        "get_edit_menu_button_frame()",
        "get_selection_menu_button_frame()",
        "get_view_menu_button_frame()",
        "get_window_menu_button_frame()",
        "get_help_menu_button_frame()",
    ] {
        assert!(
            !pointer_layout.contains(getter),
            "menu pointer sync should not keep direct geometry getter `{getter}`"
        );
    }
    assert!(pointer_layout.contains("build_host_menu_pointer_layout("));
    assert!(pointer_builder.contains("menu_button_frames_from_chrome_asset"));
}

#[test]
fn host_menu_chrome_uses_projected_toml_frames_and_rust_owned_data() {
    let host_components = source("src/ui/slint_host/host_contract/data/host_components.rs");
    let host_interaction = source("src/ui/slint_host/host_contract/data/host_interaction.rs");
    let pointer_builder =
        source("src/ui/slint_host/menu_pointer/build_host_menu_pointer_layout.rs");
    let chrome_projection =
        source("src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs");
    let scene_projection =
        source("src/ui/layouts/windows/workbench_host_window/scene_projection.rs");
    let menu_asset = source("assets/ui/editor/workbench_menu_chrome.ui.toml");
    let popup_asset = source("assets/ui/editor/workbench_menu_popup.ui.toml");

    assert!(host_components.contains("pub menu_frames: ModelRc<HostChromeControlFrameData>"));
    assert!(host_interaction.contains("pub menu_bar_scroll_px: f32"));
    assert!(host_interaction.contains("pub window_menu_scroll_px: f32"));
    for required in [
        "menu_button_frames_from_chrome_asset",
        "SlotFilter::new(MENU_SLOT_PREFIX, MENU_SLOT_COUNT)",
        "menu_control_frames(&template_nodes, menus.row_count().max(MENU_SLOT_COUNT))",
    ] {
        assert!(
            pointer_builder.contains(required)
                || chrome_projection.contains(required)
                || scene_projection.contains(required),
            "menu projection missing `{required}`"
        );
    }
    for required in [
        "WorkbenchMenuBarRoot",
        "MenuSlot0",
        "MenuSlot5",
        "MenuSlot6",
    ] {
        assert!(
            menu_asset.contains(required),
            "menu chrome asset missing `{required}`"
        );
    }
    for required in [
        "WorkbenchMenuPopupRoot",
        "WorkbenchMenuPopupPanel",
        "MenuPopupItemLabel0",
    ] {
        assert!(
            popup_asset.contains(required),
            "menu popup asset missing `{required}`"
        );
    }
}

#[test]
fn menu_popup_projection_mutes_disabled_item_labels() {
    let chrome_projection =
        source("src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs");

    for required in [
        "if !item.enabled {",
        "label_node.text_tone = \"muted\".into();",
        "shortcut_node.text_tone = \"muted\".into();",
    ] {
        assert!(
            chrome_projection.contains(required),
            "menu popup projection should make disabled item text visually muted `{required}`"
        );
    }
}
