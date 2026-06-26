mod commands;
mod geometry;
mod identity;
mod style;
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_icon_button_commands;

#[cfg(test)]
#[path = "template_icon_buttons_tests/mod.rs"]
mod tests;

#[cfg(test)]
use geometry::{icon_button_paint_rect, icon_glyph_rect};
#[cfg(test)]
use identity::is_workbench_icon_button;
#[cfg(test)]
use style::{icon_button_context, icon_button_style, IconButtonContext};
