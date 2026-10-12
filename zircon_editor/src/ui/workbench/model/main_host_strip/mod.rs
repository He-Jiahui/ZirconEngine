#[path = "active_page.rs"]
mod active_page;
#[path = "active_view.rs"]
mod active_view;
#[path = "breadcrumbs.rs"]
mod breadcrumbs;
#[path = "host_strip_model.rs"]
mod host_strip_model;
#[path = "page_access.rs"]
mod page_access;

pub(crate) use active_page::active_page_snapshot;
pub(crate) use host_strip_model::host_strip_model;
