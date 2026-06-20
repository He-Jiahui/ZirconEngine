mod actions;
mod border;
mod surface;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use actions::{
    cancel_action_color, confirm_action_color, dialog_action_color,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use border::dialog_border_color;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use surface::dialog_surface_color;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::{
    dialog_body_color, dialog_title_color,
};
