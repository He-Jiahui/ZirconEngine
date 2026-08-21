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

#[test]
fn retained_registry_materializes_the_shared_catalog_union_once() {
    use zircon_runtime::ui::component::UiComponentDescriptorRegistry;

    let retained = retained_component_registry();
    let material = UiComponentDescriptorRegistry::material_editor_foundation_shared();

    assert_eq!(retained.len(), 258);
    assert_eq!(retained.descriptor("Button"), material.descriptor("Button"));
}
