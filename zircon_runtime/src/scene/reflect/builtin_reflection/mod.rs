#[path = "active_in_hierarchy.rs"]
mod active_in_hierarchy;
#[path = "component_support.rs"]
mod component_support;
#[path = "hierarchy.rs"]
mod hierarchy;
#[path = "registration.rs"]
mod registration;

pub(super) use registration::register;
