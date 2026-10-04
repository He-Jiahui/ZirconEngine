//! Inspector 资源、阴影行采用当前宿主密度的统一快照；常量只描述默认 Slate 基线，不代替动态指标。
//! 所有几何和 glyph 入口应在同一次绘制中遵循这套列宽及间距约定。

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_ROW_TEXT_Y:
    f32 = 5.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_LABEL_WIDTH: f32 = 104.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_COUNT_WIDTH: f32 = 24.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_FIELD_TEXT_X: f32 = 8.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_FIELD_RIGHT_PAD: f32 = 22.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_CHEVRON_SIZE: f32 = 10.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_NESTED_LABEL_WIDTH: f32 = 116.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_NESTED_LABEL_BASE_X: f32 = 6.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_NESTED_LABEL_OFFSET_X: f32 = 8.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_NESTED_SELECT_OFFSET_X: f32 = 14.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_FIELD_INSET_Y: f32 = 3.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_ICON_SIZE:
    f32 = 13.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_CHEVRON_RIGHT_PAD: f32 = 5.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_CHECK_SIZE:
    f32 = 14.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const INSPECTOR_SHADOW_CHECK_DEFAULT_CONTENT_OFFSET_X: f32 = INSPECTOR_COUNT_WIDTH + 4.0;

// Inspector rows keep the familiar Slate baseline at the default token values,
// while every derived dimension follows the active retained-host density.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct InspectorRowMetrics {
    pub border_width: f32,
    pub gap_s: f32,
    pub row_text_y: f32,
    pub label_width: f32,
    pub count_width: f32,
    pub field_text_x: f32,
    pub icon_text_gap: f32,
    pub field_right_pad: f32,
    pub chevron_size: f32,
    pub nested_label_width: f32,
    pub nested_label_base_x: f32,
    pub nested_label_offset_x: f32,
    pub nested_select_offset_x: f32,
    pub field_inset_y: f32,
    pub icon_size: f32,
    pub chevron_right_pad: f32,
    pub check_size: f32,
    pub shadow_check_default_content_offset_x: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn inspector_row_metrics(
) -> InspectorRowMetrics {
    inspector_row_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_paintable_rect(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

fn inspector_row_metrics_from_host(host: HostControlMetrics) -> InspectorRowMetrics {
    let border = finite_non_negative(host.border_width, 1.0);
    let row_height = finite_non_negative(host.row_height, 24.0);
    let gap_s = finite_non_negative(host.gap_s, 4.0);
    let gap_m = finite_non_negative(host.gap_m, 8.0);
    let gap_l = finite_non_negative(host.gap_l, 12.0);
    let chevron_reserve = finite_non_negative(host.button_chevron_reserve, 18.0);
    let icon_text_gap = finite_non_negative(host.button_icon_gap, 7.0);
    let input_left = finite_non_negative(host.input_pad[0], 8.0);
    let input_top = finite_non_negative(host.input_pad[2], 3.0);

    InspectorRowMetrics {
        border_width: border,
        gap_s,
        row_text_y: input_top + border * 2.0,
        label_width: row_height * 4.0 + gap_m,
        count_width: gap_l * 2.0,
        field_text_x: input_left,
        icon_text_gap,
        field_right_pad: chevron_reserve + gap_s,
        chevron_size: (gap_l - border * 2.0).max(0.0),
        nested_label_width: row_height * 4.0 + gap_l + input_left,
        nested_label_base_x: gap_s + border * 2.0,
        nested_label_offset_x: gap_m,
        nested_select_offset_x: gap_l + border * 2.0,
        field_inset_y: input_top,
        icon_size: (row_height - gap_l + border).max(0.0),
        chevron_right_pad: gap_s + border,
        check_size: gap_l + border * 2.0,
        shadow_check_default_content_offset_x: gap_l * 2.0 + gap_s,
    }
}

fn finite_non_negative(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value >= 0.0 {
        value
    } else {
        fallback
    }
}

#[cfg(test)]
#[path = "tests/metrics.rs"]
mod tests;
