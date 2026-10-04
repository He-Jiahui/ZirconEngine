use bytemuck::{Pod, Zeroable};

use crate::graphics::scene::resources::{standard_material_uniform_contents, MaterialRuntime};

pub(crate) const BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT: usize = 6;
pub(crate) const GPU_BINDLESS_MATERIAL_PAYLOAD_STRIDE: usize = 288;
const STANDARD_MATERIAL_UNIFORM_BYTE_LEN: usize = 256;
const RESERVED_BINDLESS_MATERIAL_SLOT_COUNT: usize = 2;

/// std430-ready material row consumed by the bindless shader variant.
///
/// The first 256 bytes deliberately mirror `StandardMaterialPropertyUniform`, preserving the
/// established surface ABI. The trailing indices select the global texture/sampler array; unused
/// entries remain slot zero so shader variants can safely grow without introducing unbound data.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuBindlessMaterialPayload {
    pub(crate) properties: [[f32; 4]; 16],
    pub(crate) texture_slots: [u32; BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT
        + RESERVED_BINDLESS_MATERIAL_SLOT_COUNT],
}

impl GpuBindlessMaterialPayload {
    /// Encodes the same standard-material properties as the per-material uniform path.
    pub(crate) fn from_standard_material(
        material: &MaterialRuntime,
        texture_slots: [u32; BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT],
    ) -> Self {
        Self::from_standard_uniform_bytes(
            standard_material_uniform_contents(material),
            texture_slots,
        )
    }

    pub(crate) fn from_standard_uniform_bytes(
        uniform_bytes: [u8; STANDARD_MATERIAL_UNIFORM_BYTE_LEN],
        texture_slots: [u32; BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT],
    ) -> Self {
        let mut slots = [0; BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT
            + RESERVED_BINDLESS_MATERIAL_SLOT_COUNT];
        for (target, source) in slots.iter_mut().zip(texture_slots) {
            *target = source;
        }

        Self {
            properties: bytemuck::cast(uniform_bytes),
            texture_slots: slots,
        }
    }

    pub(crate) const fn texture_slot(&self, slot: usize) -> u32 {
        if slot < self.texture_slots.len() {
            self.texture_slots[slot]
        } else {
            0
        }
    }
}

const _: () = assert!(
    std::mem::size_of::<GpuBindlessMaterialPayload>() == GPU_BINDLESS_MATERIAL_PAYLOAD_STRIDE
);

#[cfg(test)]
#[path = "tests/bindless_material_payload.rs"]
mod tests;
