use super::*;
use crate::core::editing::operation::{
    DeferredOperationInvocation, OperationCommand, OperationCommandFactory,
    OperationCommandFactoryError, OperationCommandFactoryRegistration, PendingEditRetention,
};
use crate::core::editor_event::{
    EditorEventTransient, EditorOperationEvent, EditorViewportEvent, SelectionHostEvent,
};
use crate::core::editor_operation::EditorOperationInvocation;
use crate::core::play::{PlayEditTarget, PlayKind, PlayStartRequest};
use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::TransformHandleKind;
use crate::ui::template_runtime::builtin::WORKBENCH_WINDOW_DOCUMENT_ID;
use std::sync::Arc;
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::{UiComponentEvent, UiValue},
    dispatch::{
        UiPointerComponentEvent, UiPointerComponentEventReason, UiTemplateActionInvocation,
    },
    event_ui::{UiNodeId, UiTreeId},
    tree::UiVisibility,
};

const ASSET_WINDOW_DOCUMENT_ID: &str = "res://ui/editor/windows/asset_window.zui";

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
        vec![EditorEvent::Viewport(
            EditorViewportEvent::ActivateSceneMode {
                mode: SceneModeActivation::Transform(TransformHandleKind::Scale),
            }
        )]
    );
}

#[test]
fn pointer_component_template_action_reaches_retained_host_operation_dispatch() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_pointer_component_template_action");
    let baseline = harness.journal_len();
    let event = UiPointerComponentEvent::new(
        &UiTreeId::new("editor.plugin.template"),
        UiNodeId::new(7),
        "PluginOpenProject",
        "PluginOpenProject/Click",
        UiEventKind::Click,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        },
        UiPointerComponentEventReason::DefaultClick,
    )
    .with_template_action(UiTemplateActionInvocation::route(
        "file.project.open",
        Default::default(),
    ));

    pane_surface_host(&harness.root_ui).invoke_pointer_component_event(event);

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::WorkbenchMenu(MenuAction::OpenProject)]
    );
}

#[test]
fn pointer_component_command_action_preserves_registry_disabled_policy() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_pointer_command_action_disabled");
    let baseline = harness.journal_len();
    let event = UiPointerComponentEvent::new(
        &UiTreeId::new("editor.workbench.template"),
        UiNodeId::new(8),
        "WorkbenchStop",
        "WorkbenchStop/Click",
        UiEventKind::Click,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        },
        UiPointerComponentEventReason::DefaultClick,
    )
    .with_template_action(UiTemplateActionInvocation::action("runtime.play_mode.exit"));

    pane_surface_host(&harness.root_ui).invoke_pointer_component_event(event);

    let events = harness.delta_events_since(baseline);
    assert!(matches!(
        events.as_slice(),
        [EditorEvent::Operation(EditorOperationEvent::ControlFailure {
            operation_id,
            error,
        })] if operation_id == "runtime.play_mode.exit"
            && error == "editor command runtime.play_mode.exit is disabled by its when clause"
    ));
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

#[test]
fn closing_project_dismisses_the_workbench_command_palette() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_project_close_palette");
    harness.activate_workbench_page();

    let mut host = harness.host.borrow_mut();
    host.open_workbench_command_palette();
    assert!(host.workbench_window_bridge.command_palette_open());

    host.close_project_from_workbench()
        .expect("project close should restore the welcome workspace");

    assert!(!host.workbench_window_bridge.command_palette_open());
}

#[test]
fn resolved_pending_play_decision_clears_the_retained_notification_modal() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_pending_play_decision_clear");
    harness.activate_workbench_page();

    let mut host = harness.host.borrow_mut();
    let stopped = {
        let controller = host.runtime.play_sessions();
        controller
            .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
            .expect("play should start before a deferred edit is queued");
        controller
            .route_edit(
                PlayEditTarget::EditWorkspace,
                deferred_pending_edit("discard"),
            )
            .expect("edit should queue while play is active");
        controller
            .request_stop()
            .expect("stop should surface the real pending-edit prompt")
    };
    host.runtime
        .publish_pending_edit_decision(stopped.pending_edit_prompt.as_ref())
        .expect("pending decision should publish from the controller queue");
    host.sync_pending_play_decisions();
    assert!(workbench_control_bool(
        &host,
        "WorkbenchNotificationCenter",
        "open"
    ));

    let selection_id = host
        .runtime
        .pending_play_decision_options()
        .expect("pending decision options should project")
        .into_iter()
        .last()
        .expect("discard option should be available")
        .selection_id()
        .to_string();
    let effects = host
        .dispatch_componentized_workbench_option_selected(
            "WorkbenchNotificationCenter",
            "PendingEdit/Discard",
            &selection_id,
        )
        .expect("notification-center control should dispatch the selected decision")
        .expect("discard callback should resolve the queued edit");
    assert_eq!(effects.toast_notifications.len(), 1);
    host.apply_dispatch_effects(effects);

    assert!(!workbench_control_bool(
        &host,
        "WorkbenchNotificationCenter",
        "open"
    ));
    assert!(host
        .runtime
        .play_sessions()
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .is_ok());
}

fn deferred_pending_edit(name: &str) -> DeferredOperationInvocation {
    let invocation = EditorOperationInvocation::parse(format!("editor.test.{name}"))
        .expect("test operation should be valid");
    OperationCommandFactoryRegistration::new(
        invocation.operation_id.clone(),
        "retained decision fixture",
        Arc::new(DiscardOnlyPendingEditFactory),
    )
    .with_pending_edit_retention(PendingEditRetention::Lossless)
    .defer(invocation)
    .expect("fixture registration should bind the test operation")
}

struct DiscardOnlyPendingEditFactory;

impl OperationCommandFactory for DiscardOnlyPendingEditFactory {
    fn create(
        &self,
        _invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError> {
        unreachable!("discard resolution must not execute a queued operation")
    }
}
