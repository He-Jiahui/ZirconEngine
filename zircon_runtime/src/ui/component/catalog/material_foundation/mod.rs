use std::sync::OnceLock;

use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::UiComponentDescriptor;

mod data_display;
mod feedback;
mod inputs;
mod layout;
mod mui_x;
mod navigation;
mod shared;
mod surfaces;

static MATERIAL_EDITOR_FOUNDATION_REGISTRY: OnceLock<UiComponentDescriptorRegistry> =
    OnceLock::new();

impl UiComponentDescriptorRegistry {
    /// Builds the component catalog for the Material Dark editor foundation.
    pub fn material_editor_foundation() -> Self {
        MATERIAL_EDITOR_FOUNDATION_REGISTRY
            .get_or_init(build_material_editor_foundation_registry)
            .clone()
    }
}

fn build_material_editor_foundation_registry() -> UiComponentDescriptorRegistry {
    let mut registry = UiComponentDescriptorRegistry::new();
    for descriptor in material_editor_foundation_descriptors() {
        registry
            .register(descriptor)
            .expect("Material editor foundation descriptors must validate");
    }
    registry
}

fn material_editor_foundation_descriptors() -> Vec<UiComponentDescriptor> {
    let mut descriptors = Vec::new();
    descriptors.extend(inputs::descriptors());
    descriptors.extend(data_display::descriptors());
    descriptors.extend(feedback::descriptors());
    descriptors.extend(surfaces::descriptors());
    descriptors.extend(navigation::descriptors());
    descriptors.extend(layout::descriptors());
    descriptors.extend(mui_x::descriptors());
    descriptors
}
