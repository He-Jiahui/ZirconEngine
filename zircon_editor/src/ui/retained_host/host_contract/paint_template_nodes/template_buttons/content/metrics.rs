use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::style_selector::{
    is_asset_browser_toolbar_chip_button, is_asset_browser_utility_tab_button,
};
use crate::ui::retained_host::host_contract::paint_text::measure_runtime_text_width_with_style;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, current_host_text_preferences, HostTextPreferences,
};
use zircon_runtime_interface::ui::style::ButtonInteractionState;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const BUTTON_LABEL_STRONG_FONT_WEIGHT: i32 = 600;

pub(super) fn button_label_font_size(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size.min(rect.height.max(1.0))
    } else {
        current_host_metrics().font_body
    }
}

pub(super) fn button_label_line_height(font_size: f32) -> f32 {
    let font_size = font_size.max(1.0);
    current_host_metrics()
        .line_height(font_size)
        .round()
        .max(font_size.ceil())
}

pub(super) fn button_label_paint_style(node: &TemplatePaneNodeData) -> UiTextRunPaintStyle {
    let preferences = current_host_text_preferences();
    button_label_paint_style_with_preferences(node, &preferences)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_label_paint_style_with_preferences(
    node: &TemplatePaneNodeData,
    preferences: &HostTextPreferences,
) -> UiTextRunPaintStyle {
    let mut style = UiTextRunPaintStyle {
        strong: node.font_weight >= BUTTON_LABEL_STRONG_FONT_WEIGHT,
        ..UiTextRunPaintStyle::default()
    };
    if is_asset_browser_utility_tab_button(node) && preferences.utility_tab_uses_code_text() {
        style.code = true;
    }
    style
}

pub(super) fn measured_label_width(
    label: &str,
    font_size: f32,
    text_style: UiTextRunPaintStyle,
) -> f32 {
    measure_runtime_text_width_with_style(label, font_size, text_style)
        + current_host_metrics().text_clip_guard
}

pub(super) fn max_label_slot_width(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let metrics = current_host_metrics();
    let pad_x = if is_asset_browser_utility_tab_button(node) {
        metrics.gap_s
    } else if is_asset_browser_toolbar_chip_button(node) {
        metrics.gap_m
    } else {
        metrics.button_pad_x
    };
    (rect.width - pad_x * 2.0).max(1.0)
}

pub(super) fn content_offset_y(interaction: ButtonInteractionState) -> f32 {
    match interaction {
        ButtonInteractionState::Pressed => current_host_metrics().button_pressed_offset_y,
        _ => 0.0,
    }
}
