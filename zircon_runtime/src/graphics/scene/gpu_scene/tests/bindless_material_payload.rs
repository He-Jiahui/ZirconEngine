use super::{
    GpuBindlessMaterialPayload, BINDLESS_STANDARD_MATERIAL_TEXTURE_SLOT_COUNT,
    GPU_BINDLESS_MATERIAL_PAYLOAD_STRIDE,
};

#[test]
fn bindless_material_payload_preserves_uniform_rows_and_fallback_reserve_slots() {
    let mut uniform_bytes = [0; 256];
    for (index, value) in [1.0_f32, 2.0, 3.0, 4.0].into_iter().enumerate() {
        let byte_offset = index * std::mem::size_of::<f32>();
        uniform_bytes[byte_offset..byte_offset + std::mem::size_of::<f32>()]
            .copy_from_slice(&value.to_le_bytes());
    }
    uniform_bytes[192..196].copy_from_slice(&0.35_f32.to_le_bytes());
    uniform_bytes[208..212].copy_from_slice(&0.5_f32.to_le_bytes());
    let slots = [1, 2, 3, 4, 5, 6];

    let payload = GpuBindlessMaterialPayload::from_standard_uniform_bytes(uniform_bytes, slots);

    assert_eq!(
        std::mem::size_of_val(&payload),
        GPU_BINDLESS_MATERIAL_PAYLOAD_STRIDE
    );
    assert_eq!(payload.properties[0], [1.0, 2.0, 3.0, 4.0]);
    assert_eq!(payload.properties[12], [0.35, 0.0, 0.0, 0.0]);
    assert_eq!(payload.properties[13], [0.5, 0.0, 0.0, 0.0]);
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
