mod borders;
mod rects;

pub(in crate::ui::retained_host::host_contract) use borders::{
    draw_border, draw_border_clipped, draw_rounded_border_clipped,
};
pub(in crate::ui::retained_host::host_contract) use rects::{
    draw_rect, draw_rect_clipped, draw_rounded_rect_clipped,
};
