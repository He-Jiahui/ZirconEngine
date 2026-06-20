mod classifier;
mod path;
mod provider;
mod request;

pub(in crate::ui::retained_host::host_contract) use request::workbench_context_menu_request_for_hit;

#[cfg(test)]
#[path = "workbench_context_menu_tests.rs"]
mod tests;
