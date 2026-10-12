use crate::ui::retained_host::host_contract::data::{HostPaneInteractionStateData, SceneNodeData};
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;

pub(super) fn hierarchy_row_color(
    index: usize,
    node: &SceneNodeData,
    interaction: &HostPaneInteractionStateData,
) -> [u8; 4] {
    let palette = current_host_palette();
    // Selection remains data-owned; only the live native input owner decides active tint.
    if node.selected {
        if interaction.focused_template_control_id == "HierarchyListPanel" {
            palette.surface_selected
        } else {
            palette.surface_selected_inactive
        }
    } else if interaction.hovered_hierarchy_index == index as i32 {
        palette.surface_panel
    } else {
        palette.surface_inset
    }
}
