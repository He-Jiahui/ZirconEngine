use super::*;

#[test]
fn rgba32_face_mip_decodes_into_destination_without_changing_layout() {
    let info = ExternalSourceCubemapContainerInfo {
        kind: ExternalSourceCubemapContainerKind::Dds,
        format: "dds/rgba32f".to_string(),
        face_size: 1,
        mip_count: 1,
    };
    let bytes = [
        2.0_f32.to_le_bytes(),
        (-1.0_f32).to_le_bytes(),
        f32::NAN.to_le_bytes(),
        7.0_f32.to_le_bytes(),
    ]
    .concat();
    let mut output = vec![[9.0; 4]; source_cubemap_sample_count(1, 1)];

    write_face_mip(
        &mut output,
        &info,
        CubemapFace::PositiveX,
        0,
        &bytes,
        SourceTexelFormat::Rgba32Float,
    )
    .expect("valid RGBA32F face/mip");

    assert_eq!(output[0], [2.0, 0.0, 0.0, 1.0]);
}

#[test]
fn rgba32_face_mip_rejects_wrong_length_before_writing() {
    let info = ExternalSourceCubemapContainerInfo {
        kind: ExternalSourceCubemapContainerKind::Dds,
        format: "dds/rgba32f".to_string(),
        face_size: 1,
        mip_count: 1,
    };
    let mut output = vec![[9.0; 4]; source_cubemap_sample_count(1, 1)];

    let result = write_face_mip(
        &mut output,
        &info,
        CubemapFace::PositiveX,
        0,
        &[0; 15],
        SourceTexelFormat::Rgba32Float,
    );

    assert!(result.is_err());
    assert_eq!(output[0], [9.0; 4]);
}
