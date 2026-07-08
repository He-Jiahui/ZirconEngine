use super::super::super::layers::{label_order, value_surface_order};
use super::super::context::SliderCommandParts;
use super::label::push_sequence_label;
use super::thumbs::push_sequence_thumbs;
use super::track::push_sequence_track;
use super::values::push_sequence_values;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

pub(in super::super) fn push_ready_slider_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    context: SliderCommandParts,
) {
    let SliderCommandParts {
        rect,
        value_rect,
        track_rect,
        label,
        percent,
        range_min_percent,
        tick_count,
        style,
    } = context;

    push_sequence_label(
        commands,
        &rect,
        clip,
        label_order(order),
        label,
        &style,
        opacity,
    );
    push_sequence_track(
        commands,
        &track_rect,
        clip,
        order,
        percent,
        range_min_percent,
        tick_count,
        &style,
        opacity,
    );
    push_sequence_thumbs(
        commands,
        node,
        &track_rect,
        clip,
        order,
        percent,
        range_min_percent,
        &style,
        opacity,
    );
    push_sequence_values(
        commands,
        node,
        &rect,
        value_rect.as_ref(),
        &track_rect,
        clip,
        value_surface_order(order),
        percent,
        range_min_percent,
        &style,
        opacity,
    );
}
