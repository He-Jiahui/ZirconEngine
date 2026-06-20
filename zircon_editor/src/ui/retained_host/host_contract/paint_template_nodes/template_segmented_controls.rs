mod commands;
mod identity;
mod labels;
mod options;
mod segments;
mod style;
mod tabs;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_segmented_control_commands;

#[cfg(test)]
use super::super::data::{FrameRect, TemplatePaneNodeData};
#[cfg(test)]
use super::super::paint_theme::PALETTE;
#[cfg(test)]
use super::template_segmented_control_geometry::{
    segment_rect, segmented_body_rect, tab_paint_rect,
};
#[cfg(test)]
use identity::is_workbench_tab;
#[cfg(test)]
use options::{segmented_options, selected_segment_value};
#[cfg(test)]
use style::{
    segmented_background, segmented_control_style, selected_segment_border_width,
    selected_segment_underline_color, selected_segment_underline_height, tab_background, tab_style,
    tab_text_color, SEGMENT_IDLE_BACKGROUND,
};

#[cfg(test)]
#[path = "template_segmented_controls_tests.rs"]
mod tests;
