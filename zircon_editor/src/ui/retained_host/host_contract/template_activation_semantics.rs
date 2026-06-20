mod asset;
mod dispatch;
mod helpers;
mod route;

pub(in crate::ui::retained_host::host_contract) use dispatch::dispatch_template_node_primary_press;

#[cfg(test)]
#[path = "template_activation_semantics_tests.rs"]
mod tests;
