#[path = "../tree_view/content.rs"]
mod content;
#[path = "../tree_view/identity.rs"]
mod identity;
#[path = "tree_view/rows.rs"]
mod rows;
#[path = "tree_view/style.rs"]
mod style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::is_tree_view;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use rows::push_tree_view;
