use super::*;

#[test]
fn material_ranges_are_bounded_sorted_and_coalesced() {
    assert_eq!(
        coalesced_material_ranges(&[8..12, 2..4, 4..7, 20..24, 7..8], 16),
        vec![2..12]
    );
}

#[test]
fn material_range_payload_preserves_uniform_alignment() {
    let materials = [
        SdfTextMaterial {
            fill_color: [0.25; 4],
            ..Default::default()
        },
        SdfTextMaterial {
            fill_color: [0.75; 4],
            ..Default::default()
        },
    ];
    let uniform_len = std::mem::size_of::<SdfTextMaterialUniform>();
    let stride = aligned_uniform_stride(uniform_len as u32, 256) as usize;
    let bytes = material_range_upload_bytes(&materials, stride);

    assert_eq!(bytes.len(), stride * materials.len());
    assert_eq!(
        &bytes[..uniform_len],
        bytemuck::bytes_of(&materials[0].uniform())
    );
    assert_eq!(
        &bytes[stride..stride + uniform_len],
        bytemuck::bytes_of(&materials[1].uniform())
    );
}
