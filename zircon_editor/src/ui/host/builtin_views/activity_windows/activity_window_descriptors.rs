use crate::ui::workbench::view::ViewDescriptor;

use super::animation_graph_view_descriptor::animation_graph_view_descriptor;
use super::animation_sequence_view_descriptor::animation_sequence_view_descriptor;
use super::asset_browser_view_descriptor::asset_browser_view_descriptor;
use super::prefab_view_descriptor::prefab_view_descriptor;
use super::workbench_window_view_descriptor::workbench_window_view_descriptor;

pub(in crate::ui::host::builtin_views) fn activity_window_descriptors() -> Vec<ViewDescriptor> {
    vec![
        workbench_window_view_descriptor(),
        prefab_view_descriptor(),
        asset_browser_view_descriptor(),
        animation_sequence_view_descriptor(),
        animation_graph_view_descriptor(),
    ]
}
