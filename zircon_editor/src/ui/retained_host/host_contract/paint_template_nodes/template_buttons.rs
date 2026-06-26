mod commands;
mod content;
mod geometry;
mod identity;
mod style;
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_button_commands;

#[cfg(test)]
use content::button_glyph;
#[cfg(test)]
use geometry::button_paint_rect;
#[cfg(test)]
use geometry::button_radius;
#[cfg(test)]
use identity::{button_kind, is_workbench_button};
#[cfg(test)]
use style::button_opacity;
#[cfg(test)]
use style::button_style;

#[cfg(test)]
#[path = "template_buttons_tests/mod.rs"]
mod tests;
