fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn shared_menu_pointer_layout_sync_replaces_direct_menu_button_frame_getters() {
    let pointer_layout = source("src/ui/retained_host/app/pointer_layout/menu.rs");
    let pointer_builder =
        source("src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs");
    let menu_sync = pointer_layout
        .split("fn sync_menu_pointer_layout")
        .nth(1)
        .and_then(|source| source.split("fn apply_menu_pointer_state_to_ui").next())
        .expect("menu pointer sync function should exist");

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
    assert!(menu_sync.contains("self.template_bridge.outer_shell_frames()"));
    assert!(!menu_sync.contains("self.template_bridge.root_shell_frames()"));
    assert!(!menu_sync.contains("workbench_window_bridge.layout_frames()"));
    assert!(menu_sync.contains("build_host_menu_pointer_layout("));
    assert!(pointer_builder.contains("menu_button_frames_from_chrome_asset"));
}

#[test]
fn shared_menu_pointer_layout_keeps_outer_shell_owner_contract() {
    let pointer_layout = source("src/ui/retained_host/app/pointer_layout/menu.rs");
    let pointer_builder =
        source("src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs");
    let workbench_bridge =
        source("src/ui/retained_host/callback_dispatch/template_bridge/workbench/bridge.rs");
    let workbench_layout_frames =
        source("src/ui/retained_host/callback_dispatch/template_bridge/workbench/layout_frames.rs");
    let menu_sync = pointer_layout
        .split("fn sync_menu_pointer_layout")
        .nth(1)
        .and_then(|source| source.split("fn apply_menu_pointer_state_to_ui").next())
        .expect("menu pointer sync function should exist");

    assert!(workbench_bridge.contains("pub(crate) fn outer_shell_frames(&self)"));
    assert!(
        menu_sync.contains("let outer_shell_frames = self.template_bridge.outer_shell_frames();")
    );
    assert!(pointer_builder.contains("BuiltinHostOuterShellFrames"));
    assert!(pointer_builder.contains("menu_bar_frame"));
    assert!(
        !workbench_layout_frames.contains("menu_bar_frame"),
        "componentized Workbench layout frames must not grow an outer menu-bar owner"
    );
}

#[test]
fn host_menu_chrome_uses_projected_toml_frames_and_rust_owned_data() {
    let host_components =
        source("src/ui/retained_host/host_contract/data/host_components/menus.rs");
    let host_interaction =
        source("src/ui/retained_host/host_contract/data/host_interaction/menu.rs");
    let pointer_builder =
        source("src/ui/retained_host/menu_pointer/build_host_menu_pointer_layout.rs");
    let chrome_projection =
        source("src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs");
    let menu_chrome = source(
        "src/ui/layouts/windows/workbench_host_window/chrome_template_projection/menu_chrome.rs",
    );
    let scene_projection =
        source("src/ui/layouts/windows/workbench_host_window/scene_projection.rs");
    let menu_asset = source("assets/ui/editor/workbench_menu_chrome.zui");
    let popup_asset = source("assets/ui/editor/workbench_menu_popup.zui");

    assert!(host_components.contains("pub menu_frames: ModelRc<HostChromeControlFrameData>"));
    assert!(host_interaction.contains("pub menu_bar_scroll_px: f32"));
    assert!(host_interaction.contains("pub window_menu_scroll_px: f32"));
    assert!(pointer_builder.contains("menu_button_frames_from_chrome_asset"));
    assert!(chrome_projection.contains("mod menu_chrome;"));
    assert!(menu_chrome.contains("control_frames(nodes, MENU_SLOT_PREFIX, count)"));
    assert!(scene_projection
        .contains("menu_control_frames(&template_nodes, menus.row_count().max(MENU_SLOT_COUNT))"));
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
    let menu_chrome = source(
        "src/ui/layouts/windows/workbench_host_window/chrome_template_projection/menu_chrome.rs",
    );

    assert!(chrome_projection.contains("mod menu_chrome;"));

    for required in [
        "if !item.enabled {",
        "label_node.text_tone = \"muted\".into();",
        "shortcut_node.text_tone = \"muted\".into();",
    ] {
        assert!(
            menu_chrome.contains(required),
            "menu popup projection should make disabled item text visually muted `{required}`"
        );
    }
}

#[test]
fn menu_pointer_bridge_uses_route_intent_only() {
    let bridge = source("src/ui/retained_host/menu_pointer/host_menu_pointer_bridge.rs");
    let rebuild =
        source("src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs");
    let dispatch =
        source("src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_dispatch_event.rs");
    let route_payload =
        source("src/ui/retained_host/menu_pointer/host_menu_pointer_route_intent.rs");

    assert!(bridge.contains("route_intents: EditorRouteIntentMap"));
    assert!(rebuild.contains("EditorRouteIntent::Menu"));
    assert!(rebuild.contains("route_intents.bind_node"));
    assert!(dispatch.contains("menu_route_for_pointer_dispatch"));
    assert!(route_payload.contains("HostMenuPointerRouteIntent"));
    for forbidden in [
        "targets:",
        "HostMenuPointerTarget",
        "handled_by",
        "route.target",
    ] {
        assert!(
            !bridge.contains(forbidden)
                && !rebuild.contains(forbidden)
                && !dispatch.contains(forbidden),
            "menu pointer bridge should not keep old hit target marker `{forbidden}`"
        );
    }
}

#[test]
fn menu_pointer_reuses_the_committed_item_tree() {
    let items = source("src/ui/retained_host/menu_pointer/menu_items_for_layout.rs");
    let rebuild =
        source("src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs");

    assert!(items.contains("Cow<'a, [MenuItemSpec]>"));
    for forbidden in ["root_items.clone()", "branch_item.children.clone()"] {
        assert!(
            !rebuild.contains(forbidden),
            "menu surface rebuild must borrow the committed item tree instead of `{forbidden}`"
        );
    }
}

#[test]
fn menu_popup_routes_use_one_surface_per_layer_and_published_item_indices() {
    let rebuild =
        source("src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_rebuild_surface.rs");
    let popup_items =
        source("src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_popup_items.rs");
    let route_projection =
        source("src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_project_route.rs");
    let item_tree = source("src/ui/retained_host/menu_pointer/menu_item_tree.rs");
    let behavior = source("src/tests/host/retained_menu_pointer/pointer_bridge.rs");

    assert_eq!(
        rebuild
            .matches("insert_popup_layer(PopupLayerInsert {")
            .count(),
        2,
        "root and nested popup layers should each insert one surface node"
    );
    assert!(rebuild.contains("visible_items = branch_item.children.as_slice()"));
    assert!(rebuild.contains("HostMenuPointerRouteIntent::PopupSurface(args.menu_index)"));
    assert!(!rebuild.contains("menu_item_route_index("));
    assert!(popup_items
        .contains("self.popup_route_indices = menu_item_route_indices(&self.popup_items)"));
    assert!(item_tree.contains("indices.insert(path.clone(), *current)"));
    assert!(route_projection
        .contains("popup_item_index_at_point(args.grid, args.items.len(), args.point)"));
    assert!(route_projection.contains("args.route_indices.get(args.item_path.as_slice()).copied()"));
    assert!(behavior
        .contains("fn shared_menu_pointer_bridge_keeps_large_popup_item_authority_virtual()"));
    assert!(behavior.contains("logical menu items must not become retained hit-test nodes"));
    assert!(behavior
        .contains("fn shared_menu_pointer_bridge_opens_flipped_nested_popup_for_branch_hover()"));
}

#[test]
fn menu_scroll_does_not_clone_the_owned_route_payload() {
    let scroll =
        source("src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_scroll.rs");

    assert!(!scroll.contains("route.clone()"));
}

#[test]
fn closed_menu_state_does_not_rebuild_again() {
    let popup = source("src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_popup_state.rs");

    assert!(popup.contains("if self.state.open_menu_index.is_none()"));
    assert!(popup.contains("return;"));
}
