use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use crate::ui::slint_host::callback_dispatch::{
    dispatch_menu_action, dispatch_shared_menu_pointer_click, BuiltinWorkbenchTemplateBridge,
};
use crate::ui::slint_host::menu_pointer::{
    build_workbench_menu_pointer_layout, WorkbenchMenuPointerBridge, WorkbenchMenuPointerLayout,
    WorkbenchMenuPointerRoute, WorkbenchMenuPointerState,
};
use crate::{EditorEvent, LayoutCommand, MenuAction};
use zircon_ui::{UiFrame, UiPoint, UiSize};

#[test]
fn shared_menu_pointer_bridge_opens_and_closes_top_level_menu_from_shared_hit_test() {
    let mut bridge = WorkbenchMenuPointerBridge::new();
    bridge.sync(default_menu_layout(), WorkbenchMenuPointerState::default());

    let opened = bridge.handle_click(UiPoint::new(20.0, 12.0)).unwrap();
    assert_eq!(opened.route, Some(WorkbenchMenuPointerRoute::MenuButton(0)));
    assert_eq!(opened.state.open_menu_index, Some(0));
    assert_eq!(opened.state.hovered_menu_index, Some(0));
    assert_eq!(opened.state.hovered_item_index, None);
    assert_eq!(opened.action_id, None);

    bridge.sync(default_menu_layout(), opened.state);
    let closed = bridge.handle_click(UiPoint::new(20.0, 12.0)).unwrap();
    assert_eq!(closed.route, Some(WorkbenchMenuPointerRoute::MenuButton(0)));
    assert_eq!(closed.state.open_menu_index, None);
    assert_eq!(closed.state.hovered_menu_index, None);
    assert_eq!(closed.state.hovered_item_index, None);
    assert_eq!(closed.action_id, None);
}

#[test]
fn shared_menu_pointer_bridge_resolves_popup_item_and_dismiss_overlay_routes() {
    let mut bridge = WorkbenchMenuPointerBridge::new();
    bridge.sync(
        default_menu_layout(),
        WorkbenchMenuPointerState {
            open_menu_index: Some(0),
            hovered_menu_index: Some(0),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
        },
    );

    let item = bridge.handle_click(UiPoint::new(60.0, 72.0)).unwrap();
    assert_eq!(
        item.route,
        Some(WorkbenchMenuPointerRoute::MenuItem {
            menu_index: 0,
            item_index: 1,
            action_id: "SaveProject".to_string(),
        })
    );
    assert_eq!(item.action_id.as_deref(), Some("SaveProject"));
    assert_eq!(item.state.open_menu_index, None);
    assert_eq!(item.state.hovered_menu_index, None);
    assert_eq!(item.state.hovered_item_index, None);

    bridge.sync(
        default_menu_layout(),
        WorkbenchMenuPointerState {
            open_menu_index: Some(0),
            hovered_menu_index: Some(0),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
        },
    );
    let dismiss = bridge.handle_click(UiPoint::new(420.0, 260.0)).unwrap();
    assert_eq!(
        dismiss.route,
        Some(WorkbenchMenuPointerRoute::DismissOverlay)
    );
    assert_eq!(dismiss.action_id, None);
    assert_eq!(dismiss.state.open_menu_index, None);
}

#[test]
fn shared_menu_pointer_bridge_scrolls_window_popup_using_shared_scroll_state() {
    let mut bridge = WorkbenchMenuPointerBridge::new();
    bridge.sync(
        window_menu_layout(10),
        WorkbenchMenuPointerState {
            open_menu_index: Some(4),
            hovered_menu_index: Some(4),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
        },
    );

    let moved = bridge.handle_move(UiPoint::new(240.0, 110.0)).unwrap();
    assert_eq!(
        moved.route,
        Some(WorkbenchMenuPointerRoute::MenuItem {
            menu_index: 4,
            item_index: 2,
            action_id: "LoadPreset.alpha-00".to_string(),
        })
    );
    assert_eq!(moved.state.hovered_menu_index, Some(4));
    assert_eq!(moved.state.hovered_item_index, Some(2));

    bridge.sync(window_menu_layout(10), moved.state);
    let scrolled = bridge
        .handle_scroll(UiPoint::new(240.0, 110.0), 96.0)
        .unwrap();
    assert_eq!(
        scrolled.route,
        Some(WorkbenchMenuPointerRoute::PopupSurface(4))
    );
    assert!(scrolled.state.popup_scroll_offset > 0.0);
    assert_eq!(scrolled.action_id, None);
    assert_eq!(scrolled.state.open_menu_index, Some(4));
}

#[test]
fn shared_menu_pointer_bridge_dismiss_keeps_window_popup_scroll_state() {
    let mut bridge = WorkbenchMenuPointerBridge::new();
    bridge.sync(
        window_menu_layout(10),
        WorkbenchMenuPointerState {
            open_menu_index: Some(4),
            hovered_menu_index: Some(4),
            hovered_item_index: None,
            popup_scroll_offset: 96.0,
        },
    );

    let dismissed = bridge.handle_click(UiPoint::new(520.0, 260.0)).unwrap();
    assert_eq!(
        dismissed.route,
        Some(WorkbenchMenuPointerRoute::DismissOverlay)
    );
    assert_eq!(dismissed.action_id, None);
    assert_eq!(dismissed.state.open_menu_index, None);
    assert_eq!(dismissed.state.hovered_menu_index, None);
    assert_eq!(dismissed.state.hovered_item_index, None);
    assert_eq!(dismissed.state.popup_scroll_offset, 96.0);
}

#[test]
fn shared_menu_pointer_click_dispatches_reset_layout_through_runtime_dispatcher() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_reset_layout");
    let template_bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let mut pointer_bridge = WorkbenchMenuPointerBridge::new();
    pointer_bridge.sync(default_menu_layout(), WorkbenchMenuPointerState::default());

    let opened = dispatch_shared_menu_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        UiPoint::new(20.0, 12.0),
    )
    .expect("shared pointer route should open the file menu");
    assert_eq!(
        opened.pointer.route,
        Some(WorkbenchMenuPointerRoute::MenuButton(0))
    );
    assert!(opened.effects.is_none());

    let dispatched = dispatch_shared_menu_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        UiPoint::new(60.0, 126.0),
    )
    .expect("shared pointer route should dispatch reset layout");
    assert_eq!(
        dispatched.pointer.route,
        Some(WorkbenchMenuPointerRoute::MenuItem {
            menu_index: 0,
            item_index: 3,
            action_id: "ResetLayout".to_string(),
        })
    );
    let effects = dispatched
        .effects
        .expect("menu item click should dispatch into the runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert!(!effects.render_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)
    );
}

#[test]
fn shared_menu_pointer_click_dispatches_scrolled_window_preset_selection() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_load_preset");
    for index in 0..10 {
        dispatch_menu_action(&harness.runtime, &format!("SavePreset.alpha-{index:02}"))
            .expect("preset save setup should succeed");
    }
    let template_bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0))
        .expect("builtin workbench template bridge should build");
    let mut pointer_bridge = WorkbenchMenuPointerBridge::new();
    let layout = window_menu_layout(10);
    pointer_bridge.sync(
        layout.clone(),
        WorkbenchMenuPointerState {
            open_menu_index: Some(4),
            hovered_menu_index: Some(4),
            hovered_item_index: None,
            popup_scroll_offset: 0.0,
        },
    );

    let scrolled = pointer_bridge
        .handle_scroll(UiPoint::new(240.0, 110.0), 96.0)
        .expect("window popup should accept shared scroll input");
    assert!(scrolled.state.popup_scroll_offset > 0.0);

    pointer_bridge.sync(layout, scrolled.state.clone());
    let preset_index = 5usize;
    let click_y = 32.0 + preset_index as f32 * 30.0 - scrolled.state.popup_scroll_offset + 14.0;
    let dispatched = dispatch_shared_menu_pointer_click(
        &harness.runtime,
        &template_bridge,
        &mut pointer_bridge,
        UiPoint::new(240.0, click_y),
    )
    .expect("shared pointer route should dispatch a scrolled preset selection");
    assert_eq!(
        dispatched.pointer.route,
        Some(WorkbenchMenuPointerRoute::MenuItem {
            menu_index: 4,
            item_index: preset_index,
            action_id: "LoadPreset.alpha-03".to_string(),
        })
    );
    let effects = dispatched
        .effects
        .expect("preset click should dispatch into the runtime");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(LayoutCommand::LoadPreset {
            name: "alpha-03".to_string(),
        })
    );
}

fn default_menu_layout() -> WorkbenchMenuPointerLayout {
    WorkbenchMenuPointerLayout {
        shell_frame: UiFrame::new(0.0, 0.0, 1280.0, 720.0),
        button_frames: [
            UiFrame::new(8.0, 1.0, 40.0, 22.0),
            UiFrame::new(50.0, 1.0, 42.0, 22.0),
            UiFrame::new(94.0, 1.0, 74.0, 22.0),
            UiFrame::new(170.0, 1.0, 42.0, 22.0),
            UiFrame::new(214.0, 1.0, 56.0, 22.0),
            UiFrame::new(272.0, 1.0, 40.0, 22.0),
        ],
        save_project_enabled: true,
        undo_enabled: true,
        redo_enabled: true,
        delete_enabled: true,
        preset_names: vec!["rider".to_string(), "compact".to_string()],
        active_preset_name: "rider".to_string(),
        resolved_preset_name: "rider".to_string(),
        window_popup_height: 132.0,
    }
}

fn window_menu_layout(preset_count: usize) -> WorkbenchMenuPointerLayout {
    let mut layout = default_menu_layout();
    layout.preset_names = (0..preset_count)
        .map(|index| format!("alpha-{index:02}"))
        .collect();
    layout.window_popup_height = 192.0;
    layout
}

#[test]
fn shared_menu_pointer_layout_prefers_shared_root_menu_bar_projection_over_stale_legacy_frames() {
    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_root_projection");
    let chrome = harness.runtime.chrome_snapshot();
    let layout = build_workbench_menu_pointer_layout(
        &chrome,
        UiSize::new(1280.0, 720.0),
        &["alpha-00".to_string(), "alpha-01".to_string()],
        Some("compact"),
        Some(
            &crate::ui::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames {
                shell_frame: Some(UiFrame::new(32.0, 18.0, 1440.0, 900.0)),
                menu_bar_frame: Some(UiFrame::new(32.0, 18.0, 1440.0, 40.0)),
                ..Default::default()
            },
        ),
    );

    assert_eq!(layout.shell_frame, UiFrame::new(32.0, 18.0, 1440.0, 900.0));
    assert_eq!(
        layout.button_frames,
        [
            UiFrame::new(40.0, 19.0, 40.0, 22.0),
            UiFrame::new(82.0, 19.0, 42.0, 22.0),
            UiFrame::new(126.0, 19.0, 74.0, 22.0),
            UiFrame::new(202.0, 19.0, 42.0, 22.0),
            UiFrame::new(246.0, 19.0, 56.0, 22.0),
            UiFrame::new(304.0, 19.0, 40.0, 22.0),
        ],
        "shared root menu bar projection should own top-level menu button frames"
    );
    assert_eq!(layout.active_preset_name, "compact");
    assert_eq!(layout.resolved_preset_name, "compact");
}

#[test]
fn shared_menu_pointer_layout_derives_button_frames_from_shared_shell_when_menu_bar_frame_is_missing(
) {
    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_shell_projection");
    let chrome = harness.runtime.chrome_snapshot();
    let layout = build_workbench_menu_pointer_layout(
        &chrome,
        UiSize::new(1280.0, 720.0),
        &["alpha-00".to_string(), "alpha-01".to_string()],
        Some("compact"),
        Some(
            &crate::ui::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames {
                shell_frame: Some(UiFrame::new(32.0, 18.0, 1440.0, 900.0)),
                menu_bar_frame: None,
                ..Default::default()
            },
        ),
    );

    assert_eq!(
        layout.button_frames,
        [
            UiFrame::new(40.0, 19.0, 40.0, 22.0),
            UiFrame::new(82.0, 19.0, 42.0, 22.0),
            UiFrame::new(126.0, 19.0, 74.0, 22.0),
            UiFrame::new(202.0, 19.0, 42.0, 22.0),
            UiFrame::new(246.0, 19.0, 56.0, 22.0),
            UiFrame::new(304.0, 19.0, 40.0, 22.0),
        ],
        "shared shell projection should still own top-level menu button frames when menu bar frame is temporarily unavailable"
    );
}

#[test]
fn shared_menu_pointer_layout_sync_replaces_direct_slint_menu_button_frame_getters() {
    let pointer_layout = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/pointer_layout.rs"
    ));

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
            "menu pointer sync should not keep direct Slint geometry getter `{getter}`"
        );
    }

    assert!(
        pointer_layout.contains("build_workbench_menu_pointer_layout("),
        "menu pointer sync should delegate top-level button frame authority to a shared layout builder"
    );
}

#[test]
fn shared_menu_popup_presentation_drops_host_menu_button_frame_setters() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let scaffold = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_scaffold.slint"
    ));
    let host_components = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/host_components.slint"
    ));
    let pointer_layout = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/pointer_layout.rs"
    ));

    for legacy_anchor in [
        "x: top_bar.file_menu_button_local_frame.x * 1px;",
        "x: top_bar.edit_menu_button_local_frame.x * 1px;",
        "x: top_bar.selection_menu_button_local_frame.x * 1px;",
        "x: top_bar.view_menu_button_local_frame.x * 1px;",
        "x: top_bar.window_menu_button_local_frame.x * 1px;",
        "x: top_bar.help_menu_button_local_frame.x * 1px;",
    ] {
        assert!(
            !workbench.contains(legacy_anchor),
            "menu popup presentation should not anchor to legacy local frame `{legacy_anchor}`"
        );
    }

    for projected_anchor in [
        "x: root.file_menu_button_frame.x * 1px;",
        "x: root.edit_menu_button_frame.x * 1px;",
        "x: root.selection_menu_button_frame.x * 1px;",
        "x: root.view_menu_button_frame.x * 1px;",
        "x: root.window_menu_button_frame.x * 1px;",
        "x: root.help_menu_button_frame.x * 1px;",
    ] {
        assert!(
            host_components.contains(projected_anchor),
            "host menu chrome is missing shared projected anchor `{projected_anchor}`"
        );
    }

    for removed_setter in [
        "set_file_menu_button_frame(",
        "set_edit_menu_button_frame(",
        "set_selection_menu_button_frame(",
        "set_view_menu_button_frame(",
        "set_window_menu_button_frame(",
        "set_help_menu_button_frame(",
    ] {
        assert!(
            !pointer_layout.contains(removed_setter),
            "menu popup presentation should not keep host menu frame setter `{removed_setter}`"
        );
    }

    for removed_binding in [
        "file_menu_button_frame <=> host.file_menu_button_frame",
        "edit_menu_button_frame <=> host.edit_menu_button_frame",
        "selection_menu_button_frame <=> host.selection_menu_button_frame",
        "view_menu_button_frame <=> host.view_menu_button_frame",
        "window_menu_button_frame <=> host.window_menu_button_frame",
        "help_menu_button_frame <=> host.help_menu_button_frame",
    ] {
        assert!(
            !workbench.contains(removed_binding) && !scaffold.contains(removed_binding),
            "menu popup presentation should not keep root/scaffold frame binding `{removed_binding}`"
        );
    }
}
