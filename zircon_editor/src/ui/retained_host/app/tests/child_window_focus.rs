use super::support::*;

#[test]
fn child_window_viewport_pointer_event_focuses_source_window_before_runtime_dispatch() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_child_window_viewport_dispatch");
    let child = harness.detach_view_to_child_window("editor.scene#1", "window:scene");
    let baseline = harness.journal_len();

    pane_surface_host(&child).invoke_viewport_pointer_event(0, 1, 24.0, 32.0, 0.0, false, false);

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![
            EditorEvent::Layout(EventLayoutCommand::FocusView {
                instance_id: EventViewInstanceId::new("editor.scene#1"),
            }),
            EditorEvent::Viewport(EditorViewportEvent::LeftPressed {
                x: 24.0,
                y: 32.0,
                selection_mutation: crate::scene::selection::SelectionMutation::Replace,
            }),
        ]
    );

    let host = harness.host.borrow();
    assert_eq!(
        host.last_focused_callback_window,
        Some(MainPageId::new("window:scene"))
    );
    assert_eq!(host.callback_source_window, None);
}

#[test]
fn child_window_asset_control_focuses_source_window_before_runtime_dispatch() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_child_window_asset_dispatch");
    let child = harness.detach_view_to_child_window("editor.assets#1", "window:assets");
    let baseline = harness.journal_len();

    pane_surface_host(&child)
        .invoke_asset_control_clicked("activity".into(), "OpenAssetBrowser".into());

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![
            EditorEvent::Layout(EventLayoutCommand::FocusView {
                instance_id: EventViewInstanceId::new("editor.assets#1"),
            }),
            EditorEvent::Asset(EditorAssetEvent::OpenAssetBrowser),
        ]
    );

    let host = harness.host.borrow();
    assert_eq!(
        host.last_focused_callback_window,
        Some(MainPageId::new("window:assets"))
    );
    assert_eq!(host.callback_source_window, None);
}

#[test]
fn child_window_inspector_control_focuses_source_window_before_runtime_dispatch() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_child_window_inspector_dispatch");
    let child = harness.detach_view_to_child_window("editor.inspector#1", "window:inspector");
    let baseline = harness.journal_len();

    pane_surface_host(&child)
        .invoke_inspector_control_changed("NameField".into(), "Draft Cube".into());

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(EventLayoutCommand::FocusView {
            instance_id: EventViewInstanceId::new("editor.inspector#1"),
        })]
    );
    assert_eq!(
        harness
            .host
            .borrow()
            .runtime
            .editor_snapshot()
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.as_str()),
        Some("Draft Cube")
    );

    let host = harness.host.borrow();
    assert_eq!(
        host.last_focused_callback_window,
        Some(MainPageId::new("window:inspector"))
    );
    assert_eq!(host.callback_source_window, None);
}

fn assert_child_window_focus_tracks_asset_scroll(
    event_name: &str,
    invoke: impl FnOnce(&UiHostWindow),
) {
    let harness = ChildWindowHostHarness::new(event_name);
    let child = harness.detach_view_to_child_window("editor.assets#1", "window:assets");
    let baseline = harness.journal_len();

    invoke(&child);

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
fn child_window_asset_tree_scroll_focuses_source_window_before_shared_scroll_dispatch() {
    let _guard = lock_env();

    assert_child_window_focus_tracks_asset_scroll(
        "zircon_retained_child_window_asset_tree_scroll",
        |child| {
            pane_surface_host(child).invoke_asset_tree_pointer_scrolled(
                "activity".into(),
                32.0,
                84.0,
                48.0,
                280.0,
                360.0,
            );
        },
    );
}

#[test]
fn child_window_asset_content_scroll_focuses_source_window_before_shared_scroll_dispatch() {
    let _guard = lock_env();

    assert_child_window_focus_tracks_asset_scroll(
        "zircon_retained_child_window_asset_content_scroll",
        |child| {
            pane_surface_host(child).invoke_asset_content_pointer_scrolled(
                "activity".into(),
                72.0,
                120.0,
                48.0,
                320.0,
                360.0,
            );
        },
    );
}

#[test]
fn child_window_asset_reference_scroll_focuses_source_window_before_shared_scroll_dispatch() {
    let _guard = lock_env();

    assert_child_window_focus_tracks_asset_scroll(
        "zircon_retained_child_window_asset_reference_scroll",
        |child| {
            pane_surface_host(child).invoke_asset_reference_pointer_scrolled(
                "activity".into(),
                "references".into(),
                72.0,
                160.0,
                48.0,
                320.0,
                240.0,
            );
        },
    );
}
