mod executor;
mod resources;

pub(crate) use resources::{
    irradiance_volume_bind_group_layout_entries, IrradianceVolumeResources,
};

pub const IRRADIANCE_VOLUME_BIND_EXECUTOR_ID: &str = "irradiance.volume_bind";
pub const IRRADIANCE_VOLUME_RESOURCE: &str = "advanced_lighting.irradiance_volume";

pub(crate) fn registrations(
) -> Vec<crate::graphics::scene::scene_renderer::graph_execution::RenderPassExecutorRegistration> {
    executor::registrations()
}
