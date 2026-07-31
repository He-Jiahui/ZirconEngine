use super::support::*;
use crate::ui::workbench::document_tabs::{
    DOCUMENT_CLOSEABLE_TAB_MIN_WIDTH, DOCUMENT_TAB_CLOSE_EXTENT, document_tab_close_x,
};

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
