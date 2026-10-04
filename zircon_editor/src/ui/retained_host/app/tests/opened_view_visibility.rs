use super::support::*;
use crate::ui::workbench::view::ViewHost;

fn resize_root(harness: &ChildWindowHostHarness, width: u32) {
    harness
        .root_ui
        .window()
        .set_size(PhysicalSize::new(width, 800));
    let mut host = harness.host.borrow_mut();
    host.sync_shell_size();
    host.mark_layout_dirty();
    host.refresh_ui();
    host.recompute_if_dirty();
}

fn instance_id(harness: &ChildWindowHostHarness, descriptor: &str) -> ViewInstanceId {
    harness
        .host
        .borrow()
        .editor_manager
        .current_view_instances()
        .into_iter()
        .find(|instance| instance.descriptor_id.0 == descriptor)
        .expect("real builtin view instance")
        .instance_id
}

#[test]
fn window_open_inspector_at_640_reveals_same_instance_with_live_child_callbacks() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_narrow_open_inspector");
    harness.activate_workbench_page();
    let id = instance_id(&harness, "editor.inspector");
    resize_root(&harness, 640);
    assert_eq!(
        harness
            .host
            .borrow()
            .editor_manager
            .view_host_for_instance_key(&id.0),
        Some(ViewHost::Drawer(ActivityDrawerSlot::RightTop)),
        "resize alone preserves placement"
    );
    let baseline = harness.journal_len();
    harness.dispatch_menu_action("workbench.view.open.editor.inspector");
    assert!(harness.host.borrow().runtime.journal().records()[baseline..].iter().any(|record| {
        record.effects.iter().any(|effect| matches!(effect,
            crate::core::editor_event::EditorEventEffect::OpenedViewVisibilityRequested { instance_id }
                if instance_id.0 == id.0))
    }), "the real command outcome must carry the exact opened instance");
    assert_eq!(instance_id(&harness, "editor.inspector"), id);
    let window_id = MainPageId::new(format!("window:{}", id.0));
    let child = harness
        .host
        .borrow()
        .native_window_presenters
        .window(&window_id)
        .expect("explicit Window/Open Inspector must create its real native owner");
    let content = harness
        .host
        .borrow()
        .floating_window_projection_bundle
        .content_frame(&window_id)
        .expect("native child must have actual projected content");
    assert!(content.width > 0.0 && content.height > 0.0);
    pane_surface_host(&child)
        .invoke_inspector_control_changed("NameField".into(), "Narrow Cube".into());
    let host = harness.host.borrow();
    assert_eq!(
        host.runtime
            .editor_snapshot()
            .inspector
            .as_ref()
            .map(|snapshot| snapshot.name.as_str()),
        Some("Narrow Cube")
    );
    assert_eq!(host.last_focused_callback_window, Some(window_id));
    assert_eq!(host.callback_source_window, None);
}

#[test]
fn window_open_hierarchy_at_640_reuses_authored_outliner_instance_in_native_owner() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_narrow_open_hierarchy");
    harness.activate_workbench_page();
    let id = instance_id(&harness, "editor.hierarchy");
    resize_root(&harness, 640);
    assert_eq!(
        harness
            .host
            .borrow()
            .editor_manager
            .view_host_for_instance_key(&id.0),
        Some(ViewHost::Drawer(ActivityDrawerSlot::LeftTop))
    );
    harness.dispatch_menu_action("workbench.view.open.editor.hierarchy");
    assert_eq!(instance_id(&harness, "editor.hierarchy"), id);
    let window_id = MainPageId::new(format!("window:{}", id.0));
    let child = harness
        .host
        .borrow()
        .native_window_presenters
        .window(&window_id)
        .expect("same Outliner instance must acquire its real native owner");
    let content = harness
        .host
        .borrow()
        .floating_window_projection_bundle
        .content_frame(&window_id)
        .expect("Outliner child content must be projected");
    assert!(content.width > 0.0 && content.height > 0.0);
    pane_surface_host(&child).invoke_hierarchy_pointer_clicked(24.0, 24.0, 0.0, 0.0);
    let host = harness.host.borrow();
    assert_eq!(host.last_focused_callback_window, Some(window_id));
    assert_eq!(
        host.editor_manager
            .current_view_instances()
            .iter()
            .filter(|instance| instance.descriptor_id.0 == "editor.hierarchy")
            .count(),
        1
    );
    assert!(host.hierarchy_pointer_size.width > 0.0 && host.hierarchy_pointer_size.height > 0.0);
}

#[test]
fn window_open_view_preserves_wide_dock_and_existing_custom_floating_owner() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_open_view_preserve_owner");
    harness.activate_workbench_page();
    resize_root(&harness, 1280);
    let id = instance_id(&harness, "editor.inspector");
    harness.dispatch_menu_action("workbench.view.open.editor.inspector");
    assert_eq!(
        harness
            .host
            .borrow()
            .editor_manager
            .view_host_for_instance_key(&id.0),
        Some(ViewHost::Drawer(ActivityDrawerSlot::RightTop))
    );
    let custom = MainPageId::new("window:user-inspector");
    harness.detach_view_to_child_window(&id.0, &custom.0);
    let before = harness
        .host
        .borrow()
        .editor_manager
        .view_host_for_instance_key(&id.0);
    resize_root(&harness, 640);
    harness.dispatch_menu_action("workbench.view.open.editor.inspector");
    assert_eq!(
        harness
            .host
            .borrow()
            .editor_manager
            .view_host_for_instance_key(&id.0),
        before
    );
    assert!(harness
        .host
        .borrow()
        .native_window_presenters
        .window(&custom)
        .is_some());
    assert!(harness
        .host
        .borrow()
        .native_window_presenters
        .window(&MainPageId::new(format!("window:{}", id.0)))
        .is_none());
}

#[test]
fn actual_builtin_window_pane_visibility_matches_wide_and_narrow_root_clips() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_actual_authored_pane_visibility");
    harness.activate_workbench_page();
    for width in [1280, 640] {
        resize_root(&harness, width);
        let presentation = harness.root_ui.get_host_presentation();
        let panes = crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_panes(&presentation);
        for control in [
            "WorkbenchMainBandInspectorPanel",
            "WorkbenchMainBandSceneTreePanel",
        ] {
            let pane = panes
                .iter()
                .find(|pane| pane.root.control_id.as_str() == control);
            if width == 640 {
                assert!(
                    pane.is_none(),
                    "actual collapsed authored pane must have no paint/hit owner: {control}"
                );
            } else {
                let pane = pane.expect("actual builtin wide root must expose both pane owners");
                assert!(pane.frame.width > 0.0 && pane.frame.height > 0.0);
                assert!(
                    pane.frame.x >= 0.0 && pane.frame.x + pane.frame.width <= width as f32 + 1.0
                );
                if pane.root.has_clip_frame {
                    assert!(pane.frame.x >= pane.root.clip_frame.x);
                    assert!(
                        pane.frame.x + pane.frame.width
                            <= pane.root.clip_frame.x + pane.root.clip_frame.width + 1.0
                    );
                }
            }
        }
    }
}

#[test]
fn window_open_inspector_uses_real_140_percent_root_dpi_once() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_narrow_open_inspector_dpi140");
    harness.activate_workbench_page();
    harness.root_ui.window().set_scale_factor(1.4);
    resize_root(&harness, 896);
    {
        let host = harness.host.borrow();
        let resolution =
            crate::ui::workbench::autolayout::ResolutionContext::from_physical_size_with_scale_mode(
                host.shell_size,
                host.shell_scale_factor,
                host.shell_scale_mode,
            );
        assert!((resolution.logical_width() - 640.0).abs() < 1.0);
    }
    let id = instance_id(&harness, "editor.inspector");
    harness.dispatch_menu_action("workbench.view.open.editor.inspector");
    assert!(harness
        .host
        .borrow()
        .native_window_presenters
        .window(&MainPageId::new(format!("window:{}", id.0)))
        .is_some());
}
