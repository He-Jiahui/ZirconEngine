mod bindless_material_eligibility;
mod bindless_material_payload_registry;
mod bindless_slab;

pub(crate) use crate::graphics::scene::gpu_scene::{
    BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT, GPU_BINDLESS_MATERIAL_PAYLOAD_STRIDE,
    GpuBindlessMaterialPayload,
};
pub(crate) use bindless_material_eligibility::{
    BindlessMaterialEligibility, BindlessMaterialFallbackReason, bindless_material_eligibility,
};
pub(crate) use bindless_material_payload_registry::{
    BindlessMaterialPayloadPrepareResult, BindlessMaterialPayloadRegistry,
    BindlessMaterialPayloadSlot,
};
pub(crate) use bindless_slab::{
    BindlessMaterialBindingTable, BindlessMaterialBindingTableError, BindlessMaterialSlab,
    BindlessMaterialSlabError, BindlessSlotIndex, BindlessSlotLease, BindlessTextureKey,
};
