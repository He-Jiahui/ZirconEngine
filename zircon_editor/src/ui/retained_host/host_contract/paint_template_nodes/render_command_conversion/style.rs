mod colors;
mod frame;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use colors::{
    parse_style_color, runtime_foreground_color,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use frame::frame_from_ui;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::aligned_text_x;
