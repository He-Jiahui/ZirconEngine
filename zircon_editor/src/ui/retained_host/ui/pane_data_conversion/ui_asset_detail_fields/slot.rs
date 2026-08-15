use crate::ui::asset_editor;

use super::row_model::{push_detail_row, semantic_label, UiAssetDetailFieldRow};

pub(super) fn slot_detail_rows(
    data: &asset_editor::UiAssetEditorPanePresentation,
) -> Vec<UiAssetDetailFieldRow> {
    let mut rows = Vec::new();
    push_detail_row(
        &mut rows,
        "Mount",
        &data.inspector_mount,
        "slot.mount.set",
        "UiAssetSlotFieldMount",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Padding",
        &data.inspector_slot_padding,
        "slot.padding.set",
        "UiAssetSlotFieldPadding",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Width preferred",
        &data.inspector_slot_width_preferred,
        "slot.layout.width.preferred.set",
        "UiAssetSlotFieldWidthPreferred",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Height preferred",
        &data.inspector_slot_height_preferred,
        "slot.layout.height.preferred.set",
        "UiAssetSlotFieldHeightPreferred",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        semantic_label("Semantic", &data.inspector_slot_semantic_path).as_str(),
        &data.inspector_slot_semantic_value,
        "slot.semantic.value.set",
        "UiAssetSlotFieldSemanticValue",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Linear width weight",
        &data.inspector_slot_linear_main_weight,
        "slot.linear.width_weight.set",
        "UiAssetSlotFieldLinearWidthWeight",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Linear width stretch",
        &data.inspector_slot_linear_main_stretch,
        "slot.linear.width_stretch.set",
        "UiAssetSlotFieldLinearWidthStretch",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Linear height weight",
        &data.inspector_slot_linear_cross_weight,
        "slot.linear.height_weight.set",
        "UiAssetSlotFieldLinearHeightWeight",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Linear height stretch",
        &data.inspector_slot_linear_cross_stretch,
        "slot.linear.height_stretch.set",
        "UiAssetSlotFieldLinearHeightStretch",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Overlay anchor x",
        &data.inspector_slot_overlay_anchor_x,
        "slot.overlay.anchor_x.set",
        "UiAssetSlotFieldOverlayAnchorX",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Overlay anchor y",
        &data.inspector_slot_overlay_anchor_y,
        "slot.overlay.anchor_y.set",
        "UiAssetSlotFieldOverlayAnchorY",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Overlay position x",
        &data.inspector_slot_overlay_position_x,
        "slot.overlay.position_x.set",
        "UiAssetSlotFieldOverlayPositionX",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Overlay position y",
        &data.inspector_slot_overlay_position_y,
        "slot.overlay.position_y.set",
        "UiAssetSlotFieldOverlayPositionY",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Overlay z",
        &data.inspector_slot_overlay_z_index,
        "slot.overlay.z_index.set",
        "UiAssetSlotFieldOverlayZ",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Grid row",
        &data.inspector_slot_grid_row,
        "slot.grid.row.set",
        "UiAssetSlotFieldGridRow",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Grid column",
        &data.inspector_slot_grid_column,
        "slot.grid.column.set",
        "UiAssetSlotFieldGridColumn",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Flow break before",
        &data.inspector_slot_flow_break_before,
        "slot.flow.break_before.set",
        "UiAssetSlotFieldFlowBreakBefore",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Flow alignment",
        &data.inspector_slot_flow_alignment,
        "slot.flow.alignment.set",
        "UiAssetSlotFieldFlowAlignment",
        false,
        false,
    );
    rows
}
