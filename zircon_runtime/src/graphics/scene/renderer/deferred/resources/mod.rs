#[path = "construct.rs"]
mod construct;
#[path = "resources.rs"]
mod deferred_scene_resources;
#[path = "../../../deferred/resources/execute_lighting.rs"]
mod execute_lighting;
#[path = "../../../deferred/resources/record_gbuffer_geometry.rs"]
mod record_gbuffer_geometry;

pub(crate) use construct::DeferredSceneResourcesStartupReport;
pub(crate) use deferred_scene_resources::DeferredSceneResources;
