mod connector;
mod dot;
mod tokens;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use connector::timeline_connector_color;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use dot::{
    timeline_dot_background_color, timeline_dot_border_color, timeline_dot_border_width,
    timeline_dot_is_outlined,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use tokens::timeline_dot_tone_color;
