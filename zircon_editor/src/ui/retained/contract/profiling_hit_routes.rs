#[path = "profiling_hit_routes/chrome.rs"]
mod chrome;
#[path = "profiling_hit_routes/geometry.rs"]
mod geometry;
#[path = "profiling_hit_routes/identity.rs"]
mod identity;
#[path = "profiling_hit_routes/routes.rs"]
mod routes;
#[path = "profiling_hit_routes/tabs.rs"]
mod tabs;
#[path = "profiling_hit_routes/template.rs"]
mod template;
#[path = "profiling_hit_routes/viewport_toolbar.rs"]
mod viewport_toolbar;

pub(in crate::ui::retained_host::host_contract) use routes::route_contains_profile_frame;
