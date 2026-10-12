#[path = "command.rs"]
mod command;
#[path = "document_tab.rs"]
mod document_tab;
#[path = "drawer_toggle.rs"]
mod drawer_toggle;
#[path = "floating_window/mod.rs"]
mod floating_window;
#[path = "main_page.rs"]
mod main_page;
#[path = "tab_drop.rs"]
mod tab_drop;

pub(crate) use command::dispatch_layout_command;
pub(crate) use document_tab::{
    dispatch_builtin_host_document_tab_activation, dispatch_builtin_host_document_tab_close,
};
pub(crate) use drawer_toggle::dispatch_builtin_host_drawer_toggle;
pub(crate) use floating_window::{
    dispatch_builtin_floating_window_focus, dispatch_builtin_floating_window_focus_for_source,
};
pub(crate) use main_page::dispatch_builtin_host_page_activation;
pub(crate) use tab_drop::dispatch_tab_drop;
