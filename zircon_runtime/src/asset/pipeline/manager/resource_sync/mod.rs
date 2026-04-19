mod clear_removed_project_resources;
mod project_locators;
mod register_project_resource;
mod store_runtime_payload;

pub(in crate::asset::pipeline::manager) use clear_removed_project_resources::clear_removed_project_resources;
pub(in crate::asset::pipeline::manager) use project_locators::project_locators;
pub(in crate::asset::pipeline::manager) use register_project_resource::register_project_resource;
pub(in crate::asset::pipeline::manager) use store_runtime_payload::store_runtime_payload;
