#[path = "draw/border.rs"]
mod border;
#[path = "draw/color.rs"]
mod color;
#[path = "draw/dispatch.rs"]
mod dispatch;
#[path = "draw/image.rs"]
mod image;
#[path = "draw/ordering.rs"]
mod ordering;
#[path = "draw/quad.rs"]
mod quad;
#[path = "draw/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use ordering::draw_host_paint_commands;
