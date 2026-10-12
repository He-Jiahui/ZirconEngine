#[path = "activity_rail.rs"]
mod activity_rail;
#[path = "asset_content.rs"]
mod asset_content;
#[path = "asset_reference.rs"]
mod asset_reference;
#[path = "asset_tree.rs"]
mod asset_tree;
#[path = "document_tab.rs"]
mod document_tab;
#[path = "drawer_header.rs"]
mod drawer_header;
#[path = "hierarchy.rs"]
mod hierarchy;
#[path = "host_page.rs"]
mod host_page;
#[path = "menu.rs"]
mod menu;
#[path = "viewport_toolbar.rs"]
mod viewport_toolbar;
#[path = "welcome_recent.rs"]
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
pub(crate) use viewport_toolbar::{
    dispatch_shared_viewport_toolbar_pointer_click,
    dispatch_shared_viewport_toolbar_pointer_click_at_point,
};
pub(crate) use welcome_recent::dispatch_shared_welcome_recent_pointer_click;
