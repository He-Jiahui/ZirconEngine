mod check;
mod chevron;
mod plus;
mod trash;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use check::push_check_adornment;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use chevron::push_chevron_adornment;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use plus::push_plus_adornment;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use trash::push_trash_adornment;
