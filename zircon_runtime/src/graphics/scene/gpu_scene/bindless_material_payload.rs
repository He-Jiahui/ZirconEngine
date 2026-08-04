use bytemuck::{Pod, Zeroable};

use crate::graphics::scene::resources::{MaterialRuntime, standard_material_uniform_contents};

pub(crate) const BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT: usize = 6;
pub(crate) const GPU_BINDLESS_MATERIAL_PAYLOAD_STRIDE: usize = 224;
const STANDARD_MATERIAL_UNIFORM_BYTE_LEN: usize = 192;
const RESERVED_BINDLESS_MATERIAL_SLOT_COUNT: usize = 2;

/// std430-ready material row consumed by the bindless shader variant.
///
/// The first 192 bytes deliberately mirror `StandardMaterialPropertyUniform`, preserving the
/// established surface ABI. The trailing indices select the global texture/sampler array; unused
/// entries remain slot zero so shader variants can safely grow without introducing unbound data.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuBindlessMaterialPayload {
    pub(crate) properties: [[f32; 4]; 12],
    pub(crate) texture_slots: [u32; BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT
        + RESERVED_BINDLESS_MATERIAL_SLOT_COUNT],
}

impl GpuBindlessMaterialPayload {
    /// Encodes the same standard-material properties as the per-material uniform path.
    pub(crate) fn from_standard_material(
        material: &MaterialRuntime,
        texture_slots: [u32; BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT],
    ) -> Self {
        Self::from_standard_uniform_bytes(standard_material_uniform_contents(material), texture_slots)
    }

    pub(crate) fn from_standard_uniform_bytes(
        uniform_bytes: [u8; STANDARD_MATERIAL_UNIFORM_BYTE_LEN],
        texture_slots: [u32; BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT],
    ) -> Self {
        let mut slots = [0;
            BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT + RESERVED_BINDLESS_MATERIAL_SLOT_COUNT];
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
mod tests {
    use super::{
        BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT, GPU_BINDLESS_MATERIAL_PAYLOAD_STRIDE,
        GpuBindlessMaterialPayload,
    };

    #[test]
    fn bindless_material_payload_preserves_uniform_rows_and_fallback_reserve_slots() {
        let mut uniform_bytes = [0; 192];
        for (index, value) in [1.0_f32, 2.0, 3.0, 4.0].into_iter().enumerate() {
            let byte_offset = index * std::mem::size_of::<f32>();
            uniform_bytes[byte_offset..byte_offset + std::mem::size_of::<f32>()]
                .copy_from_slice(&value.to_le_bytes());
        }
        let slots = [1, 2, 3, 4, 5, 6];

        let payload = GpuBindlessMaterialPayload::from_standard_uniform_bytes(uniform_bytes, slots);

        assert_eq!(
            std::mem::size_of_val(&payload),
            GPU_BINDLESS_MATERIAL_PAYLOAD_STRIDE
        );
        assert_eq!(payload.properties[0], [1.0, 2.0, 3.0, 4.0]);
        for slot in 0..BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT {
            assert_eq!(payload.texture_slot(slot), slot as u32 + 1);
        }
        assert_eq!(payload.texture_slot(6), 0);
        assert_eq!(payload.texture_slot(7), 0);
    }

    #[test]
    fn bindless_material_payload_default_uses_zero_properties_and_fallback_texture_slots() {
        let payload = GpuBindlessMaterialPayload::default();

        assert!(payload.properties.iter().all(|row| *row == [0.0; 4]));
        for slot in 0..BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT + 2 {
            assert_eq!(payload.texture_slot(slot), 0);
        }
        assert_eq!(payload.texture_slot(usize::MAX), 0);
    }
}
