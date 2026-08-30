use std::sync::OnceLock;

use zircon_runtime::ui::component::UiComponentDescriptorRegistry;

pub(crate) fn retained_component_registry() -> &'static UiComponentDescriptorRegistry {
    static REGISTRY: OnceLock<UiComponentDescriptorRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry =
            UiComponentDescriptorRegistry::material_editor_foundation_shared().clone();
        for descriptor in UiComponentDescriptorRegistry::editor_showcase_shared().descriptors() {
            if registry.contains(&descriptor.id) {
                continue;
            }
            registry
                .register(descriptor.clone())
                .expect("retained host component descriptors must validate");
        }
        registry
    })
}
