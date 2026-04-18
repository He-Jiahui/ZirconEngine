use std::cell::Cell;
use std::rc::Rc;

use crate::ui::slint_host::floating_window_projection::build_floating_window_projection_bundle;
use crate::ui::slint_host::UiHostWindow;
use crate::{
    compute_workbench_shell_geometry, default_preview_fixture, DocumentNode, FloatingWindowLayout,
    MainPageId, NativeWindowHostState, ShellFrame, ShellSizePx, TabStackLayout, ViewDescriptorId,
    ViewHost, ViewInstance, ViewInstanceId, WorkbenchChromeMetrics, WorkbenchViewModel,
};
use slint::{ComponentHandle, PhysicalSize};

fn floating_preview_fixture(window_id: &MainPageId) -> crate::PreviewFixture {
    let mut fixture = default_preview_fixture();
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#native-window"),
        descriptor_id: ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Native Preview".to_string(),
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![scene_instance.instance_id.clone()],
            active_tab: Some(scene_instance.instance_id.clone()),
        }),
        focused_view: Some(scene_instance.instance_id),
        frame: ShellFrame::default(),
    });
    fixture
}

#[test]
fn workbench_shell_window_can_resize_and_toggle_maximize() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");

    let initial = ui.window().size();
    assert!(initial.width > 0);
    assert!(initial.height > 0);

    ui.window()
        .set_size(PhysicalSize::new(initial.width + 120, initial.height + 80));

    let resized = ui.window().size();
    assert_eq!(resized.width, initial.width + 120);
    assert_eq!(resized.height, initial.height + 80);
    assert_eq!(ui.get_shell_width_px(), resized.width as f32);
    assert_eq!(ui.get_shell_height_px(), resized.height as f32);

    assert!(!ui.window().is_maximized());
    ui.window().set_maximized(true);
    assert!(ui.window().is_maximized());
    ui.window().set_maximized(false);
    assert!(!ui.window().is_maximized());
}

#[test]
fn native_floating_window_targets_fall_back_to_shared_geometry_when_host_bounds_are_empty() {
    let window_id = MainPageId::new("window:native-preview");
    let fixture = floating_preview_fixture(&window_id);
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(1440.0, 900.0),
        &WorkbenchChromeMetrics::default(),
        None,
    );

    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        &[NativeWindowHostState {
            window_id: window_id.clone(),
            handle: None,
            bounds: [0.0, 0.0, 0.0, 0.0],
        }],
    );
    let targets = crate::ui::slint_host::collect_native_floating_window_targets(
        &model,
        &floating_window_projection_bundle,
    );

    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].window_id, window_id);
    assert_eq!(targets[0].title, "Native Preview");
    let expected = geometry.floating_window_frame(&targets[0].window_id);
    assert_eq!(
        targets[0].bounds,
        [expected.x, expected.y, expected.width, expected.height]
    );
}

#[test]
fn native_window_presenter_store_creates_updates_and_hides_secondary_windows() {
    i_slint_backend_testing::init_no_event_loop();

    let window_id = MainPageId::new("window:native-preview");
    let mut presenters = crate::ui::slint_host::NativeWindowPresenterStore::default();
    let initial = crate::ui::slint_host::NativeFloatingWindowTarget {
        window_id: window_id.clone(),
        title: "Native Preview".to_string(),
        bounds: [120.0, 80.0, 640.0, 480.0],
    };

    presenters
        .sync_targets(
            &[initial.clone()],
            |_ui, _target| {},
            |ui, target| {
                crate::ui::slint_host::configure_native_floating_window_presentation(ui, target);
            },
        )
        .expect("initial native window sync should succeed");

    assert_eq!(presenters.window_ids(), vec![window_id.clone()]);
    let window = presenters
        .window(&window_id)
        .expect("native window should exist after first sync");
    assert!(window.window().is_visible());
    assert!(window.get_native_floating_window_mode());
    assert_eq!(
        window.get_native_floating_window_id(),
        "window:native-preview"
    );
    assert_eq!(window.get_native_window_title(), "Native Preview");
    let initial_bounds = window.get_native_window_bounds();
    assert_eq!(initial_bounds.x, 120.0);
    assert_eq!(initial_bounds.y, 80.0);
    assert_eq!(initial_bounds.width, 640.0);
    assert_eq!(initial_bounds.height, 480.0);
    assert_eq!(window.window().size(), PhysicalSize::new(640, 480));

    let updated = crate::ui::slint_host::NativeFloatingWindowTarget {
        window_id: window_id.clone(),
        title: "Native Preview Updated".to_string(),
        bounds: [160.0, 110.0, 720.0, 520.0],
    };
    presenters
        .sync_targets(
            &[updated],
            |_ui, _target| {},
            |ui, target| {
                crate::ui::slint_host::configure_native_floating_window_presentation(ui, target);
            },
        )
        .expect("updated native window sync should succeed");

    assert_eq!(presenters.window_ids(), vec![window_id.clone()]);
    assert_eq!(window.get_native_window_title(), "Native Preview Updated");
    let updated_bounds = window.get_native_window_bounds();
    assert_eq!(updated_bounds.x, 160.0);
    assert_eq!(updated_bounds.y, 110.0);
    assert_eq!(updated_bounds.width, 720.0);
    assert_eq!(updated_bounds.height, 520.0);
    assert_eq!(window.window().size(), PhysicalSize::new(720, 520));

    presenters
        .sync_targets(
            &[],
            |_ui, _target| {},
            |ui, target| {
                crate::ui::slint_host::configure_native_floating_window_presentation(ui, target);
            },
        )
        .expect("removing native windows should succeed");

    assert!(presenters.window_ids().is_empty());
    assert!(!window.window().is_visible());
}

#[test]
fn native_window_presenter_store_runs_child_window_creation_hook_for_callback_wiring() {
    i_slint_backend_testing::init_no_event_loop();

    let window_id = MainPageId::new("window:native-preview");
    let mut presenters = crate::ui::slint_host::NativeWindowPresenterStore::default();
    let target = crate::ui::slint_host::NativeFloatingWindowTarget {
        window_id: window_id.clone(),
        title: "Native Preview".to_string(),
        bounds: [120.0, 80.0, 640.0, 480.0],
    };
    let callback_hits = Rc::new(Cell::new(0));

    presenters
        .sync_targets(
            &[target.clone()],
            |ui, _target| {
                let callback_hits = callback_hits.clone();
                ui.on_menu_pointer_clicked(move |_x, _y| {
                    callback_hits.set(callback_hits.get() + 1);
                });
            },
            |ui, target| {
                crate::ui::slint_host::configure_native_floating_window_presentation(ui, target);
            },
        )
        .expect("native window sync should install callback wiring hook");

    let window = presenters
        .window(&window_id)
        .expect("native window should exist after sync");
    window.invoke_menu_pointer_clicked(18.0, 24.0);

    assert_eq!(callback_hits.get(), 1);
}

#[test]
fn native_floating_window_mode_forwards_tabs_header_and_pane_callbacks_to_root() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let native_start = workbench
        .find("if root.native_floating_window_mode: Rectangle {")
        .expect("native floating window block should exist");
    let native_mode = &workbench[native_start..];

    for needle in [
        "closeable: tab.closeable;",
        "pointer_clicked(x, y) => {",
        "close_pointer_clicked(x, y) => {",
        "root.document_tab_pointer_clicked(",
        "root.document_tab_close_pointer_clicked(",
        "root.floating_window_header_pointer_clicked(",
        "root.native_window_bounds.x + self.x / 1px + self.mouse-x / 1px",
        "root.native_window_bounds.y + self.mouse-y / 1px",
        "surface_control_clicked(control_id, action_id) => { root.pane_surface_control_clicked(control_id, action_id); }",
        "asset_tree_pointer_clicked(surface_mode, x, y, width, height) => { root.asset_tree_pointer_clicked(surface_mode, x, y, width, height); }",
        "hierarchy_pointer_scrolled(x, y, delta, width, height) => { root.hierarchy_pointer_scrolled(x, y, delta, width, height); }",
        "console_pointer_scrolled(x, y, delta, width, height) => { root.console_pointer_scrolled(x, y, delta, width, height); }",
        "inspector_pointer_scrolled(x, y, delta, width, height) => { root.inspector_pointer_scrolled(x, y, delta, width, height); }",
        "viewport_pointer_event(kind, button, x, y, delta) => { root.viewport_pointer_event(kind, button, x, y, delta); }",
        "ui_asset_action(instance_id, action_id) => { root.ui_asset_action(instance_id, action_id); }",
    ] {
        assert!(
            native_mode.contains(needle),
            "native floating window mode is missing shared callback forwarding `{needle}`"
        );
    }

    assert!(
        !native_mode.contains("closeable: false;"),
        "native floating window mode should not hardcode tabs as non-closeable"
    );
}

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
        "ui.on_ui_asset_collection_event(",
        "ui.on_ui_asset_detail_event(",
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

#[test]
fn ui_asset_editor_host_genericizes_collection_event_dispatch() {
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));
    let ui_asset_editor = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/ui_asset_editor.rs"
    ));

    assert!(
        wiring.contains("ui.on_ui_asset_collection_event("),
        "callback wiring should converge root-level UI asset selection callbacks into a generic collection event hook"
    );

    for legacy_wiring in [
        "ui.on_ui_asset_matched_style_rule_selected(",
        "ui.on_ui_asset_palette_selected(",
        "ui.on_ui_asset_palette_target_candidate_selected(",
        "ui.on_ui_asset_hierarchy_selected(",
        "ui.on_ui_asset_hierarchy_activated(",
        "ui.on_ui_asset_preview_selected(",
        "ui.on_ui_asset_preview_activated(",
        "ui.on_ui_asset_source_outline_selected(",
        "ui.on_ui_asset_preview_mock_selected(",
        "ui.on_ui_asset_binding_selected(",
        "ui.on_ui_asset_binding_event_selected(",
        "ui.on_ui_asset_binding_action_kind_selected(",
        "ui.on_ui_asset_binding_payload_selected(",
        "ui.on_ui_asset_slot_semantic_selected(",
        "ui.on_ui_asset_layout_semantic_selected(",
    ] {
        assert!(
            !wiring.contains(legacy_wiring),
            "callback wiring should drop legacy UI asset collection hook `{legacy_wiring}`"
        );
    }

    assert!(
        ui_asset_editor.contains("pub(super) fn dispatch_ui_asset_collection_event("),
        "ui asset editor host dispatch should expose a generic collection event dispatcher"
    );

    for legacy_dispatch in [
        "pub(super) fn dispatch_ui_asset_matched_style_rule_selected(",
        "pub(super) fn dispatch_ui_asset_palette_selected(",
        "pub(super) fn dispatch_ui_asset_palette_target_candidate_selected(",
        "pub(super) fn dispatch_ui_asset_hierarchy_selected(",
        "pub(super) fn dispatch_ui_asset_hierarchy_activated(",
        "pub(super) fn dispatch_ui_asset_preview_selected(",
        "pub(super) fn dispatch_ui_asset_preview_activated(",
        "pub(super) fn dispatch_ui_asset_source_outline_selected(",
        "pub(super) fn dispatch_ui_asset_preview_mock_selected(",
        "pub(super) fn dispatch_ui_asset_binding_selected(",
        "pub(super) fn dispatch_ui_asset_binding_event_selected(",
        "pub(super) fn dispatch_ui_asset_binding_action_kind_selected(",
        "pub(super) fn dispatch_ui_asset_binding_payload_selected(",
        "pub(super) fn dispatch_ui_asset_slot_semantic_selected(",
        "pub(super) fn dispatch_ui_asset_layout_semantic_selected(",
    ] {
        assert!(
            !ui_asset_editor.contains(legacy_dispatch),
            "ui asset editor host dispatch should remove legacy collection handler `{legacy_dispatch}`"
        );
    }

    for manager_call in [
        ".select_ui_asset_editor_matched_style_rule(",
        ".select_ui_asset_editor_palette_index(",
        ".select_ui_asset_editor_palette_target_candidate(",
        ".select_ui_asset_editor_hierarchy_index(",
        ".activate_ui_asset_editor_hierarchy_index(",
        ".select_ui_asset_editor_preview_index(",
        ".activate_ui_asset_editor_preview_index(",
        ".select_ui_asset_editor_source_outline_index(",
        ".select_ui_asset_editor_preview_mock_property(",
        ".select_ui_asset_editor_binding(",
        ".select_ui_asset_editor_binding_event_option(",
        ".select_ui_asset_editor_binding_action_kind(",
        ".select_ui_asset_editor_binding_payload(",
        ".select_ui_asset_editor_slot_semantic(",
        ".select_ui_asset_editor_layout_semantic(",
    ] {
        assert!(
            ui_asset_editor.contains(manager_call),
            "generic collection dispatch should still route through `{manager_call}`"
        );
    }
}

#[test]
fn ui_asset_editor_host_genericizes_detail_event_dispatch() {
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/callback_wiring.rs"
    ));
    let ui_asset_editor = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/ui/slint_host/app/ui_asset_editor.rs"
    ));

    assert!(
        wiring.contains("ui.on_ui_asset_detail_event("),
        "callback wiring should converge UI asset detail callbacks into a generic detail event hook"
    );

    for legacy_wiring in [
        "ui.on_ui_asset_inspector_widget_action(",
        "ui.on_ui_asset_style_rule_action(",
        "ui.on_ui_asset_style_rule_declaration_action(",
        "ui.on_ui_asset_style_token_action(",
        "ui.on_ui_asset_preview_mock_action(",
        "ui.on_ui_asset_binding_payload_action(",
    ] {
        assert!(
            !wiring.contains(legacy_wiring),
            "callback wiring should drop legacy detail hook `{legacy_wiring}`"
        );
    }

    assert!(
        ui_asset_editor.contains("pub(super) fn dispatch_ui_asset_detail_event("),
        "ui asset editor host dispatch should expose a generic detail event dispatcher"
    );

    for legacy_dispatch in [
        "pub(super) fn dispatch_ui_asset_inspector_widget_action(",
        "pub(super) fn dispatch_ui_asset_style_rule_action(",
        "pub(super) fn dispatch_ui_asset_style_rule_declaration_action(",
        "pub(super) fn dispatch_ui_asset_style_token_action(",
        "pub(super) fn dispatch_ui_asset_preview_mock_action(",
        "pub(super) fn dispatch_ui_asset_binding_payload_action(",
    ] {
        assert!(
            !ui_asset_editor.contains(legacy_dispatch),
            "ui asset editor host dispatch should drop legacy detail dispatcher `{legacy_dispatch}`"
        );
    }

    for manager_call in [
        ".set_ui_asset_editor_selected_widget_control_id(",
        ".rename_ui_asset_editor_selected_stylesheet_rule(",
        ".upsert_ui_asset_editor_selected_style_rule_declaration(",
        ".upsert_ui_asset_editor_style_token(",
        ".set_ui_asset_editor_selected_preview_mock_value(",
        ".upsert_ui_asset_editor_selected_binding_payload(",
    ] {
        assert!(
            ui_asset_editor.contains(manager_call),
            "generic detail dispatch should still route through `{manager_call}`"
        );
    }
}
