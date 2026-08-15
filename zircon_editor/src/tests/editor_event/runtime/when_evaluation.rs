use super::*;
use crate::core::commands::{DocumentKind, EditorCommandDescriptor, WhenClause};
use crate::core::editor_extension::EditorExtensionRegistry;
use crate::core::editor_message::{
    EditorMessagePayload, EditorTopic, SceneModeId, TOPIC_SCENE_INSPECTION,
};
use crate::core::editor_operation::{
    EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
    EditorOperationSource,
};
use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::TransformHandleKind;
use crate::ui::binding::ViewportCommand;
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorHostEventController;
use crate::ui::host::EditorManager;
use crate::ui::retained_host::callback_dispatch::{
    dispatch_componentized_workbench_transform_axis_commit,
    BuiltinWorkbenchWindowTemplateSurfaceBridge,
};
use crate::ui::workbench::view::ViewDescriptorId;
use zircon_runtime_interface::ui::layout::UiSize;

fn register_when_command(
    runtime: &EditorHostEventController,
    operation_path: &EditorOperationPath,
    when: WhenClause,
) {
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation_path.clone(), "When Command")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout))
                .with_when(when),
        )
        .unwrap();
    runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .expect("register when command");
}

#[test]
fn project_open_does_not_fabricate_a_focused_scene_document() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_no_fabricated_document_focus");
    let context = runtime.runtime.context().command_eval().snapshot();

    assert!(WhenClause::ProjectOpen.eval(&context));
    assert!(!WhenClause::FocusedDocumentKind(DocumentKind::parse("scene").unwrap()).eval(&context));
}

#[test]
fn reflection_projects_the_active_viewport_mode_into_command_evaluation() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_viewport_mode_when");
    {
        let mut shell = runtime.runtime.shell().lock();
        shell
            .state
            .apply_viewport_command(&ViewportCommand::ActivateSceneMode(
                SceneModeActivation::Transform(TransformHandleKind::Rotate),
            ))
            .expect("viewport mode command should update the editor mode");
    }

    runtime.runtime.refresh_reflection();
    let context = runtime.runtime.context().command_eval().snapshot();

    assert!(WhenClause::SceneModeActive(SceneModeId::new("scene.transform")).eval(&context));
    assert!(!WhenClause::SceneModeActive(SceneModeId::new("scene.select")).eval(&context));
}

#[test]
fn workbench_position_commit_dispatches_a_typed_inspector_transaction() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_transform_commit");
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("workbench surface should build");

    let effects = dispatch_componentized_workbench_transform_axis_commit(
        &runtime.runtime,
        &bridge,
        "WorkbenchTransformPositionX",
        "Inspector/TransformPositionXCommit",
        "X 4.25",
    )
    .expect("position X commit should be recognized")
    .expect("position X commit should dispatch");

    assert!(effects.render_dirty);
    assert!(effects.presentation_dirty);
    let committed_x = {
        let shell = runtime.runtime.shell().lock();
        let selected = shell
            .state
            .viewport_controller
            .selection()
            .active_primary()
            .expect("default cube should be selected");
        shell
            .state
            .world
            .try_with_world(|scene| scene.find_node(selected).unwrap().transform.translation.x)
            .expect("default world should remain loaded")
    };
    assert_eq!(committed_x, 4.25);
    assert!(runtime.runtime.editor_snapshot().can_undo);
}

#[test]
fn workbench_scale_commit_dispatches_a_typed_inspector_transaction() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_scale_commit");
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("workbench surface should build");

    let effects = dispatch_componentized_workbench_transform_axis_commit(
        &runtime.runtime,
        &bridge,
        "WorkbenchTransformScaleX",
        "Inspector/TransformScaleXCommit",
        "X 2.5",
    )
    .expect("scale X commit should be recognized")
    .expect("scale X commit should dispatch");

    assert!(effects.render_dirty);
    assert!(effects.presentation_dirty);
    let committed_x = {
        let shell = runtime.runtime.shell().lock();
        let selected = shell
            .state
            .viewport_controller
            .selection()
            .active_primary()
            .expect("default cube should be selected");
        shell
            .state
            .world
            .try_with_world(|scene| scene.find_node(selected).unwrap().transform.scale.x)
            .expect("default world should remain loaded")
    };
    assert_eq!(committed_x, 2.5);
    assert!(runtime.runtime.editor_snapshot().can_undo);
}

#[test]
fn workbench_scale_commit_rejects_non_finite_scalars_before_dispatch() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_scale_non_finite");
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("workbench surface should build");

    let error = dispatch_componentized_workbench_transform_axis_commit(
        &runtime.runtime,
        &bridge,
        "WorkbenchTransformScaleX",
        "Inspector/TransformScaleXCommit",
        "X NaN",
    )
    .expect("scale X commit route should be recognized")
    .expect_err("non-finite scale input must be rejected before runtime dispatch");

    assert_eq!(
        error,
        "Inspector transform X value `NaN` must be a finite number"
    );
    assert!(runtime.runtime.journal().records().is_empty());
}

#[test]
fn workbench_position_commit_rejects_non_finite_scalars_before_dispatch() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_transform_non_finite");
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("workbench surface should build");

    let error = dispatch_componentized_workbench_transform_axis_commit(
        &runtime.runtime,
        &bridge,
        "WorkbenchTransformPositionX",
        "Inspector/TransformPositionXCommit",
        "X NaN",
    )
    .expect("position X commit route should be recognized")
    .expect_err("non-finite position input must be rejected before runtime dispatch");

    assert_eq!(
        error,
        "Inspector transform X value `NaN` must be a finite number"
    );
    assert!(runtime.runtime.journal().records().is_empty());
}

#[test]
fn workbench_position_commit_publishes_only_inspection_generation_and_property_delta() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_transform_inspection_delta");
    let topic = EditorTopic::parse(TOPIC_SCENE_INSPECTION).expect("valid scene inspection topic");
    let subscriber = runtime
        .runtime
        .context()
        .bus()
        .register_subscriber([topic])
        .expect("register scene inspection subscriber");
    let (selected, previous_generation) = {
        let shell = runtime.runtime.shell().lock();
        let selected = shell
            .state
            .viewport_controller
            .selection()
            .active_primary()
            .expect("default cube should be selected");
        let generation = shell
            .state
            .world
            .try_with_world(|scene| scene.world_generation())
            .expect("default world should remain loaded");
        (selected, generation)
    };
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("workbench surface should build");

    dispatch_componentized_workbench_transform_axis_commit(
        &runtime.runtime,
        &bridge,
        "WorkbenchTransformPositionX",
        "Inspector/TransformPositionXCommit",
        "X 4.25",
    )
    .expect("position X commit should be recognized")
    .expect("position X commit should dispatch");

    let deliveries = runtime.runtime.context().bus().drain_deliveries(subscriber);
    assert_eq!(deliveries.len(), 1);
    let EditorMessagePayload::SceneInspection(message) = deliveries[0].message().payload() else {
        panic!("transform commit must publish a typed scene inspection message");
    };
    assert_eq!(message.previous_generation(), Some(previous_generation));
    assert!(message.generation() > previous_generation);
    assert_eq!(message.focused_entity(), Some(selected));
    assert!(message.added_anchors().is_empty());
    assert!(message.changed_anchors().is_empty());
    assert!(message.removed_entities().is_empty());
    assert_eq!(message.focused_fields().entity(), Some(selected));
    assert!(!message.focused_fields().requires_resync());
    assert!(!message.focused_fields().changed_properties().is_empty());
    assert!(message.focused_fields().removed_properties().is_empty());
}

#[test]
fn typed_document_focus_tracks_floating_activation_and_focused_close() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_typed_document_focus");
    let manager = runtime
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .expect("editor manager should resolve");

    let ui_asset = manager
        .open_view(ViewDescriptorId::new("editor.ui_asset"), None)
        .expect("UI asset document should open");
    let animation = manager
        .open_view(ViewDescriptorId::new("editor.animation_sequence"), None)
        .expect("animation sequence document should open");
    manager
        .detach_view_to_window(&animation)
        .expect("animation document should detach");
    manager
        .focus_view(&ui_asset)
        .expect("UI asset document should focus");
    runtime.runtime.refresh_reflection();
    let context = runtime.runtime.context().command_eval().snapshot();
    assert_eq!(
        context.focused_document_kind().map(DocumentKind::as_str),
        Some("ui_asset")
    );
    assert!(WhenClause::FocusedDocumentKind(DocumentKind::ui_asset()).eval(&context));

    manager
        .focus_view(&animation)
        .expect("floating animation document should focus");
    runtime.runtime.refresh_reflection();
    let context = runtime.runtime.context().command_eval().snapshot();
    assert_eq!(
        context.focused_document_kind().map(DocumentKind::as_str),
        Some("animation_sequence")
    );
    assert!(WhenClause::FocusedDocumentKind(DocumentKind::animation_sequence()).eval(&context));

    manager
        .close_view(&animation)
        .expect("focused floating animation document should close");
    runtime.runtime.refresh_reflection();
    let context = runtime.runtime.context().command_eval().snapshot();
    assert_eq!(
        context.focused_document_kind().map(DocumentKind::as_str),
        Some("ui_asset")
    );
    assert!(WhenClause::FocusedDocumentKind(DocumentKind::ui_asset()).eval(&context));
}

#[test]
fn remote_list_and_invoke_use_headless_when_even_if_project_is_open() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_remote_headless_when");
    let operation_path = EditorOperationPath::parse("test.project.contextual").unwrap();
    register_when_command(&runtime.runtime, &operation_path, WhenClause::ProjectOpen);

    let listed = runtime
        .runtime
        .handle_operation_control_request(EditorOperationControlRequest::ListOperations);
    assert!(!listed.value.as_ref().unwrap()["operations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|operation| operation["operation_id"] == operation_path.as_str()));

    let invoked = runtime.runtime.handle_operation_control_request(
        EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
            operation_path,
        )),
    );
    assert_eq!(
        invoked.error.as_deref(),
        Some("editor command test.project.contextual is disabled by its when clause")
    );
}

#[test]
fn ui_binding_invoke_uses_the_interactive_editor_snapshot() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_ui_interactive_when");
    let operation_path = EditorOperationPath::parse("test.project.interactive").unwrap();
    register_when_command(&runtime.runtime, &operation_path, WhenClause::ProjectOpen);
    let binding = EditorUiBinding::new(
        "CommandPalette",
        "InteractiveWhen",
        EditorUiEventKind::Submit,
        EditorUiBindingPayload::editor_command(operation_path.as_str()),
    );
    let binding_path = binding.path().native_prefix();

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::RetainedHost)
        .expect("interactive project-open command should be enabled");

    assert_eq!(
        record.operation_id.as_deref(),
        Some(operation_path.as_str())
    );
    assert_eq!(record.binding_path.as_deref(), Some(binding_path.as_str()));
}

#[test]
fn remote_capability_failure_comes_from_effective_when() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_remote_capability_when");
    let operation_path = EditorOperationPath::parse("test.capability.required").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(operation_path.clone(), "Capability")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout))
                .with_required_capabilities(["editor.extension.missing"]),
        )
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .unwrap();

    let response = runtime
        .runtime
        .handle_operation_control_request_from_source(
            EditorOperationSource::Cli,
            EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
                operation_path,
            )),
        );

    assert_eq!(
        response.error.as_deref(),
        Some(
            "editor command test.capability.required requires disabled capabilities: editor.extension.missing"
        )
    );
}

#[test]
fn remote_and_cli_lists_exclude_commands_the_same_sources_cannot_invoke() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_remote_callable_discovery");
    let operation_path = EditorOperationPath::parse("editor.command.palette").unwrap();

    for source in [EditorOperationSource::Remote, EditorOperationSource::Cli] {
        let listed = runtime
            .runtime
            .handle_operation_control_request_from_source(
                source.clone(),
                EditorOperationControlRequest::ListOperations,
            );
        assert!(!listed.value.as_ref().unwrap()["operations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|operation| operation["operation_id"] == operation_path.as_str()));

        let invoked = runtime
            .runtime
            .handle_operation_control_request_from_source(
                source,
                EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
                    operation_path.clone(),
                )),
            );
        assert_eq!(
            invoked.error.as_deref(),
            Some("editor command editor.command.palette is not callable from remote control")
        );
    }
}
