use super::super::support::*;
use crate::core::editor_event::{
    LayoutCommand as EventLayoutCommand, MainPageId as EventMainPageId,
};
use crate::ui::template_runtime::EditorUiCompatibilityHarness;

#[test]
fn builtin_host_page_activation_dispatches_activate_main_page_from_template_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_host_page");
    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let workbench_shell = bridge
        .host_projection()
        .node_by_control_id("UiHostWindowRoot")
        .expect("workbench shell control should exist in builtin template projection");
    assert!(workbench_shell.routes.iter().any(|route| {
        route.event_kind == UiEventKind::Change
            && route.binding_id == "UiHostWindow/ActivateMainPage"
    }));

    let effects = dispatch_builtin_host_page_activation(&harness.runtime, &bridge, "workbench")
        .expect("builtin host page activation should resolve through template bridge")
        .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(EventLayoutCommand::ActivateMainPage {
            page_id: EventMainPageId::new("workbench"),
        })
    );
    assert!(!effects.render_dirty);
}

#[test]
fn builtin_host_page_activation_matches_legacy_layout_command_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let legacy_harness = EventRuntimeHarness::new("zircon_slint_parity_host_page_legacy");
    let legacy_effects = dispatch_layout_command(
        &legacy_harness.runtime,
        LayoutCommand::ActivateMainPage {
            page_id: MainPageId::new("workbench"),
        },
    )
    .unwrap();
    let legacy_record = legacy_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    let builtin_harness = EventRuntimeHarness::new("zircon_slint_parity_host_page_builtin");
    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let builtin_effects =
        dispatch_builtin_host_page_activation(&builtin_harness.runtime, &bridge, "workbench")
            .expect("templated host page activation should resolve")
            .unwrap();
    let builtin_record = builtin_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    let _snapshot = EditorUiCompatibilityHarness::capture_event_journal_delta_snapshot(
        &builtin_harness.runtime.journal(),
        0,
    );

    assert_eq!(builtin_effects, legacy_effects);
    assert_eq!(builtin_record, legacy_record);
}
