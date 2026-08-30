use std::sync::OnceLock;

use crate::ui::component::UiComponentDescriptorRegistry;
use zircon_runtime_interface::ui::component::UiComponentDescriptor;

mod button_inputs;
mod data_display;
mod data_display_editor;
mod data_display_subcomponents;
mod data_display_table;
mod data_display_visuals;
mod feedback;
mod feedback_editor_overlays;
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

#[cfg(test)]
mod capacity_tests;

static MATERIAL_EDITOR_FOUNDATION_REGISTRY: OnceLock<UiComponentDescriptorRegistry> =
    OnceLock::new();

const MATERIAL_FOUNDATION_DESCRIPTOR_GROUP_COUNT: usize = 25;

impl UiComponentDescriptorRegistry {
    /// Builds the component catalog for the Material Dark editor foundation.
    pub fn material_editor_foundation() -> Self {
        Self::material_editor_foundation_shared().clone()
    }

    /// Returns the process-wide read-only Material editor foundation catalog.
    pub fn material_editor_foundation_shared() -> &'static Self {
        MATERIAL_EDITOR_FOUNDATION_REGISTRY.get_or_init(build_material_editor_foundation_registry)
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
    let descriptor_groups: [Vec<UiComponentDescriptor>;
        MATERIAL_FOUNDATION_DESCRIPTOR_GROUP_COUNT] = [
        button_inputs::descriptors(),
        inputs::descriptors(),
        selection_inputs::descriptors(),
        text_inputs::descriptors(),
        form_controls::descriptors(),
        data_display::descriptors(),
        data_display_editor::descriptors(),
        data_display_subcomponents::descriptors(),
        data_display_table::descriptors(),
        data_display_visuals::descriptors(),
        feedback::descriptors(),
        feedback_editor_overlays::descriptors(),
        surface_subcomponents::descriptors(),
        surfaces::descriptors(),
        navigation::descriptors(),
        navigation_subcomponents::descriptors(),
        navigation_secondary::descriptors(),
        navigation_editor::descriptors(),
        layout_mui::descriptors(),
        layout::descriptors(),
        layout_utilities::descriptors(),
        layout_transitions::descriptors(),
        layout_editor::descriptors(),
        mui_x::descriptors(),
        lab_subcomponents::descriptors(),
    ];
    let descriptor_capacity = descriptor_groups.iter().fold(0usize, |capacity, group| {
        capacity.saturating_add(group.len())
    });
    let mut descriptors = Vec::with_capacity(descriptor_capacity);
    for group in descriptor_groups {
        descriptors.extend(group);
    }
    descriptors
}
