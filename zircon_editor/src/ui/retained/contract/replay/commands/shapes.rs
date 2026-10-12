#[path = "shapes/border.rs"]
mod border;
#[path = "shapes/quad.rs"]
mod quad;

pub(super) use border::paint_border_command;
pub(super) use quad::{paint_quad_command, paint_rounded_box_commands};
