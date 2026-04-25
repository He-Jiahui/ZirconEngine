use super::super::support::*;
use crate::core::editor_event::{
    ActivityDrawerMode as EventActivityDrawerMode, ActivityDrawerSlot as EventActivityDrawerSlot,
    LayoutCommand as EventLayoutCommand, ViewInstanceId as EventViewInstanceId,
};

#[test]
fn builtin_host_activity_toggle_collapses_active_project_drawer_from_template_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_activity_collapse");
    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let before = harness.runtime.current_layout();
    let drawer = before.drawers.get(&ActivityDrawerSlot::LeftTop).unwrap();
    assert_eq!(drawer.mode, ActivityDrawerMode::Pinned);
    assert_eq!(
        drawer.active_view.as_ref().map(|id| id.0.as_str()),
        Some("editor.project#1")
    );

    let effects = dispatch_builtin_host_drawer_toggle(
        &harness.runtime,
        &bridge,
        "left_top",
        "editor.project#1",
    )
    .expect("builtin activity rail target should resolve through template bridge")
    .unwrap();

    let after = harness.runtime.current_layout();
    assert_eq!(
        after
            .drawers
            .get(&ActivityDrawerSlot::LeftTop)
            .unwrap()
            .mode,
        ActivityDrawerMode::Collapsed
    );
    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(EventLayoutCommand::SetDrawerMode {
            slot: EventActivityDrawerSlot::LeftTop,
            mode: EventActivityDrawerMode::Collapsed,
        })
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn builtin_host_activity_toggle_reopens_collapsed_project_drawer_from_template_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_activity_reopen");
    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let collapse = dispatch_builtin_host_drawer_toggle(
        &harness.runtime,
        &bridge,
        "left_top",
        "editor.project#1",
    )
    .expect("builtin activity rail target should resolve through template bridge")
    .unwrap();
    assert!(collapse.layout_dirty);

    let effects = dispatch_builtin_host_drawer_toggle(
        &harness.runtime,
        &bridge,
        "left_top",
        "editor.project#1",
    )
    .expect("builtin activity rail target should still resolve through template bridge")
    .unwrap();

    let after = harness.runtime.current_layout();
    let drawer = after.drawers.get(&ActivityDrawerSlot::LeftTop).unwrap();
    assert_eq!(drawer.mode, ActivityDrawerMode::Pinned);
    assert_eq!(
        drawer.active_view.as_ref().map(|id| id.0.as_str()),
        Some("editor.project#1")
    );

    let journal = harness.runtime.journal();
    let events: Vec<_> = journal
        .records()
        .iter()
        .rev()
        .take(2)
        .map(|record| record.event.clone())
        .collect();
    assert_eq!(
        events.into_iter().rev().collect::<Vec<_>>(),
        vec![
            EditorEvent::Layout(EventLayoutCommand::ActivateDrawerTab {
                slot: EventActivityDrawerSlot::LeftTop,
                instance_id: EventViewInstanceId::new("editor.project#1"),
            }),
            EditorEvent::Layout(EventLayoutCommand::SetDrawerMode {
                slot: EventActivityDrawerSlot::LeftTop,
                mode: EventActivityDrawerMode::Pinned,
            }),
        ]
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}
