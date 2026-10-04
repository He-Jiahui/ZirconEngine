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
    let showcase = UiComponentDescriptorRegistry::editor_showcase_shared();
    let material = UiComponentDescriptorRegistry::material_editor_foundation_shared();
    let overlap_count = showcase
        .component_ids()
        .filter(|component_id| material.contains(component_id))
        .count();

    assert_eq!(showcase.len(), 71);
    assert_eq!(material.len(), 256);
    assert_eq!(overlap_count, 69);
    assert_eq!(retained.len(), 258);
    assert_eq!(retained.descriptor("Button"), material.descriptor("Button"));
}

#[test]
fn retained_registry_preserves_the_legacy_material_wins_union() {
    use zircon_runtime::ui::component::UiComponentDescriptorRegistry;

    let mut legacy = UiComponentDescriptorRegistry::new();
    for descriptor in UiComponentDescriptorRegistry::editor_showcase_shared()
        .descriptors()
        .chain(UiComponentDescriptorRegistry::material_editor_foundation_shared().descriptors())
        .cloned()
    {
        legacy
            .register(descriptor)
            .expect("legacy retained host component descriptors must validate");
    }

    let retained = retained_component_registry();
    assert_eq!(
        retained.component_ids().collect::<Vec<_>>(),
        legacy.component_ids().collect::<Vec<_>>()
    );
    for component_id in legacy.component_ids() {
        assert_eq!(
            retained.descriptor(component_id),
            legacy.descriptor(component_id)
        );
    }
}
