mod activity_rail;
mod asset_content;
mod asset_reference;
mod asset_tree;
mod document_tab;
mod drawer_header;
mod hierarchy;
mod host_page;
mod menu;
mod viewport_toolbar;
mod welcome_recent;

pub(crate) use activity_rail::dispatch_shared_activity_rail_pointer_click;
pub(crate) use asset_content::dispatch_shared_asset_content_pointer_click;
pub(crate) use asset_reference::dispatch_shared_asset_reference_pointer_click;
pub(crate) use asset_tree::dispatch_shared_asset_tree_pointer_click;
pub(crate) use document_tab::{
    dispatch_shared_document_tab_close_pointer_click, dispatch_shared_document_tab_pointer_click,
};
pub(crate) use drawer_header::dispatch_shared_drawer_header_pointer_click;
pub(crate) use hierarchy::dispatch_shared_hierarchy_pointer_click;
pub(crate) use host_page::dispatch_shared_host_page_pointer_click;
pub(crate) use menu::dispatch_shared_menu_pointer_click;
pub(crate) use viewport_toolbar::dispatch_shared_viewport_toolbar_pointer_click;
pub(crate) use welcome_recent::dispatch_shared_welcome_recent_pointer_click;
