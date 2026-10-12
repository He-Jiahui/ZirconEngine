use super::binding_path_action_id;

#[test]
fn normalizes_multiple_path_separators_without_dropping_empty_normalized_segments() {
    assert_eq!(
        binding_path_action_id("UiComponentShowcase/ArrayField:SetElement"),
        "ui_component_showcase.array_field.set_element"
    );
    assert_eq!(binding_path_action_id("Alpha/---/Beta"), "alpha..beta");
}
