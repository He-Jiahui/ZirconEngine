//! 展示目录为默认模板编译器和编写调色板提供可重复使用的元数据；需要项目修改时使用克隆副本。

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
#[path = "tests/editor_showcase.rs"]
mod tests;
