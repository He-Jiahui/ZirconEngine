use super::retained_component_registry;

#[test]
fn retained_registry_includes_material_text_input_contracts() {
    let search = retained_component_registry()
        .descriptor("SearchField")
        .expect("SearchField must be classified before native host projection");

    assert_eq!(search.role, "search-field");
    assert_eq!(search.category.as_str(), "input");
    assert_eq!(search.layout_role.as_str(), "leaf");
}
