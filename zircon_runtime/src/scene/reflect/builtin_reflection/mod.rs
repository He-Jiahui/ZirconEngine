#[path = "active_in_hierarchy.rs"]
mod active_in_hierarchy;
#[path = "component_support.rs"]
mod component_support;
#[path = "gameplay.rs"]
mod gameplay;
#[path = "hierarchy.rs"]
mod hierarchy;
#[path = "registration.rs"]
mod registration;
#[path = "script_bindings.rs"]
mod script_bindings;

pub(super) use registration::register;
