use zircon_runtime::asset::AssetUri;

use super::*;
use crate::core::editor_operation::EditorOperationPath;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::ViewHost;

fn animation_instance(descriptor_id: &str) -> ViewInstance {
    ViewInstance {
        instance_id: ViewInstanceId::new("editor.animation#route-test"),
        descriptor_id: ViewDescriptorId::new(descriptor_id),
        title: "Animation".to_string(),
        serializable_payload: serde_json::Value::Null,
        dirty: false,
        host: ViewHost::Document(MainPageId::workbench(), Vec::new()),
    }
}

fn animation_route(operation: &str) -> AssetToolkitOpenRoute {
    AssetToolkitOpenRoute::new(
        AssetUri::parse("res://animation/hero.zranim")
            .expect("animation fixture locator must be canonical"),
        EditorOperationPath::parse(operation).expect("animation fixture operation must be valid"),
    )
}

#[test]
fn route_operation_selects_the_registered_animation_document_kind() {
    for (descriptor_id, operation, expected_kind) in [
        (
            "editor.animation_sequence",
            "timeline_sequence.authoring.open",
            AnimationEditorDocumentKind::Sequence,
        ),
        (
            "editor.animation_graph",
            "animation_graph.authoring.open_graph",
            AnimationEditorDocumentKind::Graph,
        ),
        (
            "editor.animation_graph",
            "animation_graph.authoring.open_state_machine",
            AnimationEditorDocumentKind::StateMachine,
        ),
    ] {
        assert_eq!(
            document_kind_for_route(
                &animation_instance(descriptor_id),
                &animation_route(operation),
            )
            .expect("registered route should select its document kind"),
            expected_kind
        );
    }
}

#[test]
fn route_rejects_an_operation_with_the_wrong_view_descriptor() {
    let result = document_kind_for_route(
        &animation_instance("editor.animation_graph"),
        &animation_route("timeline_sequence.authoring.open"),
    );

    assert!(result.is_err());
}

#[test]
fn route_rejects_operations_outside_the_registered_animation_toolkits() {
    let result = document_kind_for_route(
        &animation_instance("editor.animation_graph"),
        &animation_route("animation_graph.authoring.compile"),
    );

    assert!(result.is_err());
}
