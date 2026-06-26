mod chips;
mod commands;
mod icons;
mod identity;
mod signals;

#[cfg(test)]
use super::super::paint_theme::PALETTE;
#[cfg(test)]
use super::style_selector::{
    select_workbench_status_chip_style, select_workbench_status_icon_button_style,
    WorkbenchStatusSignalKind as StatusSignalKind,
    WORKBENCH_STATUS_NO_ERRORS_FILL as STATUS_NO_ERRORS_FILL,
};
#[cfg(test)]
use super::template_status_control_geometry::status_chip_text_rect;
#[cfg(test)]
use super::template_status_control_geometry::{
    status_control_offset_rect, status_signal_icon_paint_rect, status_signal_icon_rect,
    status_signal_text_gap,
};
#[cfg(test)]
use super::template_status_glyphs::StatusIconKind;
#[cfg(test)]
use chips::status_chip_text_color;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_status_control_commands;
#[cfg(test)]
use identity::{status_control_kind, StatusControlKind};
#[cfg(test)]
use signals::{status_signal_icon_fill, status_signal_text_color};

#[cfg(test)]
#[path = "template_status_controls_tests/mod.rs"]
mod tests;
