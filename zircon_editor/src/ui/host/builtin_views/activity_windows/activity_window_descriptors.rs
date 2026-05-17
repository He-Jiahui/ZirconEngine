use crate::ui::workbench::view::ViewDescriptor;

use super::animation_graph_view_descriptor::animation_graph_view_descriptor;
use super::animation_sequence_view_descriptor::animation_sequence_view_descriptor;
use super::asset_browser_view_descriptor::asset_browser_view_descriptor;
use super::component_showcase_view_descriptor::component_showcase_view_descriptor;
use super::debug_observatory_view_descriptor::debug_observatory_view_descriptor;
use super::functional_window_view_descriptors::functional_window_view_descriptors;
use super::material_component_lab_view_descriptor::material_component_lab_view_descriptor;
use super::material_demo_view_descriptor::material_demo_view_descriptor;
use super::prefab_view_descriptor::prefab_view_descriptor;
use super::workbench_window_view_descriptor::workbench_window_view_descriptor;

pub(in crate::ui::host::builtin_views) fn activity_window_descriptors() -> Vec<ViewDescriptor> {
    let mut descriptors = vec![
        workbench_window_view_descriptor(),
        prefab_view_descriptor(),
        asset_browser_view_descriptor(),
        component_showcase_view_descriptor(),
        material_demo_view_descriptor(),
        material_component_lab_view_descriptor(),
        debug_observatory_view_descriptor(),
        animation_sequence_view_descriptor(),
        animation_graph_view_descriptor(),
    ];
    descriptors.extend(functional_window_view_descriptors());
    descriptors
}
