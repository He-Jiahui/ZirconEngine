use super::UiWidgetBehavior;

#[test]
fn editable_text_component_roles_share_one_behavior_classification() {
    for role in [
        "input-field",
        "text-field",
        "line-edit",
        "text-edit",
        "number-field",
        "search-field",
        "search-input",
        "input",
        "input-base",
        "filled-input",
        "outlined-input",
        "textarea-autosize",
        "field-editor",
        "source-editor",
    ] {
        assert_eq!(
            UiWidgetBehavior::infer_from_component_role(role),
            UiWidgetBehavior::TextInput,
            "role {role} must route through the surface text-input pipeline"
        );
    }
}

#[test]
fn editable_text_component_aliases_share_one_behavior_classification() {
    for component in [
        "InputField",
        "TextField",
        "LineEdit",
        "TextEdit",
        "NumberField",
        "SearchField",
        "SearchInput",
        "Input",
        "InputBase",
        "FilledInput",
        "OutlinedInput",
        "TextareaAutosize",
        "FieldEditor",
        "SourceEditor",
    ] {
        assert_eq!(
            UiWidgetBehavior::infer_from_component(component),
            UiWidgetBehavior::TextInput,
            "component {component} must route through the surface text-input pipeline"
        );
    }
}
