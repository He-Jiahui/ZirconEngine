mod commands;
mod content;
mod geometry;
mod identity;
mod layers;
mod metrics;
mod style;
mod surface;
mod surface_indicator;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_button_commands;

#[cfg(test)]
use super::template_button_glyphs::button_icon_size_from_host;
#[cfg(test)]
use content::{button_content_metrics_from_host, button_glyph};
#[cfg(test)]
use geometry::button_radius;
#[cfg(test)]
use geometry::{add_component_button_offset_y_from_host, button_paint_rect};
#[cfg(test)]
use identity::{button_kind, is_workbench_button};
#[cfg(test)]
use metrics::button_geometry_metrics_from_host;
#[cfg(test)]
use style::button_opacity;
#[cfg(test)]
use style::button_style;

#[cfg(test)]
#[path = "template_buttons_tests/mod.rs"]
mod tests;
