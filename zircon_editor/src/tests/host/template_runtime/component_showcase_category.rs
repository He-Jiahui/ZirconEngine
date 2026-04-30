use crate::ui::template_runtime::{
    EditorUiHostRuntime, SlintUiHostProjection, SlintUiHostValue, UiComponentShowcaseDemoEventInput,
};

fn showcase_binding(
    runtime: &EditorUiHostRuntime,
    binding_id: &str,
) -> crate::ui::binding::EditorUiBinding {
    runtime
        .project_document("editor.window.ui_component_showcase")
        .unwrap()
        .bindings
        .into_iter()
        .find(|binding| binding.binding_id == binding_id)
        .unwrap_or_else(|| panic!("missing showcase binding `{binding_id}`"))
        .binding
}

fn apply_showcase_binding(
    runtime: &mut EditorUiHostRuntime,
    binding_id: &str,
    input: UiComponentShowcaseDemoEventInput,
) {
    let binding = showcase_binding(runtime, binding_id);
    runtime
        .apply_showcase_demo_binding(&binding, input)
        .unwrap();
}

fn project_showcase(runtime: &EditorUiHostRuntime) -> SlintUiHostProjection {
    let projection = runtime
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap()
}

#[test]
fn showcase_category_selection_filters_projected_demo_controls() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowDataCategory",
        UiComponentShowcaseDemoEventInput::None,
    );

    let host_projection = project_showcase(&runtime);
    assert!(host_projection
        .node_by_control_id("ArrayFieldDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("MapFieldDemo").is_some());
    assert!(host_projection
        .node_by_control_id("PropertyRowDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("InspectorSectionDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("ButtonDemo").is_none());
    assert!(host_projection.node_by_control_id("DropdownDemo").is_none());
    assert!(host_projection
        .node_by_control_id("ColorFieldDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("AssetFieldDemo")
        .is_none());
    assert!(host_projection.node_by_control_id("LabelDemo").is_none());
    assert_eq!(
        host_projection
            .node_by_control_id("ShowDataCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&SlintUiHostValue::Bool(true))
    );
    assert_eq!(
        host_projection
            .node_by_control_id("ShowInputCategory")
            .and_then(|node| node.properties.get("selected")),
        Some(&SlintUiHostValue::Bool(false))
    );

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowInputCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    let host_projection = project_showcase(&runtime);
    assert!(host_projection.node_by_control_id("ButtonDemo").is_some());
    assert!(host_projection
        .node_by_control_id("InputFieldDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("TextFieldDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("DropdownDemo").is_none());
    assert!(host_projection
        .node_by_control_id("ColorFieldDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("AssetFieldDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("ArrayFieldDemo")
        .is_none());
    assert!(host_projection.node_by_control_id("LabelDemo").is_none());

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowNumericCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    let host_projection = project_showcase(&runtime);
    assert!(host_projection
        .node_by_control_id("NumberFieldDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("ColorFieldDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("Vector3FieldDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("ButtonDemo").is_none());
    assert!(host_projection.node_by_control_id("DropdownDemo").is_none());
    assert!(host_projection
        .node_by_control_id("AssetFieldDemo")
        .is_none());
    assert!(host_projection
        .node_by_control_id("ArrayFieldDemo")
        .is_none());

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowSelectionCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    let host_projection = project_showcase(&runtime);
    assert!(host_projection.node_by_control_id("DropdownDemo").is_some());
    assert!(host_projection
        .node_by_control_id("SearchSelectDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("ButtonDemo").is_none());
    assert!(host_projection
        .node_by_control_id("ColorFieldDemo")
        .is_none());

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowReferenceCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    let host_projection = project_showcase(&runtime);
    assert!(host_projection
        .node_by_control_id("AssetFieldDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("InstanceFieldDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("ButtonDemo").is_none());
    assert!(host_projection.node_by_control_id("DropdownDemo").is_none());
    assert!(host_projection
        .node_by_control_id("ArrayFieldDemo")
        .is_none());

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowFeedbackCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    let host_projection = project_showcase(&runtime);
    assert!(host_projection
        .node_by_control_id("ProgressBarDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("HelpRowDemo").is_some());
    assert!(host_projection.node_by_control_id("LabelDemo").is_none());
    assert!(host_projection.node_by_control_id("ButtonDemo").is_none());

    apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/ShowAllCategory",
        UiComponentShowcaseDemoEventInput::None,
    );
    let host_projection = project_showcase(&runtime);
    assert!(host_projection.node_by_control_id("LabelDemo").is_some());
    assert!(host_projection.node_by_control_id("ButtonDemo").is_some());
    assert!(host_projection
        .node_by_control_id("ColorFieldDemo")
        .is_some());
    assert!(host_projection.node_by_control_id("DropdownDemo").is_some());
    assert!(host_projection
        .node_by_control_id("AssetFieldDemo")
        .is_some());
    assert!(host_projection
        .node_by_control_id("ArrayFieldDemo")
        .is_some());
}
