#[path = "activation_semantics/asset.rs"]
mod asset;
#[path = "activation_semantics/dispatch.rs"]
mod dispatch;
#[path = "activation_semantics/helpers.rs"]
mod helpers;
#[path = "activation_semantics/route.rs"]
mod route;

pub(in crate::ui::retained_host::host_contract) use dispatch::dispatch_template_node_primary_press;

#[cfg(test)]
#[path = "tests/activation_semantics.rs"]
mod tests;
