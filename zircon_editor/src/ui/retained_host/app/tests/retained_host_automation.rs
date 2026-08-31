use super::*;
use crate::core::editor_event::{
    EditorEvent, EditorEventResult, EditorEventSource, InspectorFieldChange, MenuAction,
    SelectionHostEvent,
};
use crate::ui::binding::{
    EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind, SelectionCommand,
};
use crate::ui::retained_host::app::automation::{
    canonical_cli_binding_path, invoke_supported_binding, normalize_cli_action_records,
};
use zircon_runtime_interface::ui::binding::UiBindingValue;

fn invoke(harness: &ChildWindowHostHarness, binding: EditorUiBinding) {
    let pane = pane_surface_host(&harness.root_ui);
    invoke_supported_binding(&pane, &harness.host, &binding)
        .expect("supported retained-host automation binding should invoke its callback");
    harness.host.borrow_mut().refresh_ui();
}

#[test]
fn explicit_host_shutdown_retires_the_play_gateway_before_drop() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_runtime_shutdown");
    let play_instance = harness
        .host
        .borrow()
        .runtime
        .start_test_play_gateway(
            crate::core::play::PlayKind::Play,
            std::sync::Arc::new(crate::core::gateway::DetachedEditorRuntimeGateway),
        )
        .expect("the test host should attach a play gateway");

    harness.host.borrow_mut().shutdown_runtime_session();

    assert!(harness.host.borrow().runtime_shutdown_receipt().is_some());
    assert!(harness
        .host
        .borrow()
        .runtime
        .gateway_for(crate::core::play::WorldDomain::Play(play_instance))
        .is_none());
}

#[test]
fn automation_selection_uses_hierarchy_callback_and_changes_the_real_selection_model() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_automation_selection");
    let selected = harness
        .host
        .borrow()
        .runtime
        .editor_snapshot()
        .scene_entries
        .iter()
        .last()
        .expect("default scene should expose an authoritative hierarchy row")
        .entity;
    let baseline = harness.journal_len();

    invoke(
        &harness,
        EditorUiBinding::new(
            "Hierarchy",
            "SelectSceneNode",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode {
                node_id: selected,
            }),
        ),
    );

    assert!(harness
        .host
        .borrow()
        .runtime
        .editor_snapshot()
        .scene_entries
        .is_selected(selected));
    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Selection(
            SelectionHostEvent::SelectSceneNode {
                world_domain: crate::core::play::WorldDomain::Edit,
                node_id: selected,
            }
        )]
    );
}

#[test]
fn automation_transform_commit_uses_componentized_callback_and_transaction_path() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_automation_transform");
    let baseline = harness.journal_len();

    invoke(
        &harness,
        EditorUiBinding::new(
            "Inspector",
            "TransformPositionXCommit",
            EditorUiEventKind::Submit,
            EditorUiBindingPayload::inspector_field_batch(
                "entity://selected",
                [InspectorFieldChange::new(
                    "transform.translation.x",
                    UiBindingValue::Float(4.25),
                )],
            ),
        ),
    );

    let snapshot = harness.host.borrow().runtime.editor_snapshot();
    assert_eq!(
        snapshot
            .inspector
            .expect("selection should retain inspector")
            .translation[0],
        "4.25"
    );
    assert!(!harness.delta_events_since(baseline).is_empty());
}

#[test]
fn automation_save_uses_the_surface_callback_route() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_automation_save");
    let baseline = harness.journal_len();

    invoke(
        &harness,
        EditorUiBinding::new(
            "WorkbenchMenuBar",
            "SaveProject",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::menu_action("workbench.project.save"),
        ),
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::WorkbenchMenu(MenuAction::SaveProject)]
    );
}

#[test]
fn automation_undo_and_redo_use_the_surface_callback_route() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_automation_history");
    let baseline = harness.journal_len();

    for action_id in ["workbench.history.undo", "workbench.history.redo"] {
        invoke(
            &harness,
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "History",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action(action_id),
            ),
        );
    }

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![
            EditorEvent::WorkbenchMenu(MenuAction::Undo),
            EditorEvent::WorkbenchMenu(MenuAction::Redo),
        ]
    );
}

#[test]
fn automation_undo_and_redo_restore_and_reapply_an_inspector_transaction() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_automation_history_state");
    let original_translation_x = harness
        .host
        .borrow()
        .runtime
        .editor_snapshot()
        .inspector
        .expect("default selection should expose an inspector")
        .translation[0]
        .clone();

    invoke(
        &harness,
        EditorUiBinding::new(
            "Inspector",
            "TransformPositionXCommit",
            EditorUiEventKind::Submit,
            EditorUiBindingPayload::inspector_field_batch(
                "entity://selected",
                [InspectorFieldChange::new(
                    "transform.translation.x",
                    UiBindingValue::Float(4.25),
                )],
            ),
        ),
    );
    assert_eq!(
        harness
            .host
            .borrow()
            .runtime
            .editor_snapshot()
            .inspector
            .expect("transform commit should retain the inspector")
            .translation[0],
        "4.25"
    );

    for action_id in ["workbench.history.undo", "workbench.history.redo"] {
        invoke(
            &harness,
            EditorUiBinding::new(
                "WorkbenchMenuBar",
                "History",
                EditorUiEventKind::Click,
                EditorUiBindingPayload::menu_action(action_id),
            ),
        );

        let expected_translation_x = if action_id == "workbench.history.undo" {
            &original_translation_x
        } else {
            "4.25"
        };
        assert_eq!(
            harness
                .host
                .borrow()
                .runtime
                .editor_snapshot()
                .inspector
                .expect("history action should retain the inspector")
                .translation[0],
            expected_translation_x
        );
    }
}

#[test]
fn automation_report_evidence_uses_canonical_cli_binding_paths() {
    let selection = EditorUiBinding::new(
        "Hierarchy",
        "SelectSceneNode",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode { node_id: 7 }),
    );
    let position = EditorUiBinding::new(
        "Inspector",
        "TransformPositionXCommit",
        EditorUiEventKind::Submit,
        EditorUiBindingPayload::inspector_field_batch(
            "entity://selected",
            [InspectorFieldChange::new(
                "transform.translation.x",
                UiBindingValue::Float(4.25),
            )],
        ),
    );
    let scale = EditorUiBinding::new(
        "Inspector",
        "TransformScaleXCommit",
        EditorUiEventKind::Submit,
        EditorUiBindingPayload::inspector_field_batch(
            "entity://selected",
            [InspectorFieldChange::new(
                "transform.scale.x",
                UiBindingValue::Float(1.25),
            )],
        ),
    );
    let save = EditorUiBinding::new(
        "WorkbenchMenuBar",
        "SaveProject",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action("workbench.project.save"),
    );
    let undo = EditorUiBinding::new(
        "WorkbenchMenuBar",
        "Undo",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action("workbench.history.undo"),
    );
    let redo = EditorUiBinding::new(
        "WorkbenchMenuBar",
        "Redo",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action("workbench.history.redo"),
    );

    assert_eq!(
        canonical_cli_binding_path(&selection).unwrap(),
        "Hierarchy/SelectCube:onClick"
    );
    assert_eq!(
        canonical_cli_binding_path(&position).unwrap(),
        "Inspector/TransformPositionXCommit:onSubmit"
    );
    assert_eq!(
        canonical_cli_binding_path(&scale).unwrap(),
        "Inspector/TransformScaleXCommit:onSubmit"
    );
    assert_eq!(
        canonical_cli_binding_path(&undo).unwrap(),
        "WorkbenchMenuBar/Undo:onClick"
    );
    assert_eq!(
        canonical_cli_binding_path(&redo).unwrap(),
        "WorkbenchMenuBar/Redo:onClick"
    );
    assert_eq!(
        canonical_cli_binding_path(&save).unwrap(),
        "WorkbenchMenuBar/SaveProject:onClick"
    );
}

#[test]
fn automation_report_marks_retained_callback_records_as_cli_evidence() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_automation_cli_evidence");
    let selected = harness
        .host
        .borrow()
        .runtime
        .editor_snapshot()
        .scene_entries
        .iter()
        .last()
        .expect("default scene should expose an authoritative hierarchy row")
        .entity;
    let binding = EditorUiBinding::new(
        "Hierarchy",
        "SelectSceneNode",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode {
            node_id: selected,
        }),
    );
    let baseline = harness.journal_len();

    invoke(&harness, binding.clone());

    let retained_record = harness.host.borrow().runtime.journal().records()[baseline].clone();
    assert_eq!(retained_record.source, EditorEventSource::RetainedHost);
    let records = normalize_cli_action_records(0, &binding, &[retained_record]).unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].source, EditorEventSource::Cli);
    assert_eq!(
        records[0].binding_path.as_deref(),
        Some("Hierarchy/SelectCube:onClick")
    );
}

#[test]
fn automation_report_rejects_callback_records_with_editor_errors_without_exposing_them() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_automation_callback_failure");
    let selected = harness
        .host
        .borrow()
        .runtime
        .editor_snapshot()
        .scene_entries
        .iter()
        .last()
        .expect("default scene should expose an authoritative hierarchy row")
        .entity;
    let binding = EditorUiBinding::new(
        "Hierarchy",
        "SelectSceneNode",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::selection_command(SelectionCommand::SelectSceneNode {
            node_id: selected,
        }),
    );
    let baseline = harness.journal_len();

    invoke(&harness, binding.clone());

    let mut failed_record = harness.host.borrow().runtime.journal().records()[baseline].clone();
    failed_record.result = EditorEventResult::failure(r"\\?\C:\operations\project-save-failed");
    let error = normalize_cli_action_records(0, &binding, &[failed_record]).unwrap_err();

    assert!(error.contains("Hierarchy/SelectCube:onClick"));
    assert!(error.contains("editor callback failure"));
    assert!(!error.contains("project-save-failed"));
}

#[test]
fn automation_rejects_bindings_without_a_retained_callback_route() {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_retained_automation_unsupported");
    let pane = pane_surface_host(&harness.root_ui);
    let error = invoke_supported_binding(
        &pane,
        &harness.host,
        &EditorUiBinding::new(
            "Viewport",
            "FrameSelection",
            EditorUiEventKind::Click,
            EditorUiBindingPayload::menu_action("workbench.project.open"),
        ),
    )
    .expect_err("unsupported automation bindings must not fall back to direct dispatch");

    assert!(error.contains("unsupported retained-host automation binding"));
    assert!(error.contains("workbench.project.save"));
}

#[test]
fn automation_adapter_names_only_the_real_retained_callbacks() {
    let source = include_str!("../automation.rs");
    assert!(source.contains("invoke_hierarchy_pointer_clicked"));
    assert!(source.contains("invoke_surface_control_edited"));
    assert!(source.contains("invoke_surface_control_clicked"));
    assert!(!source.contains("dispatch_binding("));
}
