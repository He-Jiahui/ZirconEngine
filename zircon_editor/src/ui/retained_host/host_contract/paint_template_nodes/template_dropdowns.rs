mod commands;
mod geometry;
mod identity;
mod style;
mod surface;
mod text;

#[cfg(test)]
#[path = "template_dropdowns_tests.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_dropdown_commands;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::dropdown_paint_rect;

#[cfg(test)]
use super::super::data::{FrameRect, TemplatePaneNodeData};
#[cfg(test)]
use geometry::pixel_aligned_rect;
#[cfg(test)]
use identity::is_workbench_dropdown;
#[cfg(test)]
use style::dropdown_style;
