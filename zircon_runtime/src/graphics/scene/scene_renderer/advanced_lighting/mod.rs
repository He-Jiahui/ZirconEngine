pub(crate) mod froxel;
pub(crate) mod irradiance_volume;
pub(crate) mod light_cookie;
pub(crate) mod oit_buffers;
pub(crate) mod planar_filter;
pub(crate) mod subsurface_pass;
pub(crate) mod transmission;

use crate::graphics::scene::scene_renderer::graph_execution::RenderPassExecutorRegistration;

pub fn volumetric_fog_render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    froxel::executor_registrations()
}

pub fn oit_render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    oit_buffers::registrations()
}

pub fn light_cookie_render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    light_cookie::registrations()
}

pub fn irradiance_volume_render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration>
{
    irradiance_volume::registrations()
}

pub fn planar_reflection_render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration>
{
    planar_filter::registrations()
}

pub fn subsurface_render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    subsurface_pass::registrations()
}

pub use oit_buffers::{
    OIT_DRAW_SHADER_SOURCE, OIT_FRAGMENT_STORE_EXECUTOR_ID, OIT_RESOLVE_EXECUTOR_ID,
    OIT_RESOLVE_SHADER_SOURCE,
};

pub use irradiance_volume::{IRRADIANCE_VOLUME_BIND_EXECUTOR_ID, IRRADIANCE_VOLUME_RESOURCE};
pub use light_cookie::{LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID, LIGHT_COOKIE_ATLAS_RESOURCE};

pub use froxel::{
    VOLUMETRIC_INTEGRATE_EXECUTOR_ID, VOLUMETRIC_LIGHT_SCATTER_EXECUTOR_ID,
    VOLUMETRIC_MEDIA_INJECT_EXECUTOR_ID,
};

pub use planar_filter::{
    PLANAR_FILTER_EXECUTOR_ID, PLANAR_REFLECTION_TEXTURE_RESOURCE,
    planar_reflection_filter_compute_workload,
};

pub use subsurface_pass::{
    SSS_RECOMBINE_EXECUTOR_ID, SSS_SCATTER_EXECUTOR_ID, SSS_SETUP_EXECUTOR_ID,
    render_feature_descriptor as subsurface_render_feature_descriptor,
    scatter_compute_workload as subsurface_scatter_compute_workload,
    setup_compute_workload as subsurface_setup_compute_workload,
};
