#[path = "action_route.rs"]
mod action_route;
#[path = "activity_routes.rs"]
mod activity_routes;
#[path = "binding_route.rs"]
mod binding_route;
#[path = "register_model.rs"]
mod register_model;

pub(crate) use binding_route::register_binding_route;
pub use register_model::register_workbench_reflection_routes;
