mod command;
mod document_tab;
mod drawer_toggle;
mod floating_window;
mod main_page;
mod tab_drop;

pub(crate) use command::dispatch_layout_command;
pub(crate) use document_tab::{
    dispatch_builtin_host_document_tab_activation, dispatch_builtin_host_document_tab_close,
};
pub(crate) use drawer_toggle::dispatch_builtin_host_drawer_toggle;
pub(crate) use floating_window::{
    dispatch_builtin_floating_window_focus, dispatch_builtin_floating_window_focus_for_source,
    resolve_builtin_floating_window_close_instances,
};
pub(crate) use main_page::dispatch_builtin_host_page_activation;
pub(crate) use tab_drop::dispatch_tab_drop;
