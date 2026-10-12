#[path = "context_menu/classifier.rs"]
mod classifier;
#[path = "context_menu/path.rs"]
mod path;
#[path = "context_menu/provider.rs"]
mod provider;
#[path = "context_menu/request.rs"]
mod request;

pub(in crate::ui::retained_host::host_contract) use request::workbench_context_menu_request_for_hit;

#[cfg(test)]
#[path = "tests/context_menu.rs"]
mod tests;
