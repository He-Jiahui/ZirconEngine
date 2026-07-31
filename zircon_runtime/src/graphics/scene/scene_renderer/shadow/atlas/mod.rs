mod allocator;
mod bindings;
mod resources;

pub(crate) use allocator::{
    SHADOW_ATLAS_DEFAULT_CSM_ROW_HEIGHT, ShadowAtlasAllocator, ShadowAtlasConfig, ShadowAtlasRect,
    ShadowSlotAllocation, ShadowSlotKey, ShadowSlotRequest,
};
pub(crate) use bindings::{
    SHADOW_ATLAS_BINDING, SHADOW_ATLAS_SAMPLER_BINDING, SHADOW_ATLAS_SLOT_BUFFER_BINDING,
    SHADOW_GLOBALS_BINDING, shadow_atlas_bind_group_layout_entries,
};
pub(crate) use resources::{ShadowAtlasResourceConfig, ShadowAtlasResources};
