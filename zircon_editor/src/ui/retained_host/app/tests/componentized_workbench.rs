use super::*;
use crate::core::editing::operation::EditOperationTarget;
use crate::core::editing::operation::{
    DeferredOperationInvocation, OperationCommand, OperationCommandFactory,
    OperationCommandFactoryError, OperationCommandFactoryRegistration, PendingEditRetention,
};
use crate::core::editor_event::{
    EditorEventTransient, EditorOperationEvent, EditorViewportEvent, SelectionHostEvent,
};
use crate::core::editor_operation::EditorOperationInvocation;
use crate::core::play::{PlayKind, PlayStartRequest};
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
fn context_menu_open_commits_projection_without_full_shell_recompute() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_context_menu_projection");
    harness.activate_workbench_page();
    let slow_path_rebuilds = harness
        .host
        .borrow()
        .invalidation
        .diagnostics_snapshot()
        .slow_path_rebuild_count;

    {
        let mut host = harness.host.borrow_mut();
        host.dispatch_workbench_context_menu_requested(WorkbenchContextMenuRequestData {
            target_control_id: "WorkbenchScenePropsItem".into(),
            target_action_id: "workbench.hierarchy.select_props".into(),
            target_dispatch_kind: "workbench".into(),
            target_role: "tree-row".into(),
            target_value_text: "Props".into(),
            target_path: "workbench://scene/props".into(),
            popup_anchor_x: 128.0,
            popup_anchor_y: 256.0,
            menu_items: vec!["Open|icon=folder".into(), "Delete|danger,icon=trash".into()],
        });

        assert!(workbench_control_bool(
            &host,
            "WorkbenchContextMenu",
            "popup_open"
        ));
        assert!(host
            .workbench_window_bridge
            .has_pending_host_projection_commit());
        host.recompute_if_dirty();
        assert!(!host
            .workbench_window_bridge
            .has_pending_host_projection_commit());
        assert_eq!(
            host.invalidation
                .diagnostics_snapshot()
                .slow_path_rebuild_count,
            slow_path_rebuilds,
            "context-menu changed rows must patch without rebuilding the root shell"
        );
    }
}

#[test]
fn componentized_viewport_chrome_patch_queues_center_status_damage() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_componentized_chrome_damage");
    harness.activate_workbench_page();
    let presentation = harness.root_ui.get_host_presentation();
    let center = presentation.host_layout.center_band_frame;
    let status = presentation.host_layout.status_bar_frame;
    let _ = harness.root_ui.take_external_redraw_for_test();
    let slow_path_rebuilds = harness
        .host
        .borrow()
        .invalidation
        .diagnostics_snapshot()
        .slow_path_rebuild_count;

    pane_surface_host(&harness.root_ui)
        .invoke_surface_control_clicked("WorkbenchToolScale".into(), "Tool/Scale".into());
    harness.host.borrow_mut().recompute_if_dirty();

    let redraw = harness.root_ui.take_external_redraw_for_test();
    let damage = redraw
        .damage_region()
        .expect("chrome projection should queue bounded center/status damage");
    for expected in [&center, &status] {
        assert!(damage.x <= expected.x);
        assert!(damage.y <= expected.y);
        assert!(damage.right() >= expected.right());
        assert!(damage.bottom() >= expected.bottom());
    }
    assert!(redraw.requires_frame_update());
    assert_eq!(
        harness
            .host
            .borrow()
            .invalidation
            .diagnostics_snapshot()
            .slow_path_rebuild_count,
        slow_path_rebuilds,
        "chrome and changed-row damage should merge without a full shell rebuild"
    );
}

#[test]
fn componentized_viewport_chrome_patch_updates_native_presenter_damage() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_componentized_native_chrome_damage");
    harness.activate_workbench_page();
    let scene = harness.detach_view_to_child_window("editor.scene#1", "window:scene");
    let hierarchy = harness.detach_view_to_child_window("editor.hierarchy#1", "window:hierarchy");
    let native_scene_mode = |ui: &UiHostWindow, window_id: &str| {
        ui.get_host_presentation()
            .native_floating_surface_data
            .floating_windows
            .iter()
            .find(|window| window.window_id.as_str() == window_id)
            .filter(|window| window.active_pane.kind.as_str() == "Scene")
            .map(|window| window.active_pane.viewport.mode.to_string())
            .expect("native scene presenter should contain its detached scene pane")
    };
    assert_ne!(native_scene_mode(&scene, "window:scene"), "Transform.Scale");
    let scene_presentation = scene.get_host_presentation();
    let scene_bounds = scene_presentation
        .native_floating_surface_data
        .native_window_bounds;
    let scene_header_height = scene_presentation
        .native_floating_surface_data
        .header_height_px
        .clamp(0.0, scene_bounds.height);
    let _ = scene.take_external_redraw_for_test();
    let _ = hierarchy.take_external_redraw_for_test();
    let slow_path_rebuilds = harness
        .host
        .borrow()
        .invalidation
        .diagnostics_snapshot()
        .slow_path_rebuild_count;

    pane_surface_host(&harness.root_ui)
        .invoke_surface_control_clicked("WorkbenchToolScale".into(), "Tool/Scale".into());
    harness.host.borrow_mut().recompute_if_dirty();

    assert_eq!(native_scene_mode(&scene, "window:scene"), "Transform.Scale");
    let redraw = scene.take_external_redraw_for_test();
    let damage = redraw
        .damage_region()
        .expect("native presenter chrome patch should queue its own damage");
    assert_eq!(damage.x, 0.0);
    assert_eq!(damage.y, scene_header_height);
    assert_eq!(damage.width, scene_bounds.width);
    assert_eq!(damage.height, scene_bounds.height - scene_header_height);
    assert!(redraw.requires_frame_update());
    let hierarchy_redraw = hierarchy.take_external_redraw_for_test();
    assert!(!hierarchy_redraw.request_redraw());
    assert_eq!(hierarchy_redraw.damage_region(), None);
    assert_eq!(
        harness
            .host
            .borrow()
            .invalidation
            .diagnostics_snapshot()
            .slow_path_rebuild_count,
        slow_path_rebuilds,
        "native chrome patching must not rebuild the root shell"
    );
}

#[test]
fn repeated_dispatch_error_commits_pending_workbench_projection() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_repeated_dispatch_error_projection");
    harness.activate_workbench_page();
    let mut host = harness.host.borrow_mut();
    let error = "repeated componentized dispatch failure".to_string();

    host.apply_dispatch_result(Err(error.clone()));
    host.recompute_if_dirty();
    assert!(!host
        .workbench_window_bridge
        .has_pending_host_projection_commit());
    let slow_path_rebuilds = host
        .invalidation
        .diagnostics_snapshot()
        .slow_path_rebuild_count;

    host.workbench_window_bridge
        .dispatch_control_state("WorkbenchToolScale", UiEventKind::Click)
        .expect("componentized bridge state should refresh")
        .expect("tool control should have a click binding");
    assert!(host
        .workbench_window_bridge
        .has_pending_host_projection_commit());

    host.apply_dispatch_result(Err(error));
    host.recompute_if_dirty();

    assert!(!host
        .workbench_window_bridge
        .has_pending_host_projection_commit());
    assert_eq!(
        host.invalidation
            .diagnostics_snapshot()
            .slow_path_rebuild_count,
        slow_path_rebuilds,
        "repeated error should commit pending rows without rebuilding the shell"
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
            SelectionHostEvent::SelectSceneNode {
                world_domain: crate::core::play::WorldDomain::Edit,
                node_id: 0,
            }
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

    host.request_project_close()
        .expect("project close should restore the welcome workspace");

    assert!(!host.workbench_window_bridge.command_palette_open());
}

#[test]
fn resolved_pending_decision_clears_the_retained_notification_modal() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_pending_play_decision_clear");
    harness.activate_workbench_page();

    let mut host = harness.host.borrow_mut();
    {
        let controller = host.runtime.play_sessions();
        controller
            .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
            .expect("play should start before a deferred edit is queued");
        controller
            .route_edit(
                EditOperationTarget::EditWorkspace,
                deferred_pending_edit("discard"),
            )
            .expect("edit should queue while play is active");
        controller
            .request_stop()
            .expect("stop should surface the real pending-edit prompt");
    }
    host.runtime
        .reconcile_pending_play_decision_from_controller()
        .expect("pending decision should publish from the controller queue");
    host.sync_pending_activity_decisions();
    assert!(workbench_control_bool(
        &host,
        "WorkbenchNotificationCenter",
        "open"
    ));

    let selection_id = {
        let center = host
            .runtime
            .context()
            .notifications()
            .decisions()
            .expect("test notification service should expose its Decision center");
        let snapshot = center
            .pending_snapshot()
            .into_iter()
            .next()
            .expect("pending Decision should be available");
        let option = snapshot
            .notification()
            .options()
            .last()
            .expect("discard option should be available");
        format!("{}:{}", snapshot.notification().id(), option.id())
    };
    let slow_path_rebuilds = host
        .invalidation
        .diagnostics_snapshot()
        .slow_path_rebuild_count;
    let effects = host
        .dispatch_componentized_workbench_option_selected(
            "WorkbenchNotificationCenter",
            "PendingEdit/Discard",
            &selection_id,
        )
        .expect("notification-center control should dispatch the selected decision")
        .expect("discard callback should resolve the queued edit");
    assert!(effects.toast_notifications.is_empty());
    host.apply_dispatch_effects(effects);
    host.recompute_if_dirty();
    assert_eq!(
        host.invalidation
            .diagnostics_snapshot()
            .slow_path_rebuild_count,
        slow_path_rebuilds,
        "decision buttons must close through changed notification rows"
    );
    host.runtime
        .pump_pending_play_decision_receipts()
        .expect("the Play receipt should consume through its owning adapter");
    host.sync_pending_activity_decisions();

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
        EditOperationTarget::EditWorkspace,
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
