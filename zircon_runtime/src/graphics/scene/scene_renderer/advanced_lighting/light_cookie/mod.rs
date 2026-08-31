mod blit_pipeline;
mod executor;
mod frame_plan;
mod profile;
mod projection;
mod resources;

pub(crate) use frame_plan::{
    COOKIE_ATLAS_GRID_SIZE, COOKIE_ATLAS_MAX_ENTRIES, CookieFramePlan, CookieGpuMetadata,
    build_cookie_frame_plan,
};
pub(crate) use projection::{directional_cookie_uv, point_octahedral_cookie_uv, spot_cookie_uv};
pub(crate) use resources::{
    LIGHT_COOKIE_ATLAS_BINDING, LIGHT_COOKIE_SAMPLER_BINDING, LightCookieAtlasResources,
    light_cookie_bind_group_layout_entries,
};

pub const LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID: &str = "cookie.atlas_build";
pub const LIGHT_COOKIE_ATLAS_RESOURCE: &str = "advanced_lighting.cookie_atlas";

pub(crate) fn registrations()
-> Vec<crate::graphics::scene::scene_renderer::graph_execution::RenderPassExecutorRegistration> {
    executor::registrations()
}

pub(crate) const COOKIE_PROJECTION_NONE: u32 = 0;
pub(crate) const COOKIE_PROJECTION_DIRECTIONAL: u32 = 1;
pub(crate) const COOKIE_PROJECTION_SPOT: u32 = 2;
pub(crate) const COOKIE_PROJECTION_POINT_OCTAHEDRAL: u32 = 3;
