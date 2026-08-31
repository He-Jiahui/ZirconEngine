use super::super::support::*;
use crate::core::editor_event::{
    LayoutCommand as EventLayoutCommand, ViewInstanceId as EventViewInstanceId,
};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;
use crate::ui::template_runtime::EditorUiCompatibilityHarness;
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost};

#[test]
fn builtin_floating_window_focus_dispatches_focus_view_from_shared_route() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_floating_window_focus");
    let window_id = MainPageId::new("window:scene");

    dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::DetachViewToWindow {
            instance_id: ViewInstanceId::new("editor.scene#1"),
            new_window: window_id.clone(),
        },
    )
    .unwrap();

    let effects = dispatch_builtin_floating_window_focus(&harness.runtime, &window_id)
        .expect("floating window focus should resolve through shared route")
        .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(EventLayoutCommand::FocusView {
            instance_id: EventViewInstanceId::new("editor.scene#1"),
        })
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn builtin_floating_window_focus_matches_legacy_layout_focus_dispatch_event_log() {
    let _guard = env_lock().lock().unwrap();

    let window_id = MainPageId::new("window:scene");

    let legacy_harness = EventRuntimeHarness::new("zircon_retained_floating_window_focus_legacy");
    dispatch_layout_command(
        &legacy_harness.runtime,
        LayoutCommand::DetachViewToWindow {
            instance_id: ViewInstanceId::new("editor.scene#1"),
            new_window: window_id.clone(),
        },
    )
    .unwrap();
    let legacy_baseline = legacy_harness.runtime.journal().records().len();
    let legacy_effects = dispatch_layout_command(
        &legacy_harness.runtime,
        LayoutCommand::FocusView {
            instance_id: ViewInstanceId::new("editor.scene#1"),
        },
    )
    .unwrap();
    let legacy_snapshot = EditorUiCompatibilityHarness::capture_event_journal_delta_snapshot(
        &legacy_harness.runtime.journal(),
        legacy_baseline,
    );

    let builtin_harness = EventRuntimeHarness::new("zircon_retained_floating_window_focus_builtin");
    dispatch_layout_command(
        &builtin_harness.runtime,
        LayoutCommand::DetachViewToWindow {
            instance_id: ViewInstanceId::new("editor.scene#1"),
            new_window: window_id.clone(),
        },
    )
    .unwrap();
    let builtin_baseline = builtin_harness.runtime.journal().records().len();
    let builtin_effects =
        dispatch_builtin_floating_window_focus(&builtin_harness.runtime, &window_id)
            .expect("floating window focus should resolve through shared route")
            .unwrap();
    let builtin_snapshot = EditorUiCompatibilityHarness::capture_event_journal_delta_snapshot(
        &builtin_harness.runtime.journal(),
        builtin_baseline,
    );

    assert_eq!(builtin_effects, legacy_effects);
    assert_eq!(
        builtin_snapshot.event_entries,
        legacy_snapshot.event_entries
    );
}

#[test]
fn builtin_floating_window_focus_for_source_dispatches_when_switching_windows() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_floating_window_focus_source");
    let window_id = MainPageId::new("window:scene");

    dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::DetachViewToWindow {
            instance_id: ViewInstanceId::new("editor.scene#1"),
            new_window: window_id.clone(),
        },
    )
    .unwrap();

    let effects =
        dispatch_builtin_floating_window_focus_for_source(&harness.runtime, Some(&window_id), None)
            .expect("child window source should trigger focus dispatch")
            .unwrap();

    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Layout(EventLayoutCommand::FocusView {
            instance_id: EventViewInstanceId::new("editor.scene#1"),
        })
    );
}

#[test]
fn builtin_floating_window_focus_for_source_skips_redundant_focus_for_same_window() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_floating_window_focus_source_skip");
    let window_id = MainPageId::new("window:scene");

    dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::DetachViewToWindow {
            instance_id: ViewInstanceId::new("editor.scene#1"),
            new_window: window_id.clone(),
        },
    )
    .unwrap();
    let baseline = harness.runtime.journal().records().len();

    assert!(
        dispatch_builtin_floating_window_focus_for_source(
            &harness.runtime,
            Some(&window_id),
            Some(&window_id),
        )
        .is_none(),
        "same floating callback source should not emit another focus dispatch"
    );
    assert_eq!(harness.runtime.journal().records().len(), baseline);
}

#[test]
fn global_focus_changes_when_main_and_floating_tabs_are_already_locally_active() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_global_main_floating_focus");
    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let scene = ViewInstanceId::new("editor.scene#1");
    let game = ViewInstanceId::new("editor.game#1");
    let scene_window = MainPageId::new("window:scene");

    dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::DetachViewToWindow {
            instance_id: scene.clone(),
            new_window: scene_window,
        },
    )
    .unwrap();
    dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::FocusView {
            instance_id: game.clone(),
        },
    )
    .unwrap();

    let floating_effects = dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::FocusView {
            instance_id: scene.clone(),
        },
    )
    .unwrap();
    assert_eq!(manager.current_focused_view(), Some(scene));
    assert!(floating_effects.layout_dirty);

    let main_effects = dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::FocusView {
            instance_id: game.clone(),
        },
    )
    .unwrap();
    assert_eq!(manager.current_focused_view(), Some(game));
    assert!(main_effects.layout_dirty);
}

#[test]
fn global_focus_changes_between_locally_active_floating_tabs() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_global_floating_focus");
    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let scene = ViewInstanceId::new("editor.scene#1");
    let game = ViewInstanceId::new("editor.game#1");
    for (instance_id, window_id) in [
        (scene.clone(), MainPageId::new("window:scene")),
        (game.clone(), MainPageId::new("window:game")),
    ] {
        dispatch_layout_command(
            &harness.runtime,
            LayoutCommand::DetachViewToWindow {
                instance_id,
                new_window: window_id,
            },
        )
        .unwrap();
    }

    dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::FocusView {
            instance_id: scene.clone(),
        },
    )
    .unwrap();
    let effects = dispatch_layout_command(
        &harness.runtime,
        LayoutCommand::FocusView {
            instance_id: game.clone(),
        },
    )
    .unwrap();

    assert_eq!(manager.current_focused_view(), Some(game));
    assert!(effects.layout_dirty);
}

#[test]
fn closing_the_focused_floating_tab_selects_the_remaining_local_tab() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_global_focus_close");
    let manager = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let scene = ViewInstanceId::new("editor.scene#1");
    let window_id = MainPageId::new("window:editor.scene#1");
    manager.detach_view_to_window(&scene).unwrap();
    let instance = manager
        .open_view(
            ViewDescriptorId::new("editor.debug_observatory"),
            Some(ViewHost::FloatingWindow(window_id, Vec::new())),
        )
        .unwrap();
    manager.focus_view(&instance).unwrap();
    assert_eq!(manager.current_focused_view(), Some(instance.clone()));

    assert!(manager.close_view(&instance).unwrap());
    assert_eq!(manager.current_focused_view(), Some(scene));
}
