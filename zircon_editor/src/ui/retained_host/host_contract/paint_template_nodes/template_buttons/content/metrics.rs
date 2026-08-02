use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::style_selector::{
    WorkbenchButtonKind, is_asset_browser_toolbar_chip_button, is_asset_browser_utility_tab_button,
    is_compact_icon_text_workbench_button,
};
use crate::ui::retained_host::host_contract::paint_text::measure_runtime_text_width_with_style;
use crate::ui::retained_host::host_contract::paint_theme::{
    HostControlMetrics, HostTextPreferences, current_host_metrics, current_host_text_preferences,
};
use zircon_runtime_interface::ui::style::ButtonInteractionState;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const BUTTON_LABEL_STRONG_FONT_WEIGHT: i32 = 600;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes::template_buttons) struct WorkbenchButtonContentMetrics
{
    pub icon_gap: f32,
    pub chevron_reserve: f32,
    pub trailing_glyph_inset: f32,
    pub font_size: f32,
    pub text_clip_guard: f32,
    pub utility_tab_pad_x: f32,
    pub toolbar_chip_pad_x: f32,
    pub compact_icon_text_font_size: f32,
    pub compact_icon_text_pad_x: f32,
    pub compact_icon_text_gap: f32,
    pub button_pad_x: f32,
    pub pressed_offset_y: f32,
    line_height_ratio: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::template_buttons) fn button_content_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchButtonContentMetrics {
    WorkbenchButtonContentMetrics {
        icon_gap: metrics.button_icon_gap,
        chevron_reserve: metrics.button_chevron_reserve,
        trailing_glyph_inset: metrics.button_pad_x,
        font_size: metrics.font_body,
        text_clip_guard: metrics.text_clip_guard,
        utility_tab_pad_x: metrics.gap_s,
        toolbar_chip_pad_x: metrics.gap_m,
        compact_icon_text_font_size: metrics.font_small,
        compact_icon_text_pad_x: metrics.gap_s,
        compact_icon_text_gap: metrics.gap_s,
        button_pad_x: metrics.button_pad_x,
        pressed_offset_y: metrics.button_pressed_offset_y,
        line_height_ratio: metrics.line_height_ratio,
    }
}

fn button_content_metrics() -> WorkbenchButtonContentMetrics {
    button_content_metrics_from_host(current_host_metrics())
}

pub(super) fn button_label_font_size(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let metrics = button_content_metrics();
    button_label_font_size_from_metrics(node, rect, metrics)
}

fn button_label_font_size_from_metrics(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: WorkbenchButtonContentMetrics,
) -> f32 {
    if !rect.height.is_finite() || rect.height <= 0.0 {
        return 0.0;
    }
    if is_compact_icon_text_workbench_button(node) {
        metrics.compact_icon_text_font_size.min(rect.height)
    } else if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size.min(rect.height)
    } else {
        metrics.font_size.min(rect.height)
    }
}

pub(super) fn button_label_line_height(font_size: f32) -> f32 {
    if !font_size.is_finite() || font_size <= 0.0 {
        return 0.0;
    }
    button_content_metrics()
        .line_height(font_size)
        .round()
        .max(font_size.ceil())
}

pub(super) fn button_label_paint_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchButtonKind,
) -> UiTextRunPaintStyle {
    let preferences = current_host_text_preferences();
    button_label_paint_style_with_preferences(node, kind, &preferences)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_label_paint_style_with_preferences(
    node: &TemplatePaneNodeData,
    kind: WorkbenchButtonKind,
    preferences: &HostTextPreferences,
) -> UiTextRunPaintStyle {
    let mut style = UiTextRunPaintStyle {
        strong: kind == WorkbenchButtonKind::Primary
            || node.font_weight >= BUTTON_LABEL_STRONG_FONT_WEIGHT,
        ..UiTextRunPaintStyle::default()
    };
    if is_asset_browser_utility_tab_button(node) && preferences.utility_tab_uses_code_text() {
        style.code = true;
    }
    style
}

pub(super) fn measured_label_ink_width(
    label: &str,
    font_size: f32,
    text_style: UiTextRunPaintStyle,
) -> f32 {
    measure_runtime_text_width_with_style(label, font_size, text_style)
}

pub(super) fn label_text_slot_width(ink_width: f32, max_width: f32) -> f32 {
    (ink_width.max(0.0) + button_content_metrics().text_clip_guard).min(max_width.max(0.0))
}

pub(super) fn max_label_slot_width(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let metrics = button_content_metrics();
    let pad_x = if is_asset_browser_utility_tab_button(node) {
        metrics.utility_tab_pad_x
    } else if is_asset_browser_toolbar_chip_button(node) {
        metrics.toolbar_chip_pad_x
    } else if is_compact_icon_text_workbench_button(node) {
        metrics.compact_icon_text_pad_x
    } else {
        metrics.button_pad_x
    };
    (rect.width - pad_x * 2.0).max(0.0)
}

pub(super) fn button_icon_gap(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_content_offset_x.is_finite() && node.layout_content_offset_x > 0.0 {
        node.layout_content_offset_x
    } else if is_compact_icon_text_workbench_button(node) {
        button_content_metrics().compact_icon_text_gap
    } else {
        button_content_metrics().icon_gap
    }
}

pub(super) fn button_chevron_reserve() -> f32 {
    button_content_metrics().chevron_reserve
}

pub(super) fn trailing_glyph_inset() -> f32 {
    button_content_metrics().trailing_glyph_inset
}

pub(super) fn content_offset_y(interaction: ButtonInteractionState) -> f32 {
    let metrics = button_content_metrics();
    match interaction {
        ButtonInteractionState::Pressed => metrics.pressed_offset_y,
        _ => 0.0,
    }
}

impl WorkbenchButtonContentMetrics {
    fn line_height(self, font_size: f32) -> f32 {
        font_size * self.line_height_ratio
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn compact_icon_text_uses_shared_caption_size_instead_of_instance_font_size() {
        let node = TemplatePaneNodeData {
            component_variant: "code compact_icon_text".into(),
            font_size: 24.0,
            ..TemplatePaneNodeData::default()
        };
        let rect = FrameRect {
            width: 54.0,
            height: 28.0,
            ..FrameRect::default()
        };

        let metrics = button_content_metrics_from_host(METRICS);
        assert_eq!(
            button_label_font_size_from_metrics(&node, &rect, metrics),
            METRICS.font_small
        );
    }
}
