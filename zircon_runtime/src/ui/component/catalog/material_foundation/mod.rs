use std::sync::OnceLock;

use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::UiComponentDescriptor;

mod button_inputs;
mod data_display;
mod data_display_editor;
mod data_display_subcomponents;
mod data_display_table;
mod feedback;
mod form_controls;
mod inputs;
mod lab_subcomponents;
mod layout;
mod layout_editor;
mod layout_mui;
mod layout_transitions;
mod layout_utilities;
mod mui_x;
mod navigation;
mod navigation_editor;
mod navigation_secondary;
mod navigation_subcomponents;
mod selection_inputs;
mod shared;
mod surface_subcomponents;
mod surfaces;
mod text_inputs;

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
    descriptors.extend(button_inputs::descriptors());
    descriptors.extend(inputs::descriptors());
    descriptors.extend(selection_inputs::descriptors());
    descriptors.extend(text_inputs::descriptors());
    descriptors.extend(form_controls::descriptors());
    descriptors.extend(data_display::descriptors());
    descriptors.extend(data_display_editor::descriptors());
    descriptors.extend(data_display_subcomponents::descriptors());
    descriptors.extend(data_display_table::descriptors());
    descriptors.extend(feedback::descriptors());
    descriptors.extend(surface_subcomponents::descriptors());
    descriptors.extend(surfaces::descriptors());
    descriptors.extend(navigation::descriptors());
    descriptors.extend(navigation_subcomponents::descriptors());
    descriptors.extend(navigation_secondary::descriptors());
    descriptors.extend(navigation_editor::descriptors());
    descriptors.extend(layout_mui::descriptors());
    descriptors.extend(layout::descriptors());
    descriptors.extend(layout_utilities::descriptors());
    descriptors.extend(layout_transitions::descriptors());
    descriptors.extend(layout_editor::descriptors());
    descriptors.extend(mui_x::descriptors());
    descriptors.extend(lab_subcomponents::descriptors());
    descriptors
}
