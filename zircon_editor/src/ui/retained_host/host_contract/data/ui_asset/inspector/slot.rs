use crate::ui::retained_host::primitives::SharedString;

use super::semantic::UiAssetInspectorSemanticData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetInspectorSlotData {
    pub padding: SharedString,
    pub width_preferred: SharedString,
    pub height_preferred: SharedString,
    pub semantic: UiAssetInspectorSemanticData,
    pub kind: SharedString,
    pub linear_main_weight: SharedString,
    pub linear_main_stretch: SharedString,
    pub linear_cross_weight: SharedString,
    pub linear_cross_stretch: SharedString,
    pub overlay_anchor_x: SharedString,
    pub overlay_anchor_y: SharedString,
    pub overlay_pivot_x: SharedString,
    pub overlay_pivot_y: SharedString,
    pub overlay_position_x: SharedString,
    pub overlay_position_y: SharedString,
    pub overlay_z_index: SharedString,
    pub grid_row: SharedString,
    pub grid_column: SharedString,
    pub grid_row_span: SharedString,
    pub grid_column_span: SharedString,
    pub flow_break_before: SharedString,
    pub flow_alignment: SharedString,
}
