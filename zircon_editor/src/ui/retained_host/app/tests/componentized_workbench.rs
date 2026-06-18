use super::*;
use crate::core::editor_event::{EditorEventTransient, EditorViewportEvent, SelectionHostEvent};
use crate::scene::viewport::SceneViewportTool;
use crate::ui::template_runtime::builtin::WORKBENCH_WINDOW_DOCUMENT_ID;
use zircon_runtime_interface::ui::tree::UiVisibility;

const ASSET_WINDOW_DOCUMENT_ID: &str = "editor.window.asset";

#[test]
fn root_componentized_workbench_surface_tool_click_updates_bridge_and_runtime() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_componentized_workbench_surface_tool");
    harness.activate_workbench_page();
    assert_eq!(
        harness
            .host
            .borrow()
            .active_activity_window_template_document_id()
            .as_deref(),
        Some(WORKBENCH_WINDOW_DOCUMENT_ID)
    );
    let baseline = harness.journal_len();

    pane_surface_host(&harness.root_ui)
        .invoke_surface_control_clicked("WorkbenchToolScale".into(), "Tool/Scale".into());

    let host = harness.host.borrow();
    assert!(!workbench_control_bool(
        &host,
        "WorkbenchToolMove",
        "selected"
    ));
    assert!(workbench_control_bool(
        &host,
        "WorkbenchToolScale",
        "selected"
    ));
    assert!(workbench_control_bool(
        &host,
        "WorkbenchToolScale",
        "checked"
    ));
    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Viewport(EditorViewportEvent::SetTool {
            tool: SceneViewportTool::Scale,
        })]
    );
}

#[test]
fn root_componentized_workbench_surface_preview_controls_update_bridge_state() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_componentized_workbench_surface_preview");
    harness.activate_workbench_page();
    let baseline = harness.journal_len();

    pane_surface_host(&harness.root_ui).invoke_surface_control_clicked(
        "WorkbenchCheckboxOff".into(),
        "ComponentLab/CheckboxOffToggle".into(),
    );
    pane_surface_host(&harness.root_ui).invoke_surface_control_clicked(
        "WorkbenchDrawerTabConsole".into(),
        "PanelTab/ComponentDrawerConsole".into(),
    );

    let host = harness.host.borrow();
    assert!(workbench_control_bool(
        &host,
        "WorkbenchCheckboxOff",
        "checked"
    ));
    assert!(!workbench_control_bool(
        &host,
        "WorkbenchDrawerTabComponents",
        "selected"
    ));
    assert!(workbench_control_bool(
        &host,
        "WorkbenchDrawerTabConsole",
        "selected"
    ));
    assert_eq!(
        workbench_control_visibility(&host, "WorkbenchComponentDrawerBody"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        workbench_control_visibility(&host, "WorkbenchComponentDrawerConsoleBody"),
        Some(UiVisibility::Visible)
    );
    drop(host);

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![
            EditorEvent::Transient(EditorEventTransient::PressNode {
                node_path: "ComponentLab/CheckboxOffToggle".to_string(),
                pressed: false,
            }),
            EditorEvent::Transient(EditorEventTransient::PressNode {
                node_path: "PanelTab/ComponentDrawerConsole".to_string(),
                pressed: false,
            }),
        ]
    );
}

#[test]
fn root_componentized_workbench_surface_binding_keeps_clicked_control_source() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_componentized_workbench_surface_source");
    harness.activate_workbench_page();
    let baseline = harness.journal_len();

    pane_surface_host(&harness.root_ui).invoke_surface_control_clicked(
        "WorkbenchScenePlayerItem".into(),
        "Hierarchy/SelectEntity".into(),
    );

    let host = harness.host.borrow();
    assert!(!workbench_control_bool(
        &host,
        "WorkbenchScenePropsItem",
        "selected"
    ));
    assert!(workbench_control_bool(
        &host,
        "WorkbenchScenePlayerItem",
        "selected"
    ));
    drop(host);

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Selection(
            SelectionHostEvent::SelectSceneNode { node_id: 0 }
        )]
    );
}

#[test]
fn componentized_workbench_surface_control_requires_active_workbench_window_template() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_componentized_workbench_template_gate");
    harness.open_view("editor.asset_browser_window");
    {
        let host = harness.host.borrow();
        assert_eq!(
            host.active_activity_window_template_document_id()
                .as_deref(),
            Some(ASSET_WINDOW_DOCUMENT_ID)
        );
    }

    let mut host = harness.host.borrow_mut();
    let selected_before = workbench_control_bool(&host, "WorkbenchToolScale", "selected");
    assert!(host
        .dispatch_componentized_workbench_surface_control("WorkbenchToolScale", "Tool/Scale")
        .is_none());
    assert_eq!(
        workbench_control_bool(&host, "WorkbenchToolScale", "selected"),
        selected_before
    );
}
