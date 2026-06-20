mod checkbox;
mod commands;
mod identity;
mod labels;
mod radio;
mod style;
mod toggle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_selection_control_commands;

#[cfg(test)]
use super::super::paint_theme::PALETTE;
#[cfg(test)]
use super::style_selector::{
    WORKBENCH_CHECKBOX_CHECKED_FILL as CHECKBOX_CHECKED_FILL,
    WORKBENCH_RADIO_CHECKED_BORDER as RADIO_CHECKED_BORDER,
    WORKBENCH_RADIO_CHECKED_FILL as RADIO_CHECKED_FILL,
    WORKBENCH_SELECTION_LABEL_MUTED as SELECTION_LABEL_MUTED,
    WORKBENCH_SELECTION_MARK_IDLE_BORDER as SELECTION_MARK_IDLE_BORDER,
    WORKBENCH_SELECTION_MARK_IDLE_FILL as SELECTION_MARK_IDLE_FILL,
};
#[cfg(test)]
use super::template_selection_control_geometry::{
    centered_square, label_rect_after_mark, leading_mark_rect, radio_dot_size, selection_label_gap,
    toggle_thumb_rect, toggle_track_rect, RADIO_DOT_SIZE, TOGGLE_TRACK_WIDTH,
};
#[cfg(test)]
use style::{
    checkbox_background, checkbox_border_color, control_accent_color, control_border_color,
    radio_background, radio_border_color, selection_mark_label_color, selection_text_color,
    selection_visual_state, selection_visual_unavailable, toggle_thumb_color, toggle_track_color,
};
#[cfg(test)]
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[cfg(test)]
#[path = "template_selection_controls_tests.rs"]
mod tests;
