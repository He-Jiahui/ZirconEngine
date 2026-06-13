mod allocator;
mod bindings;
mod resources;

#[allow(unused_imports)]
pub(crate) use allocator::{
    ShadowAtlasAllocator, ShadowAtlasConfig, ShadowAtlasFrameAllocation, ShadowAtlasRect,
    ShadowSlotAllocation, ShadowSlotKey, ShadowSlotRejection, ShadowSlotRejectionReason,
    ShadowSlotRequest, SHADOW_ATLAS_DEFAULT_CSM_ROW_HEIGHT, SHADOW_ATLAS_DEFAULT_SIZE,
    SHADOW_ATLAS_PREEMPTION_FRAMES, SHADOW_ATLAS_PREEMPTION_SCORE_MULTIPLIER,
    SHADOW_ATLAS_SLOT_RETENTION_FRAMES,
};
#[allow(unused_imports)]
pub(crate) use bindings::{
    shadow_atlas_bind_group_layout_entries, SHADOW_ATLAS_BINDING, SHADOW_ATLAS_SAMPLER_BINDING,
    SHADOW_ATLAS_SLOT_BUFFER_BINDING, SHADOW_GLOBALS_BINDING,
};
#[allow(unused_imports)]
pub(crate) use resources::{
    ShadowAtlasResourceConfig, ShadowAtlasResources, ShadowAtlasUploadReport,
    SHADOW_ATLAS_DEFAULT_SLOT_CAPACITY,
};
