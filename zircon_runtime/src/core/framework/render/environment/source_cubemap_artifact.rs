use crate::core::math::Real;

use super::{
    source_cubemap_sample_count, IblBakeArtifactContents, IblBakeArtifactDescriptor,
    IblBakeArtifactPayload, IblBakeArtifactProducer, IblBakeArtifactRequest, IblBakeKey,
    SourceCubemapEnvironment, SourceCubemapIrradianceCube, SourceCubemapIrradianceSh9,
    SourceCubemapMipChain, SOURCE_CUBEMAP_FACE_COUNT, SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
};

const BAKE_ARTIFACT_PAYLOAD_HASH_HEADER_BYTES: usize = 108;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceCubemapBakeArtifactError {
    UnexpectedDirectHydrationProducer {
        expected: IblBakeArtifactProducer,
        actual: IblBakeArtifactProducer,
    },
    NotCurrentForRequest {
        expected: IblBakeArtifactRequest,
        actual: IblBakeArtifactDescriptor,
    },
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
    let sections = decode_bake_artifact_sections(
        source.source_face_size(),
        source.source_mip_count(),
        payload,
    )?;

    Ok(source.with_bake_artifact_pmrem(
        sections.pmrem_face_size,
        sections.pmrem_mip_count,
        sections.pmrem_texels,
        sections.irradiance_sh9,
    ))
}

/// Restores a staged source cubemap using its already-baked CPU artifact.
///
/// This path intentionally avoids reconstructing PMREM from the source texels.
/// The artifact supplies the active SH9 value, while source-derived SH9 remains
/// a separate canonical cache for future PMREM reconfiguration.
pub fn source_cubemap_environment_from_source_mips_with_bake_artifact(
    source_face_size: u32,
    source_mip_count: u32,
    source_texels: Vec<[Real; 4]>,
    source_revision: u64,
    source_hash: [u32; 4],
    payload: &IblBakeArtifactPayload,
) -> Result<SourceCubemapEnvironment, SourceCubemapBakeArtifactError> {
    if payload.descriptor().producer() != IblBakeArtifactProducer::AssetImporterCpu {
        return Err(
            SourceCubemapBakeArtifactError::UnexpectedDirectHydrationProducer {
                expected: IblBakeArtifactProducer::AssetImporterCpu,
                actual: payload.descriptor().producer(),
            },
        );
    }
    let sections = decode_bake_artifact_sections(source_face_size, source_mip_count, payload)?;
    let request = IblBakeArtifactRequest::new(
        IblBakeKey::source_cubemap(source_revision, source_hash),
        source_face_size,
        source_mip_count,
    )
    .with_pmrem_layout(sections.pmrem_face_size, sections.pmrem_mip_count)
    .with_required_contents(payload.descriptor().contents());
    ensure_payload_is_current_for_request(&request, payload)?;
    let source_irradiance_sh9 = SourceCubemapMipChain::source_irradiance_sh9_from_source_texels(
        &source_texels,
        source_face_size,
        source_mip_count,
    );
    let mip_chain = SourceCubemapMipChain::new_with_source_texels_and_irradiance_sh9_pair(
        source_face_size,
        source_mip_count,
        source_texels,
        sections.pmrem_face_size,
        sections.pmrem_mip_count,
        sections.pmrem_texels,
        source_irradiance_sh9,
        sections.irradiance_sh9,
    );
    source_cubemap_environment_from_hydrated_mip_chain(
        mip_chain,
        source_revision,
        source_hash,
        1.0,
        0.0,
        payload,
    )
}

pub fn source_cubemap_environment_with_bake_artifact(
    environment: &SourceCubemapEnvironment,
    payload: &IblBakeArtifactPayload,
) -> Result<SourceCubemapEnvironment, SourceCubemapBakeArtifactError> {
    let mip_chain = source_cubemap_mip_chain_with_bake_artifact(&environment.mip_chain, payload)?;
    let request = environment.ibl_bake_artifact_request(payload.descriptor().contents());
    ensure_payload_is_current_for_request(&request, payload)?;
    source_cubemap_environment_from_hydrated_mip_chain(
        mip_chain,
        environment.source_revision,
        environment.source_hash,
        environment.intensity,
        environment.rotation_radians,
        payload,
    )
}

fn ensure_payload_is_current_for_request(
    request: &IblBakeArtifactRequest,
    payload: &IblBakeArtifactPayload,
) -> Result<(), SourceCubemapBakeArtifactError> {
    let descriptor = payload.descriptor();
    let is_current = match descriptor.producer() {
        IblBakeArtifactProducer::AssetImporterCpu => descriptor.is_current_for(request),
        IblBakeArtifactProducer::RendererGpuRuntime => {
            descriptor.is_current_runtime_cache_for(request)
        }
    };
    if is_current {
        Ok(())
    } else {
        Err(SourceCubemapBakeArtifactError::NotCurrentForRequest {
            expected: *request,
            actual: descriptor,
        })
    }
}

fn source_cubemap_environment_from_hydrated_mip_chain(
    mip_chain: SourceCubemapMipChain,
    source_revision: u64,
    source_hash: [u32; 4],
    intensity: Real,
    rotation_radians: Real,
    payload: &IblBakeArtifactPayload,
) -> Result<SourceCubemapEnvironment, SourceCubemapBakeArtifactError> {
    let irradiance_cube = if payload
        .descriptor()
        .contents()
        .contains(IblBakeArtifactContents::IEM)
    {
        Some(decode_irradiance_cube(payload)?)
    } else {
        None
    };
    let pmrem_hash = pmrem_payload_hash(payload)?;
    let mut hydrated = SourceCubemapEnvironment::new(mip_chain, source_revision, source_hash);
    hydrated.pmrem_hash = pmrem_hash;
    hydrated.bake_artifact_hash = bake_artifact_payload_hash(payload);
    hydrated.intensity = intensity;
    hydrated.rotation_radians = rotation_radians;
    hydrated.irradiance_cube = irradiance_cube;
    Ok(hydrated
        .with_accepted_bake_artifact_descriptor(payload.descriptor())
        .with_prepared_upload_artifact())
}

struct SourceCubemapBakeArtifactSections {
    pmrem_face_size: u32,
    pmrem_mip_count: u32,
    pmrem_texels: Vec<[Real; 4]>,
    irradiance_sh9: SourceCubemapIrradianceSh9,
}

fn decode_bake_artifact_sections(
    source_face_size: u32,
    source_mip_count: u32,
    payload: &IblBakeArtifactPayload,
) -> Result<SourceCubemapBakeArtifactSections, SourceCubemapBakeArtifactError> {
    let descriptor = payload.descriptor();
    if descriptor.source_face_size() != source_face_size
        || descriptor.source_mip_count() != source_mip_count
    {
        return Err(SourceCubemapBakeArtifactError::LayoutMismatch {
            expected_face_size: source_face_size,
            actual_face_size: descriptor.source_face_size(),
            expected_mip_count: source_mip_count,
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

    Ok(SourceCubemapBakeArtifactSections {
        pmrem_face_size: descriptor.face_size(),
        pmrem_mip_count: descriptor.mip_count(),
        pmrem_texels,
        irradiance_sh9,
    })
}

fn bake_artifact_payload_hash(payload: &IblBakeArtifactPayload) -> [u32; 4] {
    let descriptor = payload.descriptor();
    let header = bake_artifact_payload_hash_header(descriptor);
    let mut hasher = blake3::Hasher::new();
    hasher.update(&header);
    hasher.update(payload.bytes());
    hash_words(hasher.finalize())
}

fn bake_artifact_payload_hash_header(
    descriptor: IblBakeArtifactDescriptor,
) -> [u8; BAKE_ARTIFACT_PAYLOAD_HASH_HEADER_BYTES] {
    let bake_key = descriptor.bake_key();
    let mut header = [0_u8; BAKE_ARTIFACT_PAYLOAD_HASH_HEADER_BYTES];
    let mut cursor = 0;
    append_hash_field(&mut header, &mut cursor, bake_key.source_kind.to_le_bytes());
    append_hash_field(
        &mut header,
        &mut cursor,
        bake_key.source_revision.to_le_bytes(),
    );
    for values in [
        bake_key.horizon_color,
        bake_key.zenith_color,
        bake_key.ground_color,
        bake_key.source_hash,
    ] {
        for value in values {
            append_hash_field(&mut header, &mut cursor, value.to_le_bytes());
        }
    }
    append_hash_field(
        &mut header,
        &mut cursor,
        descriptor.algorithm_version().to_le_bytes(),
    );
    for value in [
        descriptor.producer() as u32,
        descriptor.source_face_size(),
        descriptor.source_mip_count(),
        descriptor.face_size(),
        descriptor.mip_count(),
        descriptor.contents().bits(),
    ] {
        append_hash_field(&mut header, &mut cursor, value.to_le_bytes());
    }
    debug_assert_eq!(cursor, BAKE_ARTIFACT_PAYLOAD_HASH_HEADER_BYTES);
    header
}

fn append_hash_field<const N: usize>(
    header: &mut [u8; BAKE_ARTIFACT_PAYLOAD_HASH_HEADER_BYTES],
    cursor: &mut usize,
    field: [u8; N],
) {
    let end = *cursor + N;
    header[*cursor..end].copy_from_slice(&field);
    *cursor = end;
}

fn pmrem_payload_hash(
    payload: &IblBakeArtifactPayload,
) -> Result<[u32; 4], SourceCubemapBakeArtifactError> {
    let descriptor = payload.descriptor();
    let pmrem = payload
        .pmrem_rgba16f_byte_range()
        .ok_or(SourceCubemapBakeArtifactError::MissingPmrem)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&descriptor.algorithm_version().to_le_bytes());
    hasher.update(&descriptor.face_size().to_le_bytes());
    hasher.update(&descriptor.mip_count().to_le_bytes());
    hasher.update(&payload.bytes()[pmrem]);
    Ok(hash_words(hasher.finalize()))
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

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 13;
    const HASHES_PER_SAMPLE: usize = 8_192;

    #[test]
    fn optimization_batch_20260830ex_runtime563_batches_artifact_hash_header() {
        let production = include_str!("source_cubemap_artifact.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        assert!(production.contains("BAKE_ARTIFACT_PAYLOAD_HASH_HEADER_BYTES"));
        assert!(production.contains("hasher.update(&header)"));
        assert!(!production.contains("update_u32_array_hash"));

        let descriptor = benchmark_descriptor();
        let payload = benchmark_payload();
        assert_eq!(
            legacy_artifact_hash(descriptor, &payload),
            optimized_artifact_hash(descriptor, &payload)
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_20260830ex_runtime563_artifact_hash_header_benchmark() {
        let descriptor = benchmark_descriptor();
        let payload = benchmark_payload();
        for _ in 0..3 {
            black_box(measure_artifact_hash(descriptor, &payload, false));
            black_box(measure_artifact_hash(descriptor, &payload, true));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_artifact_hash(descriptor, &payload, false));
                optimized_samples.push(measure_artifact_hash(descriptor, &payload, true));
            } else {
                optimized_samples.push(measure_artifact_hash(descriptor, &payload, true));
                legacy_samples.push(measure_artifact_hash(descriptor, &payload, false));
            }
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME563_ARTIFACT_HASH_HEADER_BATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} hashes_per_sample={HASHES_PER_SAMPLE} payload_bytes={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20",
            payload.len()
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(80) / 100,
            "batched artifact hash header must reduce P95 by at least 20%"
        );
    }

    fn benchmark_descriptor() -> IblBakeArtifactDescriptor {
        IblBakeArtifactDescriptor::current(
            IblBakeKey {
                source_kind: 7,
                source_revision: 19,
                horizon_color: [1, 2, 3, 4],
                zenith_color: [5, 6, 7, 8],
                ground_color: [9, 10, 11, 12],
                source_hash: [13, 14, 15, 16],
            },
            128,
            8,
            IblBakeArtifactContents::PMREM_SH9_IEM,
        )
    }

    fn benchmark_payload() -> Vec<u8> {
        (0..4_096).map(|index| index as u8).collect()
    }

    fn optimized_artifact_hash(
        descriptor: IblBakeArtifactDescriptor,
        payload: &[u8],
    ) -> blake3::Hash {
        let header = bake_artifact_payload_hash_header(descriptor);
        let mut hasher = blake3::Hasher::new();
        hasher.update(&header);
        hasher.update(payload);
        hasher.finalize()
    }

    fn legacy_artifact_hash(descriptor: IblBakeArtifactDescriptor, payload: &[u8]) -> blake3::Hash {
        let bake_key = descriptor.bake_key();
        let mut hasher = blake3::Hasher::new();
        hasher.update(&bake_key.source_kind.to_le_bytes());
        hasher.update(&bake_key.source_revision.to_le_bytes());
        for values in [
            bake_key.horizon_color,
            bake_key.zenith_color,
            bake_key.ground_color,
            bake_key.source_hash,
        ] {
            for value in values {
                hasher.update(&value.to_le_bytes());
            }
        }
        hasher.update(&descriptor.algorithm_version().to_le_bytes());
        hasher.update(&(descriptor.producer() as u32).to_le_bytes());
        hasher.update(&descriptor.source_face_size().to_le_bytes());
        hasher.update(&descriptor.source_mip_count().to_le_bytes());
        hasher.update(&descriptor.face_size().to_le_bytes());
        hasher.update(&descriptor.mip_count().to_le_bytes());
        hasher.update(&descriptor.contents().bits().to_le_bytes());
        hasher.update(payload);
        hasher.finalize()
    }

    fn measure_artifact_hash(
        descriptor: IblBakeArtifactDescriptor,
        payload: &[u8],
        optimized: bool,
    ) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_u8;
        for index in 0..HASHES_PER_SAMPLE {
            let hash = if optimized {
                optimized_artifact_hash(descriptor, black_box(payload))
            } else {
                legacy_artifact_hash(descriptor, black_box(payload))
            };
            checksum ^= hash.as_bytes()[index & 31];
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }
}
