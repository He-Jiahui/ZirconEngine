mod palette;
mod radius;
mod surface;
mod text;
mod tokens;
mod variants;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use radius::alert_corner_radius;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use surface::{
    alert_background_color, alert_border_color, alert_border_width,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::{
    alert_action_color, alert_icon_color, alert_icon_cutout_color, alert_text_color,
};
