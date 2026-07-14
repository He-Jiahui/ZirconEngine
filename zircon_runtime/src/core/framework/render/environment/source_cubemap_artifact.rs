use super::{
    source_cubemap_sample_count, IblBakeArtifactContents, IblBakeArtifactPayload,
    SourceCubemapEnvironment, SourceCubemapIrradianceCube, SourceCubemapMipChain,
    SOURCE_CUBEMAP_FACE_COUNT, SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceCubemapBakeArtifactError {
    LayoutMismatch {
        expected_face_size: u32,
        actual_face_size: u32,
        expected_mip_count: u32,
        actual_mip_count: u32,
    },
    MissingPmrem,
    MissingIrradianceSh9,
    InvalidPmremTexelCount {
        expected: usize,
        actual: usize,
    },
    MissingIrradianceCube,
    InvalidIrradianceCubeTexelCount {
        expected: usize,
        actual: usize,
    },
}

pub fn source_cubemap_mip_chain_with_bake_artifact(
    source: &SourceCubemapMipChain,
    payload: &IblBakeArtifactPayload,
) -> Result<SourceCubemapMipChain, SourceCubemapBakeArtifactError> {
    let descriptor = payload.descriptor();
    if descriptor.source_face_size() != source.source_face_size()
        || descriptor.source_mip_count() != source.source_mip_count()
    {
        return Err(SourceCubemapBakeArtifactError::LayoutMismatch {
            expected_face_size: source.source_face_size(),
            actual_face_size: descriptor.source_face_size(),
            expected_mip_count: source.source_mip_count(),
            actual_mip_count: descriptor.source_mip_count(),
        });
    }

    if !descriptor
        .contents()
        .contains(IblBakeArtifactContents::PMREM)
    {
        return Err(SourceCubemapBakeArtifactError::MissingPmrem);
    }
    if !descriptor.contents().contains(IblBakeArtifactContents::SH9) {
        return Err(SourceCubemapBakeArtifactError::MissingIrradianceSh9);
    }

    let pmrem_texels = payload
        .decode_pmrem_texels()
        .ok_or(SourceCubemapBakeArtifactError::MissingPmrem)?;
    let expected = source_cubemap_sample_count(descriptor.face_size(), descriptor.mip_count());
    if pmrem_texels.len() != expected {
        return Err(SourceCubemapBakeArtifactError::InvalidPmremTexelCount {
            expected,
            actual: pmrem_texels.len(),
        });
    }
    let irradiance_sh9 = payload
        .decode_irradiance_sh9()
        .ok_or(SourceCubemapBakeArtifactError::MissingIrradianceSh9)?;

    Ok(
        SourceCubemapMipChain::new_with_source_texels_and_irradiance_sh9(
            source.source_face_size(),
            source.source_mip_count(),
            source.source_texels().to_vec(),
            descriptor.face_size(),
            descriptor.mip_count(),
            pmrem_texels,
            irradiance_sh9,
        ),
    )
}

pub fn source_cubemap_environment_with_bake_artifact(
    mut environment: SourceCubemapEnvironment,
    payload: &IblBakeArtifactPayload,
) -> Result<SourceCubemapEnvironment, SourceCubemapBakeArtifactError> {
    let bake_artifact_hash = bake_artifact_payload_hash(payload);
    let mip_chain = source_cubemap_mip_chain_with_bake_artifact(&environment.mip_chain, payload)?;
    environment.irradiance_sh9 = *mip_chain.irradiance_sh9();
    environment.mip_chain = mip_chain;
    environment.bake_artifact_hash = bake_artifact_hash;
    environment.irradiance_cube = if payload
        .descriptor()
        .contents()
        .contains(IblBakeArtifactContents::IEM)
    {
        Some(decode_irradiance_cube(payload)?)
    } else {
        None
    };
    Ok(environment)
}

fn bake_artifact_payload_hash(payload: &IblBakeArtifactPayload) -> [u32; 4] {
    let descriptor = payload.descriptor();
    let bake_key = descriptor.bake_key();
    let mut hasher = blake3::Hasher::new();
    hasher.update(&bake_key.source_kind.to_le_bytes());
    hasher.update(&bake_key.source_revision.to_le_bytes());
    update_u32_array_hash(&mut hasher, &bake_key.horizon_color);
    update_u32_array_hash(&mut hasher, &bake_key.zenith_color);
    update_u32_array_hash(&mut hasher, &bake_key.ground_color);
    update_u32_array_hash(&mut hasher, &bake_key.source_hash);
    hasher.update(&descriptor.algorithm_version().to_le_bytes());
    hasher.update(&descriptor.source_face_size().to_le_bytes());
    hasher.update(&descriptor.source_mip_count().to_le_bytes());
    hasher.update(&descriptor.face_size().to_le_bytes());
    hasher.update(&descriptor.mip_count().to_le_bytes());
    hasher.update(&descriptor.contents().bits().to_le_bytes());
    hasher.update(payload.bytes());
    hash_words(hasher.finalize())
}

fn update_u32_array_hash(hasher: &mut blake3::Hasher, values: &[u32; 4]) {
    for value in values {
        hasher.update(&value.to_le_bytes());
    }
}

fn hash_words(hash: blake3::Hash) -> [u32; 4] {
    let bytes = hash.as_bytes();
    [
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
    ]
}

fn decode_irradiance_cube(
    payload: &IblBakeArtifactPayload,
) -> Result<SourceCubemapIrradianceCube, SourceCubemapBakeArtifactError> {
    let texels = payload
        .decode_irradiance_cube_texels()
        .ok_or(SourceCubemapBakeArtifactError::MissingIrradianceCube)?;
    let expected = SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
        * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
        * SOURCE_CUBEMAP_FACE_COUNT;
    if texels.len() != expected {
        return Err(
            SourceCubemapBakeArtifactError::InvalidIrradianceCubeTexelCount {
                expected,
                actual: texels.len(),
            },
        );
    }
    Ok(SourceCubemapIrradianceCube::new(
        SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
        texels,
    ))
}
