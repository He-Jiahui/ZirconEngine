mod horizontal;
mod vertical;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use horizontal::{
    push_bottom_line, push_top_line,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use vertical::{
    push_left_line, push_right_line, push_vertical_line,
};
