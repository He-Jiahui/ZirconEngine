use super::super::resolved_state_for_node;
use super::model::WorkbenchSliderStyle;
use super::state::is_unavailable_slider_state;
use super::text::{slider_label_color, slider_value_text};
use super::thumb::{slider_thumb_color, slider_thumb_halo_color, slider_thumb_outline_color};
use super::track::{slider_fill_color, slider_tick_color, slider_track_color};
use super::value::{slider_range_value_border, slider_value_border, slider_value_surface};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_slider_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchSliderStyle {
    let state = resolved_state_for_node(node).slider_resolved_state();
    let unavailable = is_unavailable_slider_state(state);
    let fill = slider_fill_color(node, unavailable);

    WorkbenchSliderStyle {
        track: slider_track_color(node, unavailable),
        fill,
        thumb: slider_thumb_color(node, unavailable),
        thumb_outline: slider_thumb_outline_color(node, state, fill),
        thumb_halo: slider_thumb_halo_color(node, state),
        value_surface: slider_value_surface(unavailable),
        value_border: slider_value_border(state, fill),
        range_value_border: slider_range_value_border(state),
        label_text: slider_label_color(node, unavailable),
        value_text: slider_value_text(unavailable),
        tick: slider_tick_color(unavailable),
        state,
    }
}
