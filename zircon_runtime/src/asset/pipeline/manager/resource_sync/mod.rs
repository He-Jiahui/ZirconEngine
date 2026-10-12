#[path = "../../../resource_sync/clear_removed_project_resources.rs"]
mod clear_removed_project_resources;
#[path = "project_locators.rs"]
mod project_locators;
#[path = "../../../manager/resource_sync/reconcile_project_resources.rs"]
mod reconcile_project_resources;
#[path = "../../../manager/resource_sync/register_project_resource.rs"]
mod register_project_resource;
#[path = "store_runtime_payload.rs"]
mod store_runtime_payload;

pub(in crate::asset::pipeline::manager) use clear_removed_project_resources::clear_removed_project_resources;
pub(in crate::asset::pipeline::manager) use project_locators::project_locators;
pub(in crate::asset::pipeline::manager) use reconcile_project_resources::{
    project_resource_identities, reconcile_project_resources,
};
pub(in crate::asset::pipeline::manager) use register_project_resource::register_project_resource;
pub(in crate::asset::pipeline::manager) use store_runtime_payload::store_runtime_payload;
