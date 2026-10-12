use super::*;

#[test]
fn runtime_component_projection_preserves_primary_click_binding_id() {
    let mut button = projected_node("Button", [("text", Value::String("Select".to_owned()))]);
    button.bindings.push(RetainedUiHostBindingProjection {
        binding_id: "HierarchyPaneBody/SelectRoot".to_owned(),
        action_id: String::new(),
        event_kind: UiEventKind::Click,
        route_id: None,
        template_action_source: None,
        template_action: None,
    });

    let projected = host_template_node(button)
        .expect("button with a primary click binding should project into the host contract");

    assert_eq!(
        projected.binding_id.as_str(),
        "HierarchyPaneBody/SelectRoot"
    );
    assert_eq!(projected.action_id.as_str(), "");
}

#[test]
fn runtime_component_projection_carries_primary_click_menu_action_id() {
    let mut button = projected_node("Button", [("text", Value::String("Open".to_owned()))]);
    button.bindings.push(RetainedUiHostBindingProjection {
        binding_id: "WorkbenchGeneratedBottom/OpenPanel".to_owned(),
        action_id: "workbench.generated_bottom.open_panel.invoke".to_owned(),
        event_kind: UiEventKind::Click,
        route_id: None,
        template_action_source: None,
        template_action: None,
    });

    let projected = host_template_node(button)
        .expect("button with a primary click menu action should project into the host contract");

    assert_eq!(
        projected.binding_id.as_str(),
        "WorkbenchGeneratedBottom/OpenPanel"
    );
    assert_eq!(
        projected.action_id.as_str(),
        "workbench.generated_bottom.open_panel.invoke"
    );
}

#[test]
fn runtime_component_projection_derives_text_edit_targets_from_change_and_submit_bindings() {
    let mut input = projected_node(
        "InputField",
        [("value_text", Value::String("Draft".into()))],
    );
    input.control_id = Some("NameField".to_owned());
    input.bindings.push(RetainedUiHostBindingProjection {
        binding_id: "InspectorView/NameField".to_owned(),
        action_id: String::new(),
        event_kind: UiEventKind::Change,
        route_id: None,
        template_action_source: None,
        template_action: None,
    });
    input.bindings.push(RetainedUiHostBindingProjection {
        binding_id: "InspectorView/ApplyBatchButton".to_owned(),
        action_id: String::new(),
        event_kind: UiEventKind::Submit,
        route_id: None,
        template_action_source: None,
        template_action: None,
    });

    let projected = host_template_node(input)
        .expect("input with change and commit bindings should project edit targets");

    assert_eq!(projected.component_role.as_str(), "input-field");
    assert_eq!(projected.component_category.as_str(), "input");
    assert_eq!(projected.component_layout_role.as_str(), "leaf");
    assert_eq!(
        projected.edit_action_id.as_str(),
        "inspector_view.name_field"
    );
    assert_eq!(
        projected.commit_action_id.as_str(),
        "inspector_view.apply_batch_button"
    );
}
