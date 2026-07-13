use super::*;
use crate::core::commands::{DocumentKind, EditorCommandDescriptor, WhenClause};
use crate::core::editor_extension::EditorExtensionRegistry;
use crate::core::editor_operation::{
    EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
    EditorOperationSource,
};
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorHostEventController;
use crate::ui::host::EditorManager;
use crate::ui::workbench::view::ViewDescriptorId;

fn register_when_command(
    runtime: &EditorHostEventController,
    operation_path: &EditorOperationPath,
    when: WhenClause,
) {
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::pending_operation(operation_path.clone(), "When Command")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout))
                .with_when(when),
        )
        .unwrap();
    runtime
        .register_editor_extension(extension)
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

    let record = runtime
        .runtime
        .dispatch_binding(binding, EditorEventSource::RetainedHost)
        .expect("interactive project-open command should be enabled");

    assert_eq!(
        record.operation_id.as_deref(),
        Some(operation_path.as_str())
    );
}

#[test]
fn remote_capability_failure_comes_from_effective_when() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_remote_capability_when");
    let operation_path = EditorOperationPath::parse("test.capability.required").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::pending_operation(operation_path.clone(), "Capability")
                .with_event(EditorEvent::WorkbenchMenu(MenuAction::ResetLayout))
                .with_required_capabilities(["editor.extension.missing"]),
        )
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(extension)
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
