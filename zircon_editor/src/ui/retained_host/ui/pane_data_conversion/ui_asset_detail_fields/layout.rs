use crate::ui::asset_editor;

use super::row_model::{push_detail_row, semantic_label, UiAssetDetailFieldRow};

const LAYOUT_DETAIL_ROW_MAX_COUNT: usize = 10;

pub(super) fn layout_detail_rows(
    data: &asset_editor::UiAssetEditorPanePresentation,
) -> Vec<UiAssetDetailFieldRow> {
    let mut rows = Vec::with_capacity(layout_detail_row_capacity(data));
    push_detail_row(
        &mut rows,
        "Width preferred",
        &data.inspector_layout_width_preferred,
        "layout.width.preferred.set",
        "UiAssetLayoutFieldWidthPreferred",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Height preferred",
        &data.inspector_layout_height_preferred,
        "layout.height.preferred.set",
        "UiAssetLayoutFieldHeightPreferred",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        semantic_label("Semantic", &data.inspector_layout_semantic_path).as_str(),
        &data.inspector_layout_semantic_value,
        "layout.semantic.value.set",
        "UiAssetLayoutFieldSemanticValue",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Box gap",
        &data.inspector_layout_box_gap,
        "layout.box.gap.set",
        "UiAssetLayoutFieldBoxGap",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Scroll axis",
        &data.inspector_layout_scroll_axis,
        "layout.scroll.axis.set",
        "UiAssetLayoutFieldScrollAxis",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Scroll gap",
        &data.inspector_layout_scroll_gap,
        "layout.scroll.gap.set",
        "UiAssetLayoutFieldScrollGap",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Scrollbar",
        &data.inspector_layout_scrollbar_visibility,
        "layout.scroll.scrollbar_visibility.set",
        "UiAssetLayoutFieldScrollbarVisibility",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Virtual item extent",
        &data.inspector_layout_virtualization_item_extent,
        "layout.scroll.virtualization.item_extent.set",
        "UiAssetLayoutFieldVirtualItemExtent",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Virtual overscan",
        &data.inspector_layout_virtualization_overscan,
        "layout.scroll.virtualization.overscan.set",
        "UiAssetLayoutFieldVirtualOverscan",
        false,
        false,
    );
    push_detail_row(
        &mut rows,
        "Clip",
        &data.inspector_layout_clip,
        "layout.scroll.clip.set",
        "UiAssetLayoutFieldClip",
        false,
        false,
    );
    rows
}

fn layout_detail_row_capacity(data: &asset_editor::UiAssetEditorPanePresentation) -> usize {
    let capacity = [
        &data.inspector_layout_width_preferred,
        &data.inspector_layout_height_preferred,
        &data.inspector_layout_semantic_value,
        &data.inspector_layout_box_gap,
        &data.inspector_layout_scroll_axis,
        &data.inspector_layout_scroll_gap,
        &data.inspector_layout_scrollbar_visibility,
        &data.inspector_layout_virtualization_item_extent,
        &data.inspector_layout_virtualization_overscan,
        &data.inspector_layout_clip,
    ]
    .into_iter()
    .filter(|value| !value.is_empty())
    .count();
    debug_assert!(capacity <= LAYOUT_DETAIL_ROW_MAX_COUNT);
    capacity
}

#[cfg(test)]
#[path = "layout/capacity_tests.rs"]
mod capacity_tests;
