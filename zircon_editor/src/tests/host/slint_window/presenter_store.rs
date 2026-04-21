use std::cell::Cell;
use std::rc::Rc;

use crate::ui::slint_host::{
    configure_native_floating_window_presentation, NativeFloatingWindowTarget,
    NativeWindowPresenterStore, WorkbenchHostContext,
};
use crate::ui::workbench::layout::MainPageId;
use slint::{ComponentHandle, PhysicalSize};

fn native_target(
    window_id: &MainPageId,
    title: &str,
    bounds: [f32; 4],
) -> NativeFloatingWindowTarget {
    NativeFloatingWindowTarget {
        window_id: window_id.clone(),
        title: title.to_string(),
        bounds,
    }
}

#[test]
fn native_window_presenter_store_creates_updates_and_hides_secondary_windows() {
    i_slint_backend_testing::init_no_event_loop();

    let window_id = MainPageId::new("window:native-preview");
    let mut presenters = NativeWindowPresenterStore::default();
    let initial = native_target(&window_id, "Native Preview", [120.0, 80.0, 640.0, 480.0]);

    presenters
        .sync_targets(
            &[initial],
            |_ui, _target| {},
            |ui, target| {
                configure_native_floating_window_presentation(ui, target);
            },
        )
        .expect("initial native window sync should succeed");

    assert_eq!(presenters.window_ids(), vec![window_id.clone()]);
    let window = presenters
        .window(&window_id)
        .expect("native window should exist after first sync");
    let initial_shell = window.get_host_presentation().host_shell;
    assert!(window.window().is_visible());
    assert!(initial_shell.native_floating_window_mode);
    assert_eq!(
        initial_shell.native_floating_window_id,
        "window:native-preview"
    );
    assert_eq!(initial_shell.native_window_title, "Native Preview");
    let initial_bounds = initial_shell.native_window_bounds;
    assert_eq!(initial_bounds.x, 120.0);
    assert_eq!(initial_bounds.y, 80.0);
    assert_eq!(initial_bounds.width, 640.0);
    assert_eq!(initial_bounds.height, 480.0);
    assert_eq!(window.window().size(), PhysicalSize::new(640, 480));

    let updated = native_target(
        &window_id,
        "Native Preview Updated",
        [160.0, 110.0, 720.0, 520.0],
    );
    presenters
        .sync_targets(
            &[updated],
            |_ui, _target| {},
            |ui, target| {
                configure_native_floating_window_presentation(ui, target);
            },
        )
        .expect("updated native window sync should succeed");

    assert_eq!(presenters.window_ids(), vec![window_id.clone()]);
    let updated_shell = window.get_host_presentation().host_shell;
    assert_eq!(updated_shell.native_window_title, "Native Preview Updated");
    let updated_bounds = updated_shell.native_window_bounds;
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
                configure_native_floating_window_presentation(ui, target);
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
    let mut presenters = NativeWindowPresenterStore::default();
    let target = native_target(&window_id, "Native Preview", [120.0, 80.0, 640.0, 480.0]);
    let callback_hits = Rc::new(Cell::new(0));

    presenters
        .sync_targets(
            &[target],
            |ui, _target| {
                let callback_hits = callback_hits.clone();
                ui.global::<WorkbenchHostContext>()
                    .on_menu_pointer_clicked(move |_x, _y| {
                        callback_hits.set(callback_hits.get() + 1);
                    });
            },
            |ui, target| {
                configure_native_floating_window_presentation(ui, target);
            },
        )
        .expect("native window sync should install callback wiring hook");

    let window = presenters
        .window(&window_id)
        .expect("native window should exist after sync");
    window
        .global::<WorkbenchHostContext>()
        .invoke_menu_pointer_clicked(18.0, 24.0);

    assert_eq!(callback_hits.get(), 1);
}
