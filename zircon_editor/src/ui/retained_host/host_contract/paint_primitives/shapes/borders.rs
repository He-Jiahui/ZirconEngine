mod rect;
mod rounded;

pub(in crate::ui::retained_host::host_contract) use rect::{draw_border, draw_border_clipped};
pub(in crate::ui::retained_host::host_contract) use rounded::{
    draw_rounded_border_clipped, draw_rounded_box_clipped,
};
