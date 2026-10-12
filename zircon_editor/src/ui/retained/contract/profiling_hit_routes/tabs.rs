#[path = "tabs/document.rs"]
mod document;
#[path = "tabs/drawer.rs"]
mod drawer;
#[path = "tabs/floating.rs"]
mod floating;
#[path = "tabs/host_page.rs"]
mod host_page;
#[path = "tabs/shared.rs"]
mod shared;

pub(in crate::ui::retained_host::host_contract) use document::document_tab_route_hit;
pub(in crate::ui::retained_host::host_contract) use drawer::drawer_tab_route_hit;
pub(in crate::ui::retained_host::host_contract) use floating::floating_tab_route_hit;
pub(in crate::ui::retained_host::host_contract) use host_page::host_page_tab_route_hit;
