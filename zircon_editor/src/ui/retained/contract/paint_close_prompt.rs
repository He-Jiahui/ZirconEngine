#[path = "paint_close_prompt/button.rs"]
pub(in crate::ui::retained_host::host_contract) mod button;
#[path = "paint_close_prompt/colors.rs"]
pub(in crate::ui::retained_host::host_contract) mod colors;
#[path = "paint_close_prompt/draw.rs"]
mod draw;
#[path = "paint_close_prompt/layout.rs"]
mod layout;

pub(in crate::ui::retained_host::host_contract) use draw::draw_close_prompt;
