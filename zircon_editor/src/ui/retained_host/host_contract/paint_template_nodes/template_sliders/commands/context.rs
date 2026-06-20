use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::style_selector::WorkbenchSliderStyle;
use super::super::super::template_slider_geometry::{
    pixel_aligned_rect, slider_label, slider_percent, slider_range_min_percent, slider_tick_count,
    slider_track_rect, slider_value_rect,
};
use super::super::identity::{is_workbench_slider, slider_style};

pub(super) enum SliderCommandContext {
    NotSlider,
    Consumed,
    Ready(SliderCommandParts),
}

pub(super) struct SliderCommandParts {
    pub(super) rect: FrameRect,
    pub(super) value_rect: Option<FrameRect>,
    pub(super) track_rect: FrameRect,
    pub(super) label: Option<String>,
    pub(super) percent: f32,
    pub(super) range_min_percent: Option<f32>,
    pub(super) tick_count: Option<usize>,
    pub(super) style: WorkbenchSliderStyle,
}

pub(super) fn build_slider_command_context(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> SliderCommandContext {
    if !is_workbench_slider(node) {
        return SliderCommandContext::NotSlider;
    }

    let rect = pixel_aligned_rect(rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return SliderCommandContext::Consumed;
    }

    let label = slider_label(node);
    let has_label = label.is_some();
    let value_rect = slider_value_rect(&rect);
    let track_rect = slider_track_rect(&rect, value_rect.as_ref(), has_label, node);
    if track_rect.width <= 1.0 {
        return SliderCommandContext::Consumed;
    }

    SliderCommandContext::Ready(SliderCommandParts {
        rect,
        value_rect,
        track_rect,
        label,
        percent: slider_percent(node),
        range_min_percent: slider_range_min_percent(node),
        tick_count: slider_tick_count(node),
        style: slider_style(node),
    })
}
