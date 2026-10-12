#[path = "collect.rs"]
mod collect;
#[path = "page_tabs.rs"]
mod page_tabs;
#[path = "workspace_tabs.rs"]
mod workspace_tabs;

pub(crate) use page_tabs::document_tabs_for_page;
pub(crate) use workspace_tabs::workspace_tabs;
