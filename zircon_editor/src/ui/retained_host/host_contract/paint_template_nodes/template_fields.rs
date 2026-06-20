mod commands;
mod geometry;
mod identity;
mod style;
mod surface;
mod text;

#[cfg(test)]
#[path = "template_fields_tests.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_field_commands;

#[cfg(test)]
use super::super::data::{FrameRect, TemplatePaneNodeData};
#[cfg(test)]
use geometry::pixel_aligned_rect;
#[cfg(test)]
use identity::is_workbench_field;
#[cfg(test)]
use style::{field_opacity, field_style};
