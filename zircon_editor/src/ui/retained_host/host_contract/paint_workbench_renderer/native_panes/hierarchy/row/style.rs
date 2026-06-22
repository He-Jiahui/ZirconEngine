use crate::ui::retained_host::host_contract::data::{HostPaneInteractionStateData, SceneNodeData};
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

const HIERARCHY_ROW: [u8; 4] = PALETTE.surface;
const HIERARCHY_ROW_HOVERED: [u8; 4] = PALETTE.surface_hover;
const HIERARCHY_ROW_SELECTED: [u8; 4] = PALETTE.surface_selected;

pub(super) fn hierarchy_row_color(
    index: usize,
    node: &SceneNodeData,
    interaction: &HostPaneInteractionStateData,
) -> [u8; 4] {
    if interaction.hovered_hierarchy_index == index as i32 {
        HIERARCHY_ROW_HOVERED
    } else if node.selected {
        HIERARCHY_ROW_SELECTED
    } else {
        HIERARCHY_ROW
    }
}
