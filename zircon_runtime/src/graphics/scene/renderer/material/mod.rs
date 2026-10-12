#[path = "../../material/bindless_material_eligibility.rs"]
mod bindless_material_eligibility;
#[path = "../../material/bindless_material_payload_registry.rs"]
mod bindless_material_payload_registry;
#[path = "bindless_slab.rs"]
mod bindless_slab;

pub(crate) use crate::graphics::scene::gpu_scene::{
    GpuBindlessMaterialPayload, BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT,
    GPU_BINDLESS_MATERIAL_PAYLOAD_STRIDE,
};
pub(crate) use bindless_material_eligibility::{
    bindless_material_eligibility, BindlessMaterialEligibility, BindlessMaterialFallbackReason,
};
pub(crate) use bindless_material_payload_registry::{
    BindlessMaterialPayloadPrepareResult, BindlessMaterialPayloadRegistry,
    BindlessMaterialPayloadSlot,
};
pub(crate) use bindless_slab::{
    BindlessMaterialBindingTable, BindlessMaterialBindingTableError, BindlessMaterialSlab,
    BindlessMaterialSlabError, BindlessSlotIndex, BindlessSlotLease, BindlessTextureKey,
};
