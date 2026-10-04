use super::support::*;
use crate::ui::retained_host::shell_pointer::HostShellPointerRoute;
use crate::ui::retained_host::tab_drag::HostDragTargetGroup;
use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::document_tabs::{
    document_tab_close_x, DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH, DOCUMENT_TAB_CLOSE_EXTENT,
};
use crate::ui::workbench::view::ViewHost;
use zircon_runtime_interface::ui::layout::UiPoint;

fn native_child_tab_point(child: &UiHostWindow, window_id: &MainPageId) -> UiPoint {
    let presentation = child.get_host_presentation();
    let floating = presentation
        .host_scene_data
        .floating_layer
        .floating_windows
        .iter()
        .find(|window| window.window_id.as_str() == window_id.0.as_str())
        .expect("child presentation should include its native floating window");
    let tab = floating
        .tab_frames
        .get(0)
        .expect("child presentation should include its source tab");
    UiPoint::new(
        floating.frame.x + floating.header_frame.x + tab.frame.x + 4.0,
        floating.frame.y + floating.header_frame.y + tab.frame.y + tab.frame.height * 0.5,
    )
}

fn workbench_pointer_for_route(
    host: &mut RetainedEditorHost,
    expected: HostShellPointerRoute,
) -> UiPoint {
    let width = host.shell_size.width.max(1.0) as u32;
    let height = host.shell_size.height.max(1.0) as u32;
    for y in (0..height).step_by(24) {
        for x in (0..width).step_by(24) {
            let point = UiPoint::new(x as f32 + 1.0, y as f32 + 1.0);
            if host.shell_pointer_bridge.drag_route_at(point) == Some(expected.clone()) {
                return point;
            }
        }
    }
    panic!("Workbench drag surface should contain route {expected:?}");
}

fn drag_child_tab_to_workbench_point(
    child: &UiHostWindow,
    instance_id: &str,
    tab_point: UiPoint,
    workbench_point: UiPoint,
    source_frame: ShellFrame,
) {
    child.dispatch_native_primary_press_for_test(tab_point.x, tab_point.y);
    let child_local_target = UiPoint::new(
        workbench_point.x - source_frame.x,
        workbench_point.y - source_frame.y,
    );
    child.dispatch_native_pointer_move_for_test(child_local_target.x, child_local_target.y);
    assert_eq!(
        host_context(child).get_drag_state().drag_tab_id.as_str(),
        instance_id,
        "the real child tab pointer path should arm its own drag state"
    );
    child.dispatch_native_primary_release_for_test(child_local_target.x, child_local_target.y);
    assert!(
        host_context(child).get_drag_state().drag_tab_id.is_empty(),
        "the child tab capture should clear after the real native Up callback"
    );
}

#[test]
fn child_window_document_tab_pointer_event_dispatches_focus_view_and_tracks_window_focus() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_child_window_tab_dispatch");
    let child = harness.detach_view_to_child_window("editor.assets#1", "window:assets");
    let baseline = harness.journal_len();

    host_context(&child).invoke_document_tab_pointer_clicked(
        "window:assets".into(),
        0,
        8.0,
        120.0,
        40.0,
        16.0,
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(EventLayoutCommand::FocusView {
            instance_id: EventViewInstanceId::new("editor.assets#1"),
        })]
    );

    let host = harness.host.borrow();
    assert_eq!(
        host.last_focused_callback_window,
        Some(MainPageId::new("window:assets"))
    );
    assert_eq!(host.callback_source_window, None);
}

#[test]
fn child_window_document_tab_close_pointer_event_dispatches_close_view_and_keeps_window_focus() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_child_window_tab_close_dispatch");
    let asset_browser = harness.open_view("editor.asset_browser");
    let child = harness.detach_view_to_child_window(asset_browser.0.as_str(), "window:browser");
    let baseline = harness.journal_len();
    let tab_x = 8.0;
    let tab_width = DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH;
    let close_center_x = document_tab_close_x(tab_x, tab_width) + DOCUMENT_TAB_CLOSE_EXTENT * 0.5;

    host_context(&child).invoke_document_tab_close_pointer_clicked(
        "window:browser".into(),
        0,
        tab_x,
        tab_width,
        close_center_x,
        16.0,
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(EventLayoutCommand::CloseView {
            instance_id: EventViewInstanceId::new(asset_browser.0.clone()),
        })]
    );

    let host = harness.host.borrow();
    assert_eq!(host.callback_source_window, None);
}

#[test]
fn child_window_header_pointer_event_dispatches_focus_view_and_tracks_window_focus() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_child_window_header_dispatch");
    let child = harness.detach_view_to_child_window("editor.scene#1", "window:scene");
    let bounds = child
        .get_host_presentation()
        .host_shell
        .native_window_bounds;
    let baseline = harness.journal_len();

    host_context(&child).invoke_floating_window_header_pointer_clicked(
        bounds.x + bounds.width - 40.0,
        bounds.y + 20.0,
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(EventLayoutCommand::FocusView {
            instance_id: EventViewInstanceId::new("editor.scene#1"),
        })]
    );

    let host = harness.host.borrow();
    assert_eq!(
        host.last_focused_callback_window,
        Some(MainPageId::new("window:scene"))
    );
    assert_eq!(host.callback_source_window, None);
}

#[test]
fn native_child_tab_drop_to_root_uses_child_state_and_projected_workbench_coordinates() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_child_tab_drop_to_root");
    let source_id = MainPageId::new("window:drop-source-root");
    let child = harness.detach_view_to_child_window("editor.assets#1", source_id.0.as_str());
    let tab_point = native_child_tab_point(&child, &source_id);
    let (source_frame, root_point) = {
        let mut host = harness.host.borrow_mut();
        let source_frame = host
            .floating_window_projection_bundle
            .outer_frame(&source_id)
            .expect("source child should have a projected Workbench frame");
        assert!(
            source_frame.x.abs() > f32::EPSILON || source_frame.y.abs() > f32::EPSILON,
            "the child route test needs a nonzero projected origin"
        );
        let root_point = workbench_pointer_for_route(
            &mut host,
            HostShellPointerRoute::DragTarget(HostDragTargetGroup::Document),
        );
        (source_frame, root_point)
    };
    let baseline = harness.journal_len();

    drag_child_tab_to_workbench_point(
        &child,
        "editor.assets#1",
        tab_point,
        root_point,
        source_frame,
    );

    let host = harness.host.borrow();
    assert!(
        matches!(
            host.runtime.view_host_for_instance_key("editor.assets#1"),
            Some(ViewHost::Document(_, _))
        ),
        "a drop on the root document surface should attach the child tab to a document workspace"
    );
    assert!(!host.runtime.floating_window_exists(&source_id));
    assert!(harness.delta_events_since(baseline).iter().any(|event| {
        matches!(
            event,
            EditorEvent::Layout(EventLayoutCommand::AttachView {
                instance_id: EventViewInstanceId(id),
                target: crate::core::editor_event::ViewHost::Document(_, _),
                ..
            }) if id == "editor.assets#1"
        )
    }));
    assert!(host_context(&harness.root_ui)
        .get_drag_state()
        .drag_tab_id
        .is_empty());
}

#[test]
fn native_child_tab_drop_to_another_child_uses_target_window_route() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_child_tab_drop_child_to_child");
    let source_id = MainPageId::new("window:drop-source-child");
    let target_id = MainPageId::new("window:drop-target-child");
    let source = harness.detach_view_to_child_window("editor.assets#1", source_id.0.as_str());
    let _target = harness.detach_view_to_child_window("editor.scene#1", target_id.0.as_str());
    let source_tab = native_child_tab_point(&source, &source_id);
    let (source_frame, target_point) = {
        let mut host = harness.host.borrow_mut();
        let source_frame = host
            .floating_window_projection_bundle
            .outer_frame(&source_id)
            .expect("source child should have a projected Workbench frame");
        let target_frame = host
            .floating_window_projection_bundle
            .outer_frame(&target_id)
            .expect("target child should have a projected Workbench frame");
        let target_point = UiPoint::new(
            target_frame.x + target_frame.width * 0.5,
            target_frame.y + target_frame.height * 0.5,
        );
        assert_eq!(
            host.shell_pointer_bridge.drag_route_at(target_point),
            Some(HostShellPointerRoute::FloatingWindow(target_id.clone())),
            "the target point should resolve to the other native floating window"
        );
        (source_frame, target_point)
    };

    drag_child_tab_to_workbench_point(
        &source,
        "editor.assets#1",
        source_tab,
        target_point,
        source_frame,
    );

    let host = harness.host.borrow();
    assert!(matches!(
        host.runtime.view_host_for_instance_key("editor.assets#1"),
        Some(ViewHost::FloatingWindow(window_id, _)) if window_id == target_id
    ));
    assert!(host
        .runtime
        .floating_window_instance_ids(&target_id)
        .expect("target child should remain open")
        .contains(&ViewInstanceId::new("editor.assets#1")));
    assert!(!host.runtime.floating_window_exists(&source_id));
    assert!(host_context(&harness.root_ui)
        .get_drag_state()
        .drag_tab_id
        .is_empty());
}

#[test]
fn native_child_tab_drop_on_empty_workbench_space_detaches_to_a_new_window() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_child_tab_drop_blank");
    let source_id = MainPageId::new("window:drop-source-blank");
    let child = harness.detach_view_to_child_window("editor.assets#1", source_id.0.as_str());
    let tab_point = native_child_tab_point(&child, &source_id);
    let (source_frame, empty_point) = {
        let mut host = harness.host.borrow_mut();
        let source_frame = host
            .floating_window_projection_bundle
            .outer_frame(&source_id)
            .expect("source child should have a projected Workbench frame");
        let empty_point = UiPoint::new(10_000.0, 10_000.0);
        assert_eq!(
            host.shell_pointer_bridge.drag_route_at(empty_point),
            None,
            "empty off-surface space should have no drop target route"
        );
        (source_frame, empty_point)
    };

    drag_child_tab_to_workbench_point(
        &child,
        "editor.assets#1",
        tab_point,
        empty_point,
        source_frame,
    );

    let detached_id = MainPageId::new("window:editor.assets:1");
    let host = harness.host.borrow();
    assert!(matches!(
        host.runtime.view_host_for_instance_key("editor.assets#1"),
        Some(ViewHost::FloatingWindow(window_id, _)) if window_id == detached_id
    ));
    assert!(host.runtime.floating_window_exists(&detached_id));
    assert!(host.native_window_presenters.window(&detached_id).is_some());
    assert!(!host.runtime.floating_window_exists(&source_id));
    assert!(host_context(&harness.root_ui)
        .get_drag_state()
        .drag_tab_id
        .is_empty());
}
