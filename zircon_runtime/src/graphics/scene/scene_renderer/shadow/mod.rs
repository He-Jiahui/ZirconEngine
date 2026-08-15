pub(crate) mod atlas;
pub(crate) mod cascade;
mod plan;
mod shadow_cache;
mod shadow_map_renderer;
pub(crate) mod slot;
mod view_projection;

pub(crate) use plan::{
    build_shadow_frame_plan, build_shadow_frame_plan_with_static_caster_revision, ShadowFramePlan,
    ShadowLightSlotAssignment, ShadowLightSlotAssignments,
};
pub(crate) use shadow_cache::{
    shadow_light_params_hash, static_shadow_caster_revision,
    static_shadow_caster_revision_from_meshes,
    static_shadow_caster_revision_from_meshes_with_resource_revisions, ShadowCache,
    ShadowCacheDecision, ShadowCacheEntry, ShadowCacheInput, ShadowCacheInvalidationReason,
    ShadowStaticCasterRevisionInput,
};
pub(crate) use shadow_map_renderer::ShadowMapRenderer;
