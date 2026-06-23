mod allocator;
mod bindings;
mod resources;

pub(crate) use allocator::{
    ShadowAtlasAllocator, ShadowAtlasConfig, ShadowAtlasRect, ShadowSlotAllocation, ShadowSlotKey,
    ShadowSlotRequest, SHADOW_ATLAS_DEFAULT_CSM_ROW_HEIGHT,
};
pub(crate) use bindings::{
    shadow_atlas_bind_group_layout_entries, SHADOW_ATLAS_BINDING, SHADOW_ATLAS_SAMPLER_BINDING,
    SHADOW_ATLAS_SLOT_BUFFER_BINDING, SHADOW_GLOBALS_BINDING,
};
pub(crate) use resources::{ShadowAtlasResourceConfig, ShadowAtlasResources};
