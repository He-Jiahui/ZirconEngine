use crate::ui::template_runtime::{
    EditorUiHostRuntime, SlintUiHostValue, UiComponentShowcaseDemoEventInput,
};
use zircon_runtime::ui::component::UiValue;

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

fn try_apply_showcase_binding(
    runtime: &mut EditorUiHostRuntime,
    binding_id: &str,
    input: UiComponentShowcaseDemoEventInput,
) -> Result<(), crate::ui::template_runtime::UiComponentShowcaseDemoError> {
    let binding = showcase_binding(runtime, binding_id);
    runtime.apply_showcase_demo_binding(&binding, input)
}

fn project_showcase_dropdown(
    runtime: &EditorUiHostRuntime,
) -> crate::ui::template_runtime::SlintUiHostNodeModel {
    project_showcase_node(runtime, "DropdownDemo")
}

fn project_showcase_node(
    runtime: &EditorUiHostRuntime,
    control_id: &str,
) -> crate::ui::template_runtime::SlintUiHostNodeModel {
    let projection = runtime
        .project_document("editor.window.ui_component_showcase")
        .unwrap();
    let surface = runtime
        .build_shared_surface("editor.window.ui_component_showcase")
        .unwrap();
    runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap()
        .node_by_control_id(control_id)
        .unwrap()
        .clone()
}

#[test]
fn showcase_dropdown_uses_authored_multiple_and_disabled_option_metadata() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    try_apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/DropdownChanged",
        UiComponentShowcaseDemoEventInput::SelectOption {
            option_id: "editor".to_string(),
            selected: true,
        },
    )
    .unwrap();

    let dropdown = project_showcase_dropdown(&runtime);
    assert_eq!(
        dropdown.properties.get("value"),
        Some(&SlintUiHostValue::Array(vec![
            SlintUiHostValue::String("runtime".to_string()),
            SlintUiHostValue::String("editor".to_string()),
        ]))
    );
    assert_eq!(dropdown.selection_state.as_deref(), Some("multi"));

    let error = try_apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/DropdownChanged",
        UiComponentShowcaseDemoEventInput::SelectOption {
            option_id: "debug".to_string(),
            selected: true,
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("disabled option debug"));

    let dropdown = project_showcase_dropdown(&runtime);
    assert_eq!(dropdown.validation_level.as_deref(), Some("error"));
    assert_eq!(
        dropdown.validation_message.as_deref(),
        Some("disabled option `debug` cannot be selected")
    );
    assert_eq!(
        dropdown.properties.get("value"),
        Some(&SlintUiHostValue::Array(vec![
            SlintUiHostValue::String("runtime".to_string()),
            SlintUiHostValue::String("editor".to_string()),
        ]))
    );
}

#[test]
fn showcase_search_select_query_edit_is_retained_and_projected() {
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    try_apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/SearchSelectQueryChanged",
        UiComponentShowcaseDemoEventInput::Value(UiValue::String("vector".to_string())),
    )
    .unwrap();

    let search = project_showcase_node(&runtime, "SearchSelectDemo");
    assert_eq!(search.value_text.as_deref(), Some("runtime.ui.NumberField"));
    assert_eq!(
        search.properties.get("query"),
        Some(&SlintUiHostValue::String("vector".to_string()))
    );

    try_apply_showcase_binding(
        &mut runtime,
        "UiComponentShowcase/SearchSelectChanged",
        UiComponentShowcaseDemoEventInput::SelectOption {
            option_id: "runtime.ui.RangeField".to_string(),
            selected: true,
        },
    )
    .unwrap();

    let search = project_showcase_node(&runtime, "SearchSelectDemo");
    assert_eq!(search.value_text.as_deref(), Some("runtime.ui.RangeField"));
    assert_eq!(
        search.properties.get("query"),
        Some(&SlintUiHostValue::String("vector".to_string()))
    );
    assert!(runtime
        .showcase_demo_state()
        .event_log()
        .iter()
        .any(|entry| entry.action == "ValueChanged.SearchSelectQuery"
            && entry.control_id == "SearchSelectDemo"
            && entry.value_text.as_deref() == Some("vector")));
}
