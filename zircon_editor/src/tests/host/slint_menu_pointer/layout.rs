use super::support::*;

#[test]
fn shared_menu_pointer_layout_prefers_shared_root_menu_bar_projection_over_stale_legacy_frames() {
    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_root_projection");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(&chrome);
    let layout = build_host_menu_pointer_layout(
        &model.menu_bar,
        &chrome,
        UiSize::new(1280.0, 720.0),
        &["alpha-00".to_string(), "alpha-01".to_string()],
        Some("compact"),
        Some(&BuiltinHostRootShellFrames {
            shell_frame: Some(UiFrame::new(32.0, 18.0, 1440.0, 900.0)),
            menu_bar_frame: Some(UiFrame::new(32.0, 18.0, 1440.0, 40.0)),
            ..Default::default()
        }),
    );

    assert_eq!(layout.shell_frame, UiFrame::new(32.0, 18.0, 1440.0, 900.0));
    assert_eq!(
        layout.button_frames,
        vec![
            UiFrame::new(40.0, 20.0, 40.0, 22.0),
            UiFrame::new(82.0, 20.0, 42.0, 22.0),
            UiFrame::new(126.0, 20.0, 74.0, 22.0),
            UiFrame::new(202.0, 20.0, 42.0, 22.0),
            UiFrame::new(246.0, 20.0, 42.0, 22.0),
            UiFrame::new(290.0, 20.0, 56.0, 22.0),
            UiFrame::new(348.0, 20.0, 40.0, 22.0),
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
    let model = WorkbenchViewModel::build(&chrome);
    let layout = build_host_menu_pointer_layout(
        &model.menu_bar,
        &chrome,
        UiSize::new(1280.0, 720.0),
        &["alpha-00".to_string(), "alpha-01".to_string()],
        Some("compact"),
        Some(&BuiltinHostRootShellFrames {
            shell_frame: Some(UiFrame::new(32.0, 18.0, 1440.0, 900.0)),
            menu_bar_frame: None,
            ..Default::default()
        }),
    );

    assert_eq!(
        layout.button_frames,
        vec![
            UiFrame::new(40.0, 20.0, 40.0, 22.0),
            UiFrame::new(82.0, 20.0, 42.0, 22.0),
            UiFrame::new(126.0, 20.0, 74.0, 22.0),
            UiFrame::new(202.0, 20.0, 42.0, 22.0),
            UiFrame::new(246.0, 20.0, 42.0, 22.0),
            UiFrame::new(290.0, 20.0, 56.0, 22.0),
            UiFrame::new(348.0, 20.0, 40.0, 22.0),
        ],
        "shared shell projection should still own top-level menu button frames when menu bar frame is temporarily unavailable"
    );
}

#[test]
fn shared_menu_pointer_layout_keeps_editor_operation_actions_for_extension_leaves() {
    let operation_path = EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap();
    let menu_bar = MenuBarModel {
        menus: vec![MenuModel {
            label: "Tools".to_string(),
            items: vec![MenuItemModel::branch(
                "Weather",
                vec![MenuItemModel::leaf(
                    "Refresh Clouds",
                    None,
                    EditorUiBinding::new(
                        "WorkbenchMenuBar",
                        operation_path.as_str(),
                        EditorUiEventKind::Click,
                        EditorUiBindingPayload::editor_operation(operation_path.as_str()),
                    ),
                    Some(operation_path.clone()),
                    None,
                    true,
                )],
            )],
        }],
    };
    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_editor_operation_layout");
    let chrome = harness.runtime.chrome_snapshot();
    let layout = build_host_menu_pointer_layout(
        &menu_bar,
        &chrome,
        UiSize::new(1280.0, 720.0),
        &[],
        None,
        None,
    );

    assert_eq!(
        layout.menus[0][0].children[0].action_id.as_deref(),
        Some("Weather.CloudLayer.Refresh"),
        "extension menu leaves carry EditorOperation payloads through the tree layout and must stay clickable in the shared pointer layout"
    );
}

#[test]
fn shared_menu_pointer_layout_extends_menu_button_frames_for_extension_menus() {
    let menu_bar = MenuBarModel {
        menus: (0..9)
            .map(|index| MenuModel {
                label: format!("Plugin{index}"),
                items: Vec::new(),
            })
            .collect(),
    };
    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_extension_menu_slots");
    let layout = build_host_menu_pointer_layout(
        &menu_bar,
        &harness.runtime.chrome_snapshot(),
        UiSize::new(360.0, 240.0),
        &[],
        None,
        None,
    );

    assert_eq!(layout.button_frames.len(), 9);
    assert!(
        layout.button_frames[8].x
            > layout.button_frames[6].x + layout.button_frames[6].width,
        "extension menu slots should continue after the authored TOML menu bar stencil instead of replacing slot 0"
    );
    assert!(
        layout.menu_bar_content_width > layout.shell_frame.width,
        "overwide extension menu bars should expose horizontal scroll content"
    );
}

#[test]
fn shared_menu_pointer_layout_uses_chrome_menu_overflow_preference() {
    let harness = EventRuntimeHarness::new("zircon_slint_menu_pointer_overflow_preference");
    let mut chrome = harness.runtime.chrome_snapshot();
    chrome.menu_overflow_mode = MenuOverflowMode::MultiColumn;
    let model = WorkbenchViewModel::build(&chrome);
    let layout = build_host_menu_pointer_layout(
        &model.menu_bar,
        &chrome,
        UiSize::new(1280.0, 720.0),
        &[],
        None,
        None,
    );

    assert_eq!(
        layout.menu_overflow_mode,
        MenuOverflowMode::MultiColumn,
        "production menu pointer layout must consume the active chrome overflow preference"
    );
}
