mod commands;
mod fields;
mod identity;
mod labels;
mod layout;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_property_row_text_commands;

#[cfg(test)]
use super::super::data::{FrameRect, TemplatePaneNodeData};
#[cfg(test)]
use super::template_property_axis_values::{property_axis_values, PropertyAxisValue};
#[cfg(test)]
use identity::{is_property_row, MESH_PROPERTY_ROW};
#[cfg(test)]
use layout::{property_label_width, COMPONENT_PROPERTY_LABEL_WIDTH};

#[cfg(test)]
#[path = "template_property_rows_tests.rs"]
mod tests;
