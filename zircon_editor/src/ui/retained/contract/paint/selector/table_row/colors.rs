#[path = "colors/action.rs"]
mod action;
#[path = "../colors/background.rs"]
mod background;
#[path = "colors/border.rs"]
mod border;
#[path = "../colors/declared.rs"]
mod declared;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use action::table_row_action_color;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use background::table_row_background;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use border::{
    table_row_border, table_row_border_width,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use declared::declared_value_color;
