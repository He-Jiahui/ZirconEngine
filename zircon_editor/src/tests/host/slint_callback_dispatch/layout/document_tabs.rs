use super::super::support::*;
use crate::core::editor_event::{
    LayoutCommand as EventLayoutCommand, ViewInstanceId as EventViewInstanceId,
};

#[test]
fn builtin_workbench_document_tab_activation_focuses_view_from_template_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_document_focus");
    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let document_tabs = bridge
        .host_projection()
        .node_by_control_id("DocumentTabsRoot")
        .expect("document tabs control should exist in builtin template projection");
    assert!(document_tabs.routes.iter().any(|route| {
        route.event_kind == UiEventKind::Change && route.binding_id == "DocumentTabs/ActivateTab"
    }));

    let effects = dispatch_builtin_workbench_document_tab_activation(
        &harness.runtime,
        &bridge,
        "editor.game#1",
    )
    .expect("builtin document tab activation should resolve through template bridge")
    .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(EventLayoutCommand::FocusView {
            instance_id: EventViewInstanceId::new("editor.game#1"),
        })
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn builtin_workbench_document_tab_close_dispatches_close_view_from_template_binding() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_document_close");
    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();

    let document_tabs = bridge
        .host_projection()
        .node_by_control_id("DocumentTabsRoot")
        .expect("document tabs control should exist in builtin template projection");
    assert!(document_tabs.routes.iter().any(|route| {
        route.event_kind == UiEventKind::Submit && route.binding_id == "DocumentTabs/CloseTab"
    }));

    let effects =
        dispatch_builtin_workbench_document_tab_close(&harness.runtime, &bridge, "editor.game#1")
            .expect("builtin document tab close should resolve through template bridge")
            .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(EventLayoutCommand::CloseView {
            instance_id: EventViewInstanceId::new("editor.game#1"),
        })
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}
