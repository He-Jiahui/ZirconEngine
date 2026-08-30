use std::sync::OnceLock;

use crate::ui::component::UiComponentDescriptorRegistry;

use self::descriptors::editor_showcase_descriptors;

mod descriptor_builders;
mod descriptors;

static EDITOR_SHOWCASE_REGISTRY: OnceLock<UiComponentDescriptorRegistry> = OnceLock::new();

impl UiComponentDescriptorRegistry {
    /// Builds the Runtime UI component catalog used by the editor showcase.
    pub fn editor_showcase() -> Self {
        Self::editor_showcase_shared().clone()
    }

    /// Returns the process-wide read-only editor showcase catalog.
    pub fn editor_showcase_shared() -> &'static Self {
        EDITOR_SHOWCASE_REGISTRY.get_or_init(build_editor_showcase_registry)
    }
}

fn build_editor_showcase_registry() -> UiComponentDescriptorRegistry {
    let mut registry = UiComponentDescriptorRegistry::new();
    for descriptor in editor_showcase_descriptors() {
        registry
            .register(descriptor)
            .expect("built-in UI component descriptors must validate");
    }
    registry
}

#[cfg(test)]
mod tests {
    #[test]
    fn editor_showcase_catalog_builds_on_small_stack() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let registry = super::build_editor_showcase_registry();
                assert!(registry.len() >= 40);
                assert!(registry.contains("Container"));
                assert!(registry.contains("ContextActionMenu"));
            })
            .expect("spawn small-stack showcase catalog test")
            .join()
            .expect("showcase catalog should not overflow the stack");
    }
}
