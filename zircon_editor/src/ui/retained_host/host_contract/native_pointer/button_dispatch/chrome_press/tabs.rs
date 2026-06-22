mod document;
mod drawer;
mod host_page;

pub(super) use self::document::dispatch_document_tab_press;
pub(super) use self::drawer::dispatch_drawer_header_tab_press;
pub(super) use self::host_page::dispatch_host_page_tab_press;
