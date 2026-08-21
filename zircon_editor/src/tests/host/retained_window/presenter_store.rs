use std::cell::Cell;
use std::rc::Rc;

use crate::ui::retained_host::primitives::PhysicalSize;
use crate::ui::retained_host::{
    configure_native_floating_window_presentation, HostPresentationGenerationCursor,
    NativeFloatingWindowTarget, NativeWindowPresenterStore, UiHostContext,
};
use crate::ui::workbench::layout::MainPageId;
use zircon_runtime_interface::ui::event_ui::UiTreeId;

fn native_target(
    window_id: &MainPageId,
    title: &str,
    bounds: [f32; 4],
) -> NativeFloatingWindowTarget {
    NativeFloatingWindowTarget {
        window_id: window_id.clone(),
        title: title.to_string(),
        bounds,
        surface_tree_id: UiTreeId::new(format!("zircon.editor.native_window.{}", window_id.0)),
    }
}

#[test]
fn native_window_presenter_store_creates_updates_and_hides_secondary_windows() {
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
    assert_eq!(
        initial_shell.native_surface_tree_id,
        "zircon.editor.native_window.window:native-preview"
    );
    assert_eq!(initial_shell.native_window_title, "Native Preview");
    let initial_bounds = initial_shell.native_window_bounds;
    assert_eq!(initial_bounds.x, 120.0);
    assert_eq!(initial_bounds.y, 80.0);
    assert_eq!(initial_bounds.width, 640.0);
    assert_eq!(initial_bounds.height, 480.0);
    assert_eq!(
        window
            .get_host_presentation()
            .native_floating_surface_data
            .native_surface_tree_id,
        "zircon.editor.native_window.window:native-preview"
    );
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
    let window_id = MainPageId::new("window:native-preview");
    let mut presenters = NativeWindowPresenterStore::default();
    let target = native_target(&window_id, "Native Preview", [120.0, 80.0, 640.0, 480.0]);
    let callback_hits = Rc::new(Cell::new(0));

    presenters
        .sync_targets(
            &[target],
            |ui, _target| {
                let callback_hits = callback_hits.clone();
                ui.global::<UiHostContext>()
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
        .global::<UiHostContext>()
        .invoke_menu_pointer_clicked(18.0, 24.0);

    assert_eq!(callback_hits.get(), 1);
}

#[test]
fn native_window_presenter_store_skips_an_already_applied_generation() {
    let window_id = MainPageId::new("window:native-preview");
    let mut presenters = NativeWindowPresenterStore::default();
    let initial = native_target(&window_id, "Native Preview", [120.0, 80.0, 640.0, 480.0]);
    let generation = HostPresentationGenerationCursor::new(7, 11, 13, 17, 19, 23);
    let apply_count = Rc::new(Cell::new(0));

    let sync = |presenters: &mut NativeWindowPresenterStore,
                target: NativeFloatingWindowTarget,
                generation: HostPresentationGenerationCursor| {
        let apply_count = Rc::clone(&apply_count);
        presenters
            .sync_targets_with_generation(
                &[target],
                generation,
                |_ui, _target| {},
                move |_ui, _target| apply_count.set(apply_count.get() + 1),
            )
            .expect("generation-aware native window sync should succeed");
    };

    sync(&mut presenters, initial.clone(), generation);
    sync(&mut presenters, initial.clone(), generation);
    assert_eq!(apply_count.get(), 1);

    let next_generation = HostPresentationGenerationCursor::new(8, 11, 13, 17, 19, 23);
    sync(&mut presenters, initial.clone(), next_generation);
    assert_eq!(apply_count.get(), 2);

    let moved = native_target(&window_id, "Native Preview", [160.0, 80.0, 640.0, 480.0]);
    sync(&mut presenters, moved, next_generation);
    assert_eq!(apply_count.get(), 3);

    presenters
        .sync_targets(&[], |_ui, _target| {}, |_ui, _target| {})
        .expect("removing native windows should succeed");
    sync(&mut presenters, initial, next_generation);
    assert_eq!(apply_count.get(), 4);
}
