use std::sync::OnceLock;

use zircon_runtime::ui::component::UiComponentDescriptorRegistry;

pub(crate) fn retained_component_registry() -> &'static UiComponentDescriptorRegistry {
    static REGISTRY: OnceLock<UiComponentDescriptorRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = UiComponentDescriptorRegistry::editor_showcase();
        for descriptor in UiComponentDescriptorRegistry::material_editor_foundation()
            .descriptors()
            .cloned()
        {
            registry
                .register(descriptor)
                .expect("retained host component descriptors must validate");
        }
        registry
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_registry_includes_material_text_input_contracts() {
        let search = retained_component_registry()
            .descriptor("SearchField")
            .expect("SearchField must be classified before native host projection");

        assert_eq!(search.role, "search-field");
        assert_eq!(search.category.as_str(), "input");
        assert_eq!(search.layout_role.as_str(), "leaf");
    }
}
