mod avatar;
mod background;
mod border;
mod delete;
mod foreground;
mod palette;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use avatar::chip_avatar_background_color;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use background::chip_background_color;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use border::{
    chip_border_color, chip_border_width,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use delete::chip_delete_icon_color;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use foreground::chip_foreground_color;
