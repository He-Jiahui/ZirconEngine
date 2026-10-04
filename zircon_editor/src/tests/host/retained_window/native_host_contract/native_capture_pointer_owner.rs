// Exercise native capture through real root and floating child UiHostWindow instances.
use super::support::*;
use crate::ui::retained_host::primitives::CloseRequestResponse;
use winit::event::{FingerId, PointerKind};
use zircon_runtime_interface::ui::dispatch::UiPointerId;

fn tab_window(child: bool) -> UiHostWindow {
    let ui = UiHostWindow::new().expect("native host should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    if child {
        assert!(ui.set_native_floating_window_presentation(
            "native-child",
            "child-tree",
            "Child",
            &host_frame(0.0, 0.0, 420.0, 260.0),
        ));
    }
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(40.0, 58.0, 340.0, 178.0),
        header_frame: host_frame(0.0, 0.0, 340.0, 31.0),
        tab_frames: model_rc(vec![chrome_tab(
            "document.scene",
            "Scene",
            12.0,
            4.0,
            84.0,
            24.0,
        )]),
        tabs: model_rc(vec![tab_data("document.scene", "Scene")]),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);
    ui
}

fn resize_window(child: bool) -> UiHostWindow {
    let ui = UiHostWindow::new().expect("native host should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    if child {
        assert!(ui.set_native_floating_window_presentation(
            "native-child",
            "child-tree",
            "Child",
            &host_frame(0.0, 0.0, 360.0, 220.0),
        ));
    }
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.resize_layer = HostResizeLayerData {
        left_splitter_frame: host_frame(120.0, 58.0, 8.0, 138.0),
        ..HostResizeLayerData::default()
    };
    ui.set_host_presentation(presentation);
    ui
}

fn drag_events(ui: &UiHostWindow) -> Rc<RefCell<Vec<(i32, f32, f32)>>> {
    let events = Rc::new(RefCell::new(Vec::new()));
    let captured = events.clone();
    ui.global::<UiHostContext>()
        .on_host_drag_pointer_event(move |kind, x, y| captured.borrow_mut().push((kind, x, y)));
    events
}

fn resize_events(ui: &UiHostWindow) -> Rc<RefCell<Vec<(i32, f32, f32)>>> {
    let events = Rc::new(RefCell::new(Vec::new()));
    let captured = events.clone();
    ui.global::<UiHostContext>()
        .on_host_resize_pointer_event(move |kind, x, y| captured.borrow_mut().push((kind, x, y)));
    events
}

#[test]
fn native_child_tab_capture_ignores_foreign_pointer_until_owner_finishes() {
    let root = tab_window(false);
    let child = tab_window(true);
    assert!(
        !root
            .get_host_presentation()
            .host_shell
            .native_floating_window_mode
    );
    assert_eq!(
        child
            .get_host_presentation()
            .host_shell
            .native_floating_window_id
            .as_str(),
        "native-child"
    );
    let root_events = drag_events(&root);
    let child_events = drag_events(&child);
    let owner = UiPointerId::new(7);
    let foreign = UiPointerId::new(9);

    child.dispatch_native_primary_press_with_pointer_for_test(owner, 64.0, 70.0);
    assert_eq!(
        child
            .global::<UiHostContext>()
            .get_drag_state()
            .drag_tab_id
            .as_str(),
        "document.scene"
    );
    child.dispatch_native_primary_press_with_pointer_for_test(foreign, 64.0, 70.0);
    child.dispatch_native_pointer_move_with_pointer_for_test(foreign, 172.0, 132.0);
    child.dispatch_native_primary_release_with_pointer_for_test(foreign, 172.0, 132.0);
    assert!(child_events.borrow().is_empty());
    assert_eq!(
        child
            .global::<UiHostContext>()
            .get_drag_state()
            .capture_pointer_id,
        Some(owner)
    );
    assert_eq!(
        child
            .global::<UiHostContext>()
            .get_drag_state()
            .drag_tab_id
            .as_str(),
        "document.scene",
        "foreign press, move, and release must preserve the owner's pending tab capture"
    );

    child.dispatch_native_pointer_move_with_pointer_for_test(owner, 172.0, 132.0);
    assert_eq!(child_events.borrow().as_slice(), [(0, 172.0, 132.0)]);
    child.dispatch_native_pointer_move_with_pointer_for_test(foreign, 190.0, 145.0);
    child.dispatch_native_primary_release_with_pointer_for_test(foreign, 190.0, 145.0);
    assert_eq!(child_events.borrow().as_slice(), [(0, 172.0, 132.0)]);
    assert!(child.global::<UiHostContext>().get_drag_state().drag_active);

    child.dispatch_native_primary_release_with_pointer_for_test(owner, 172.0, 132.0);
    assert_eq!(
        child_events.borrow().as_slice(),
        [(0, 172.0, 132.0), (2, 172.0, 132.0)]
    );
    assert!(child
        .global::<UiHostContext>()
        .get_drag_state()
        .drag_tab_id
        .is_empty());

    child.dispatch_native_primary_press_with_pointer_for_test(foreign, 64.0, 70.0);
    child.dispatch_native_pointer_move_with_pointer_for_test(foreign, 180.0, 140.0);
    child.dispatch_native_primary_release_with_pointer_for_test(foreign, 180.0, 140.0);
    assert_eq!(
        child_events.borrow().as_slice(),
        [
            (0, 172.0, 132.0),
            (2, 172.0, 132.0),
            (0, 180.0, 140.0),
            (2, 180.0, 140.0),
        ]
    );
    root.dispatch_native_primary_press_with_pointer_for_test(foreign, 64.0, 70.0);
    root.dispatch_native_pointer_move_with_pointer_for_test(foreign, 180.0, 140.0);
    root.dispatch_native_primary_release_with_pointer_for_test(foreign, 180.0, 140.0);
    assert_eq!(
        root_events.borrow().as_slice(),
        [(0, 180.0, 140.0), (2, 180.0, 140.0)],
        "a root window has independent native capture state"
    );
}

#[test]
fn native_root_resize_capture_ignores_foreign_and_ineligible_moves() {
    let root = resize_window(false);
    let child = resize_window(true);
    assert!(
        !root
            .get_host_presentation()
            .host_shell
            .native_floating_window_mode
    );
    assert_eq!(
        child
            .get_host_presentation()
            .host_shell
            .native_floating_window_id
            .as_str(),
        "native-child"
    );
    let root_events = resize_events(&root);
    let child_events = resize_events(&child);
    let owner = UiPointerId::new(7);
    let foreign = UiPointerId::new(9);

    root.dispatch_native_primary_press_with_pointer_for_test(owner, 124.0, 80.0);
    root.dispatch_native_primary_press_with_pointer_for_test(foreign, 124.0, 80.0);
    root.dispatch_native_pointer_move_with_pointer_for_test(foreign, 170.0, 81.0);
    root.dispatch_native_touch_like_pointer_move_for_test(foreign, 171.0, 82.0);
    root.dispatch_native_untranslated_pointer_move_for_test(172.0, 83.0);
    root.dispatch_native_primary_release_with_pointer_for_test(foreign, 172.0, 83.0);
    assert_eq!(root_events.borrow().as_slice(), [(0, 124.0, 80.0)]);
    assert!(
        root.global::<UiHostContext>()
            .get_resize_state()
            .resize_active
    );
    assert_eq!(
        root.global::<UiHostContext>()
            .get_resize_state()
            .capture_pointer_id,
        Some(owner)
    );
    assert_eq!(
        root.global::<UiHostContext>()
            .get_resize_state()
            .resize_pointer_x,
        124.0,
        "foreign and ineligible moves cannot change the owner's last resize point"
    );

    root.dispatch_native_pointer_move_with_pointer_for_test(owner, 180.0, 82.0);
    root.dispatch_native_primary_release_with_pointer_for_test(owner, 180.0, 82.0);
    assert_eq!(
        root_events.borrow().as_slice(),
        [(0, 124.0, 80.0), (1, 180.0, 82.0), (2, 180.0, 82.0)]
    );
    assert!(
        !root
            .global::<UiHostContext>()
            .get_resize_state()
            .resize_active
    );

    root.dispatch_native_primary_press_with_pointer_for_test(foreign, 124.0, 80.0);
    root.dispatch_native_pointer_move_with_pointer_for_test(foreign, 190.0, 84.0);
    root.dispatch_native_primary_release_with_pointer_for_test(foreign, 190.0, 84.0);
    assert_eq!(
        root_events.borrow().as_slice(),
        [
            (0, 124.0, 80.0),
            (1, 180.0, 82.0),
            (2, 180.0, 82.0),
            (0, 124.0, 80.0),
            (1, 190.0, 84.0),
            (2, 190.0, 84.0),
        ]
    );
    child.dispatch_native_primary_press_with_pointer_for_test(owner, 124.0, 80.0);
    child.dispatch_native_pointer_move_with_pointer_for_test(owner, 180.0, 82.0);
    child.dispatch_native_primary_release_with_pointer_for_test(owner, 180.0, 82.0);
    assert_eq!(
        child_events.borrow().as_slice(),
        [(0, 124.0, 80.0), (1, 180.0, 82.0), (2, 180.0, 82.0)],
        "a floating child window has independent native capture state"
    );
}

#[test]
fn foreign_press_cannot_switch_an_active_native_capture_between_tab_and_resize() {
    let ui = tab_window(false);
    let mut presentation = ui.get_host_presentation();
    presentation.host_scene_data.resize_layer = HostResizeLayerData {
        left_splitter_frame: host_frame(120.0, 100.0, 8.0, 96.0),
        ..HostResizeLayerData::default()
    };
    ui.set_host_presentation(presentation);
    let owner = UiPointerId::new(7);
    let foreign = UiPointerId::new(9);

    ui.dispatch_native_primary_press_with_pointer_for_test(owner, 64.0, 70.0);
    ui.dispatch_native_primary_press_with_pointer_for_test(foreign, 124.0, 130.0);
    assert_eq!(
        ui.global::<UiHostContext>()
            .get_drag_state()
            .capture_pointer_id,
        Some(owner)
    );
    assert!(
        !ui.global::<UiHostContext>()
            .get_resize_state()
            .resize_active
    );
    ui.dispatch_native_primary_release_with_pointer_for_test(owner, 64.0, 70.0);

    ui.dispatch_native_primary_press_with_pointer_for_test(foreign, 124.0, 130.0);
    ui.dispatch_native_primary_press_with_pointer_for_test(owner, 64.0, 70.0);
    assert_eq!(
        ui.global::<UiHostContext>()
            .get_resize_state()
            .capture_pointer_id,
        Some(foreign)
    );
    assert!(ui
        .global::<UiHostContext>()
        .get_drag_state()
        .drag_tab_id
        .is_empty());
    ui.dispatch_native_primary_release_with_pointer_for_test(foreign, 124.0, 130.0);
    assert!(
        !ui.global::<UiHostContext>()
            .get_resize_state()
            .resize_active
    );
}

#[test]
fn native_child_focus_loss_cancels_tab_without_drop_and_releases_capture() {
    let root = tab_window(false);
    let child = tab_window(true);
    let root_events = drag_events(&root);
    let child_events = drag_events(&child);
    let owner = UiPointerId::new(7);
    let next = UiPointerId::new(9);

    root.dispatch_native_primary_press_with_pointer_for_test(owner, 64.0, 70.0);
    child.dispatch_native_primary_press_with_pointer_for_test(owner, 64.0, 70.0);
    child.dispatch_native_pointer_move_with_pointer_for_test(owner, 172.0, 132.0);
    child.dispatch_native_focus_lost_for_test();

    assert_eq!(
        child_events.borrow().as_slice(),
        [(0, 172.0, 132.0), (3, 0.0, 0.0)]
    );
    assert!(child
        .global::<UiHostContext>()
        .get_drag_state()
        .drag_tab_id
        .is_empty());
    assert_eq!(
        root.global::<UiHostContext>()
            .get_drag_state()
            .capture_pointer_id,
        Some(owner)
    );
    assert!(root_events.borrow().is_empty());

    child.dispatch_native_primary_release_with_pointer_for_test(owner, 172.0, 132.0);
    assert_eq!(
        child_events.borrow().len(),
        2,
        "late owner Up cannot commit a canceled drop"
    );
    child.dispatch_native_primary_press_with_pointer_for_test(next, 64.0, 70.0);
    child.dispatch_native_pointer_move_with_pointer_for_test(next, 180.0, 140.0);
    child.dispatch_native_primary_release_with_pointer_for_test(next, 180.0, 140.0);
    assert_eq!(child_events.borrow().last(), Some(&(2, 180.0, 140.0)));
    root.dispatch_native_primary_release_with_pointer_for_test(owner, 64.0, 70.0);
}

#[test]
fn native_root_focus_loss_cancels_resize_without_commit_and_allows_new_owner() {
    let root = resize_window(false);
    let child = resize_window(true);
    let root_events = resize_events(&root);
    let child_events = resize_events(&child);
    let owner = UiPointerId::new(7);
    let next = UiPointerId::new(9);

    root.dispatch_native_primary_press_with_pointer_for_test(owner, 124.0, 80.0);
    root.dispatch_native_pointer_move_with_pointer_for_test(owner, 180.0, 82.0);
    child.dispatch_native_primary_press_with_pointer_for_test(owner, 124.0, 80.0);
    root.dispatch_native_focus_lost_for_test();

    assert_eq!(
        root_events.borrow().as_slice(),
        [(0, 124.0, 80.0), (1, 180.0, 82.0), (3, 0.0, 0.0)]
    );
    assert!(
        !root
            .global::<UiHostContext>()
            .get_resize_state()
            .resize_active
    );
    assert_eq!(
        child
            .global::<UiHostContext>()
            .get_resize_state()
            .capture_pointer_id,
        Some(owner)
    );
    assert_eq!(child_events.borrow().as_slice(), [(0, 124.0, 80.0)]);

    root.dispatch_native_primary_release_with_pointer_for_test(owner, 180.0, 82.0);
    assert_eq!(
        root_events.borrow().len(),
        3,
        "late Up cannot commit canceled resize"
    );
    root.dispatch_native_primary_press_with_pointer_for_test(next, 124.0, 80.0);
    root.dispatch_native_primary_release_with_pointer_for_test(next, 124.0, 80.0);
    assert_eq!(root_events.borrow().last(), Some(&(2, 124.0, 80.0)));
    child.dispatch_native_primary_release_with_pointer_for_test(owner, 124.0, 80.0);
}

#[test]
fn native_pointer_leave_keeps_owner_capture_for_cross_window_release() {
    let child = tab_window(true);
    let child_events = drag_events(&child);
    let owner = UiPointerId::new(7);
    child.dispatch_native_primary_press_with_pointer_for_test(owner, 64.0, 70.0);
    child.dispatch_native_pointer_move_with_pointer_for_test(owner, 172.0, 132.0);

    child.dispatch_native_pointer_leave_for_test(PointerKind::Mouse, 430.0, 270.0);
    assert_eq!(
        child
            .global::<UiHostContext>()
            .get_drag_state()
            .capture_pointer_id,
        Some(owner)
    );
    assert_eq!(child_events.borrow().as_slice(), [(0, 172.0, 132.0)]);

    child.dispatch_native_primary_release_with_pointer_for_test(owner, 430.0, 270.0);
    assert_eq!(
        child_events.borrow().as_slice(),
        [(0, 172.0, 132.0), (2, 430.0, 270.0)]
    );
    assert!(child
        .global::<UiHostContext>()
        .get_drag_state()
        .drag_tab_id
        .is_empty());
}

#[test]
fn foreign_touch_pointer_leave_does_not_cancel_native_mouse_resize() {
    let root = resize_window(false);
    let events = resize_events(&root);
    let owner = UiPointerId::new(7);
    root.dispatch_native_primary_press_with_pointer_for_test(owner, 124.0, 80.0);
    root.dispatch_native_pointer_leave_for_test(
        PointerKind::Touch(FingerId::from_raw(99)),
        130.0,
        84.0,
    );
    assert_eq!(
        root.global::<UiHostContext>()
            .get_resize_state()
            .capture_pointer_id,
        Some(owner)
    );
    assert_eq!(events.borrow().as_slice(), [(0, 124.0, 80.0)]);

    root.dispatch_native_primary_release_with_pointer_for_test(owner, 124.0, 80.0);
    assert_eq!(
        events.borrow().as_slice(),
        [(0, 124.0, 80.0), (2, 124.0, 80.0)]
    );
}

#[test]
fn child_hide_close_cancels_only_its_tab_capture_and_late_up_cannot_drop() {
    let root = tab_window(false);
    let child = tab_window(true);
    root.show().expect("root host should show");
    child.show().expect("child host should show");
    child
        .window()
        .on_close_requested(|| CloseRequestResponse::HideWindow);
    let root_events = drag_events(&root);
    let child_events = drag_events(&child);
    let owner = UiPointerId::new(7);
    let next = UiPointerId::new(9);

    root.dispatch_native_primary_press_with_pointer_for_test(owner, 64.0, 70.0);
    child.dispatch_native_primary_press_with_pointer_for_test(owner, 64.0, 70.0);
    child.dispatch_native_pointer_move_with_pointer_for_test(owner, 172.0, 132.0);
    assert_eq!(
        child.dispatch_native_close_request_for_test(),
        CloseRequestResponse::HideWindow
    );

    assert!(!child.window().is_visible());
    assert_eq!(
        child_events.borrow().as_slice(),
        [(0, 172.0, 132.0), (3, 0.0, 0.0)]
    );
    assert!(child
        .global::<UiHostContext>()
        .get_drag_state()
        .drag_tab_id
        .is_empty());
    assert_eq!(
        root.global::<UiHostContext>()
            .get_drag_state()
            .capture_pointer_id,
        Some(owner)
    );
    assert!(root_events.borrow().is_empty());

    child.dispatch_native_primary_release_with_pointer_for_test(owner, 172.0, 132.0);
    assert_eq!(child_events.borrow().len(), 2, "late Up cannot drop");
    child
        .show()
        .expect("child host should reopen for the next gesture");
    child.dispatch_native_primary_press_with_pointer_for_test(next, 64.0, 70.0);
    child.dispatch_native_pointer_move_with_pointer_for_test(next, 180.0, 140.0);
    child.dispatch_native_primary_release_with_pointer_for_test(next, 180.0, 140.0);
    assert_eq!(child_events.borrow().last(), Some(&(2, 180.0, 140.0)));
    root.dispatch_native_primary_release_with_pointer_for_test(owner, 64.0, 70.0);
}

#[test]
fn keep_shown_close_preserves_resize_capture_until_owner_release() {
    let root = resize_window(false);
    root.show().expect("root host should show");
    root.window()
        .on_close_requested(|| CloseRequestResponse::KeepWindowShown);
    let events = resize_events(&root);
    let owner = UiPointerId::new(7);

    root.dispatch_native_primary_press_with_pointer_for_test(owner, 124.0, 80.0);
    root.dispatch_native_pointer_move_with_pointer_for_test(owner, 180.0, 82.0);
    assert_eq!(
        root.dispatch_native_close_request_for_test(),
        CloseRequestResponse::KeepWindowShown
    );

    assert!(root.window().is_visible());
    assert_eq!(
        root.global::<UiHostContext>()
            .get_resize_state()
            .capture_pointer_id,
        Some(owner)
    );
    assert_eq!(
        events.borrow().as_slice(),
        [(0, 124.0, 80.0), (1, 180.0, 82.0)]
    );
    root.dispatch_native_primary_release_with_pointer_for_test(owner, 180.0, 82.0);
    assert_eq!(events.borrow().last(), Some(&(2, 180.0, 82.0)));
}

#[test]
fn programmatic_hide_retires_only_local_tab_capture_without_callback() {
    let root = tab_window(false);
    let child = tab_window(true);
    root.show().expect("root host should show");
    child.show().expect("child host should show");
    let root_events = drag_events(&root);
    let child_events = drag_events(&child);
    let owner = UiPointerId::new(7);
    let next = UiPointerId::new(9);

    root.dispatch_native_primary_press_with_pointer_for_test(owner, 64.0, 70.0);
    child.dispatch_native_primary_press_with_pointer_for_test(owner, 64.0, 70.0);
    child.dispatch_native_pointer_move_with_pointer_for_test(owner, 172.0, 132.0);
    assert_eq!(child_events.borrow().as_slice(), [(0, 172.0, 132.0)]);

    child.hide().expect("programmatic hide should succeed");
    child
        .hide()
        .expect("repeated programmatic hide should remain harmless");

    assert!(!child.window().is_visible());
    assert!(child
        .global::<UiHostContext>()
        .get_drag_state()
        .drag_tab_id
        .is_empty());
    assert_eq!(
        child_events.borrow().as_slice(),
        [(0, 172.0, 132.0)],
        "programmatic hide retires local state without invoking a Cancel callback"
    );
    assert_eq!(
        root.global::<UiHostContext>()
            .get_drag_state()
            .capture_pointer_id,
        Some(owner),
        "hiding a child must preserve another UiHostWindow's capture"
    );

    child.dispatch_native_primary_release_with_pointer_for_test(owner, 172.0, 132.0);
    assert_eq!(child_events.borrow().as_slice(), [(0, 172.0, 132.0)]);
    child
        .show()
        .expect("hidden child should allow a new gesture");
    child.dispatch_native_primary_press_with_pointer_for_test(next, 64.0, 70.0);
    child.dispatch_native_pointer_move_with_pointer_for_test(next, 180.0, 140.0);
    child.dispatch_native_primary_release_with_pointer_for_test(next, 180.0, 140.0);
    assert_eq!(
        child_events.borrow().as_slice(),
        [(0, 172.0, 132.0), (0, 180.0, 140.0), (2, 180.0, 140.0)]
    );
    root.dispatch_native_primary_release_with_pointer_for_test(owner, 64.0, 70.0);
    assert!(root_events.borrow().is_empty());
}
