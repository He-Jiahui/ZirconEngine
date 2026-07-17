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
