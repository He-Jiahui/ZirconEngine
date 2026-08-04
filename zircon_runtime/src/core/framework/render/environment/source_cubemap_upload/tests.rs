use super::build_source_cubemap_upload_artifact;
use crate::core::framework::render::{
    encode_rgba16f_texels, source_cubemap_face_mip_offset, CubemapFace,
    SourceCubemapIrradianceCube, SourceCubemapMipChain,
};

#[test]
fn source_cubemap_upload_artifact_groups_each_mip_by_face() {
    let source_texels = (0..30)
        .map(|value| [value as f32, 0.0, 0.0, 1.0])
        .collect::<Vec<_>>();
    let mip_chain = SourceCubemapMipChain::new(
        2,
        2,
        source_texels.clone(),
        1,
        1,
        vec![[0.0, 0.0, 0.0, 1.0]; 6],
    );

    let artifact = build_source_cubemap_upload_artifact(&mip_chain, None);

    assert_eq!(artifact.source_mips().len(), 2);
    assert_eq!(artifact.source_mips()[0].face_size(), 2);
    assert_eq!(artifact.source_mips()[1].face_size(), 1);
    assert_eq!(artifact.source_mips()[0].bytes_per_row(), 256);
    assert_eq!(artifact.source_mips()[1].bytes_per_row(), 256);
    assert_eq!(artifact.source_mips()[0].bytes().len(), 6 * 2 * 256);
    assert_eq!(artifact.source_mips()[1].bytes().len(), 6 * 256);
    assert_eq!(artifact.pmrem_mips().len(), 1);
    assert_eq!(artifact.pmrem_mips()[0].face_size(), 1);
    assert_eq!(artifact.pmrem_mips()[0].bytes_per_row(), 256);
    assert_eq!(artifact.pmrem_mips()[0].bytes().len(), 6 * 256);
    for face in CubemapFace::ALL {
        let source_offset = source_cubemap_face_mip_offset(2, 2, face, 0);
        let artifact_offset = face.index() * 2 * 256;
        assert_eq!(
            &artifact.source_mips()[0].bytes()[artifact_offset..artifact_offset + 16],
            &encode_rgba16f_texels(&source_texels[source_offset..source_offset + 2])
        );
        assert!(
            artifact.source_mips()[0].bytes()[artifact_offset + 16..artifact_offset + 256]
                .iter()
                .all(|byte| *byte == 0)
        );
    }
}

#[test]
fn source_cubemap_upload_artifact_preserves_optional_irradiance_cube() {
    let mip_chain = SourceCubemapMipChain::new(
        1,
        1,
        vec![[0.0, 0.0, 0.0, 1.0]; 6],
        1,
        1,
        vec![[0.0, 0.0, 0.0, 1.0]; 6],
    );
    let irradiance =
        SourceCubemapIrradianceCube::new(1, vec![[0.25, 0.5, 0.75]; CubemapFace::ALL.len()]);

    let artifact = build_source_cubemap_upload_artifact(&mip_chain, Some(&irradiance));
    let irradiance_mip = artifact
        .irradiance_mip()
        .expect("optional irradiance cube should produce an upload mip");

    assert_eq!(irradiance_mip.mip_level(), 0);
    assert_eq!(irradiance_mip.face_size(), 1);
    assert_eq!(irradiance_mip.bytes_per_row(), 256);
    assert_eq!(irradiance_mip.bytes().len(), CubemapFace::ALL.len() * 256);
}
