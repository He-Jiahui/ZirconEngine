use std::sync::OnceLock;

use zircon_runtime_interface::ui::{
    design_tokens::EditorDesignTokens, layout::UiFrame, tree::UiTemplateNodeMetadata,
};

use super::metadata::{ChromeKind, metric_attribute, string_attribute};

#[derive(Clone, Copy)]
pub(super) struct ChromeMetrics {
    pub(super) text_inset_left: f32,
    pub(super) text_inset_right: f32,
    pub(super) text_inset_y: f32,
    pub(super) icon_size: f32,
    pub(super) icon_gap: f32,
    pub(super) separator_thickness: f32,
    pub(super) font_size: f32,
    pub(super) line_height: f32,
}

impl ChromeMetrics {
    pub(super) fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let default_metrics = default_chrome_metrics();
        let default_line_height_ratio = default_metrics.line_height / default_metrics.font_size;
        let mut metrics = default_metrics;
        metrics.text_inset_left = metric_attribute(metadata, "layout_padding_left")
            .or_else(|| metric_attribute(metadata, "text_inset_left"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(metrics.text_inset_left);
        metrics.text_inset_right = metric_attribute(metadata, "layout_padding_right")
            .or_else(|| metric_attribute(metadata, "text_inset_right"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(metrics.text_inset_right);
        metrics.text_inset_y = metric_attribute(metadata, "layout_padding_vertical")
            .or_else(|| metric_attribute(metadata, "text_inset_y"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(metrics.text_inset_y);
        metrics.icon_size = metric_attribute(metadata, "layout_icon_size")
            .or_else(|| metric_attribute(metadata, "icon_size"))
            .filter(|value| *value > 0.0)
            .unwrap_or(metrics.icon_size);
        metrics.icon_gap = metric_attribute(metadata, "layout_spacing")
            .or_else(|| metric_attribute(metadata, "icon_gap"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(metrics.icon_gap);
        metrics.separator_thickness = metric_attribute(metadata, "separator_thickness")
            .or_else(|| metric_attribute(metadata, "border_width"))
            .filter(|value| *value > 0.0)
            .unwrap_or(metrics.separator_thickness);
        metrics.font_size = metric_attribute(metadata, "font_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(metrics.font_size);
        metrics.line_height = metric_attribute(metadata, "line_height")
            .filter(|value| *value > 0.0)
            .or_else(|| {
                metric_attribute(metadata, "line_height_ratio")
                    .filter(|value| *value > 0.0)
                    .map(|ratio| metrics.font_size * ratio)
            })
            .unwrap_or(metrics.font_size * default_line_height_ratio);
        metrics
    }
}

fn default_chrome_metrics() -> ChromeMetrics {
    static METRICS: OnceLock<ChromeMetrics> = OnceLock::new();
    *METRICS.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        ChromeMetrics {
            text_inset_left: (density.gap_large - controls.border_width * 2.0).max(0.0),
            text_inset_right: (density.gap_large - controls.border_width * 2.0).max(0.0),
            text_inset_y: (density.gap_medium - controls.border_width).max(0.0),
            icon_size: (controls.dense_height - density.gap_large).max(controls.border_width),
            icon_gap: (density.gap_medium - controls.border_width * 2.0).max(0.0),
            separator_thickness: controls.border_width,
            font_size: typography.body_size,
            line_height: typography.body_size * typography.line_height,
        }
    })
}

pub(super) fn separator_edge(
    metadata: &UiTemplateNodeMetadata,
    kind: ChromeKind,
) -> Option<SeparatorEdge> {
    string_attribute(metadata, "separator_edge")
        .and_then(parse_separator_edge)
        .or_else(|| match kind {
            ChromeKind::Toolbar => Some(SeparatorEdge::Bottom),
            ChromeKind::ActivityRail => Some(SeparatorEdge::Right),
            ChromeKind::StatusBar => Some(SeparatorEdge::Top),
            _ => None,
        })
}

#[derive(Clone, Copy)]
pub(super) enum SeparatorEdge {
    Top,
    Right,
    Bottom,
    Left,
}

fn parse_separator_edge(value: &str) -> Option<SeparatorEdge> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("top") {
        Some(SeparatorEdge::Top)
    } else if value.eq_ignore_ascii_case("right") {
        Some(SeparatorEdge::Right)
    } else if value.eq_ignore_ascii_case("bottom") {
        Some(SeparatorEdge::Bottom)
    } else if value.eq_ignore_ascii_case("left") {
        Some(SeparatorEdge::Left)
    } else {
        None
    }
}

pub(super) fn separator_frame(frame: UiFrame, edge: SeparatorEdge, thickness: f32) -> UiFrame {
    match edge {
        SeparatorEdge::Top => UiFrame::new(frame.x, frame.y, frame.width, thickness),
        SeparatorEdge::Right => UiFrame::new(
            frame.x + (frame.width - thickness).max(0.0),
            frame.y,
            thickness,
            frame.height,
        ),
        SeparatorEdge::Bottom => UiFrame::new(
            frame.x,
            frame.y + (frame.height - thickness).max(0.0),
            frame.width,
            thickness,
        ),
        SeparatorEdge::Left => UiFrame::new(frame.x, frame.y, thickness, frame.height),
    }
}

pub(super) fn text_frame(frame: UiFrame, has_icon: bool, metrics: ChromeMetrics) -> UiFrame {
    let icon_offset = if has_icon {
        metrics.icon_size + metrics.icon_gap
    } else {
        0.0
    };
    UiFrame::new(
        frame.x + metrics.text_inset_left + icon_offset,
        frame.y + metrics.text_inset_y,
        (frame.width - metrics.text_inset_left - metrics.text_inset_right - icon_offset).max(1.0),
        (frame.height - metrics.text_inset_y * 2.0).max(metrics.line_height),
    )
}

pub(super) fn icon_frame(frame: UiFrame, label_follows: bool, metrics: ChromeMetrics) -> UiFrame {
    let x = if label_follows {
        frame.x + metrics.text_inset_left
    } else {
        frame.x + (frame.width - metrics.icon_size) * 0.5
    };
    UiFrame::new(
        x,
        frame.y + (frame.height - metrics.icon_size) * 0.5,
        metrics.icon_size,
        metrics.icon_size,
    )
}
