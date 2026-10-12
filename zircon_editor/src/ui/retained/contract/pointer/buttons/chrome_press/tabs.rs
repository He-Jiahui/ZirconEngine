#[path = "../tabs/document.rs"]
mod document;
#[path = "../tabs/drawer.rs"]
mod drawer;
#[path = "../tabs/host_page.rs"]
mod host_page;

pub(super) use self::document::dispatch_document_tab_press;
pub(super) use self::drawer::dispatch_drawer_header_tab_press;
pub(super) use self::host_page::dispatch_host_page_tab_press;
