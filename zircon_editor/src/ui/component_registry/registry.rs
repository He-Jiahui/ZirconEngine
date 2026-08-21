use std::sync::OnceLock;

use zircon_runtime::ui::component::UiComponentDescriptorRegistry;

pub(crate) fn retained_component_registry() -> &'static UiComponentDescriptorRegistry {
    static REGISTRY: OnceLock<UiComponentDescriptorRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = UiComponentDescriptorRegistry::new();
        for descriptor in UiComponentDescriptorRegistry::editor_showcase_shared()
            .descriptors()
            .chain(UiComponentDescriptorRegistry::material_editor_foundation_shared().descriptors())
            .cloned()
        {
            registry
                .register(descriptor)
                .expect("retained host component descriptors must validate");
        }
        registry
    })
}
