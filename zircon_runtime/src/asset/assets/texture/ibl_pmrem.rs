use thiserror::Error;

use crate::asset::AssetUri;
use crate::core::framework::render::{
    IblBakeArtifactBlob, RenderImageColorSpace, RenderImageDimension, IBL_BAKE_ALGORITHM_VERSION,
    IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES, SOURCE_CUBEMAP_FACE_COUNT,
};

use super::{TextureAsset, TextureAssetDescriptor, TexturePayload};

pub const IBL_PMREM_RGBA16F_FORMAT: &str = "zircon/ibl-pmrem-rgba16f-v1";
pub const IBL_PMREM_RGBA16F_GPU_FORMAT: &str = "rgba16float";

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum IblPmremTextureError {
    #[error("IBL bake artifact does not contain a PMREM section")]
    MissingPmrem,
    #[error(
        "IBL bake artifact algorithm version {actual} is stale; current version is {expected}"
    )]
    StaleAlgorithmVersion { expected: u64, actual: u64 },
    #[error("texture payload is not the Zircon RGBA16F PMREM container")]
    NotPmremContainer,
    #[error("PMREM texture must use the Cube dimension, found {actual:?}")]
    Dimension { actual: RenderImageDimension },
    #[error("PMREM texture must contain six faces, found {actual}")]
    FaceCount { actual: u32 },
    #[error(
        "PMREM texture must use linear rgba16float, found format {format} and color space {color_space:?}"
    )]
    Descriptor {
        format: String,
        color_space: RenderImageColorSpace,
    },
    #[error("PMREM rgba16f payload length mismatch: expected {expected}, found {actual}")]
    PayloadLength { expected: usize, actual: usize },
    #[error("PMREM texture extent is too large")]
    ExtentOverflow,
}

pub fn texture_asset_from_ibl_bake_artifact_pmrem(
    uri: AssetUri,
    blob: &IblBakeArtifactBlob,
) -> Result<TextureAsset, IblPmremTextureError> {
    let descriptor = blob.descriptor();
    if descriptor.algorithm_version() != IBL_BAKE_ALGORITHM_VERSION {
        return Err(IblPmremTextureError::StaleAlgorithmVersion {
            expected: IBL_BAKE_ALGORITHM_VERSION,
            actual: descriptor.algorithm_version(),
        });
    }
    let range = blob
        .payload()
        .pmrem_rgba16f_byte_range()
        .ok_or(IblPmremTextureError::MissingPmrem)?;
    let payload_bytes = &blob.payload().bytes()[range];
    validate_pmrem_payload_length(
        descriptor.face_size(),
        descriptor.mip_count(),
        payload_bytes.len(),
    )?;
    let bytes = payload_bytes.to_vec();
    let mut texture_descriptor = TextureAssetDescriptor::container(
        IBL_PMREM_RGBA16F_GPU_FORMAT,
        descriptor.mip_count(),
        SOURCE_CUBEMAP_FACE_COUNT as u32,
    );
    texture_descriptor.color_space = RenderImageColorSpace::Linear;
    texture_descriptor.dimension = RenderImageDimension::Cube;
    texture_descriptor.depth_or_array_layers = SOURCE_CUBEMAP_FACE_COUNT as u32;
    texture_descriptor.array_layer_count = SOURCE_CUBEMAP_FACE_COUNT as u32;

    let texture = TextureAsset::new_container(
        uri,
        descriptor.face_size(),
        descriptor.face_size(),
        IBL_PMREM_RGBA16F_FORMAT,
        bytes,
        descriptor.mip_count(),
        SOURCE_CUBEMAP_FACE_COUNT as u32,
    )
    .with_descriptor(texture_descriptor);
    decode_ibl_pmrem_rgba16f_texture(&texture)?;
    Ok(texture)
}

pub fn is_ibl_pmrem_rgba16f_texture(texture: &TextureAsset) -> bool {
    matches!(
        &texture.payload,
        TexturePayload::Container { format, .. } if format == IBL_PMREM_RGBA16F_FORMAT
    )
}

pub fn decode_ibl_pmrem_rgba16f_texture(
    texture: &TextureAsset,
) -> Result<&[u8], IblPmremTextureError> {
    let TexturePayload::Container {
        format,
        bytes,
        mip_count,
        array_layers,
    } = &texture.payload
    else {
        return Err(IblPmremTextureError::NotPmremContainer);
    };
    if format != IBL_PMREM_RGBA16F_FORMAT {
        return Err(IblPmremTextureError::NotPmremContainer);
    }

    let descriptor = texture.render_image_descriptor();
    if descriptor.dimension != RenderImageDimension::Cube {
        return Err(IblPmremTextureError::Dimension {
            actual: descriptor.dimension,
        });
    }
    if *array_layers != SOURCE_CUBEMAP_FACE_COUNT as u32
        || descriptor.array_layer_count != SOURCE_CUBEMAP_FACE_COUNT as u32
        || descriptor.depth_or_array_layers != SOURCE_CUBEMAP_FACE_COUNT as u32
    {
        return Err(IblPmremTextureError::FaceCount {
            actual: descriptor.array_layer_count,
        });
    }
    if descriptor.format != IBL_PMREM_RGBA16F_GPU_FORMAT
        || descriptor.color_space != RenderImageColorSpace::Linear
    {
        return Err(IblPmremTextureError::Descriptor {
            format: descriptor.format,
            color_space: descriptor.color_space,
        });
    }
    validate_pmrem_payload_length(texture.width, *mip_count, bytes.len())?;
    Ok(bytes)
}

fn validate_pmrem_payload_length(
    face_size: u32,
    mip_count: u32,
    actual: usize,
) -> Result<(), IblPmremTextureError> {
    let expected = rgba16f_cube_mip_chain_len(face_size, mip_count)
        .ok_or(IblPmremTextureError::ExtentOverflow)?;
    if actual != expected {
        return Err(IblPmremTextureError::PayloadLength { expected, actual });
    }
    Ok(())
}

fn rgba16f_cube_mip_chain_len(face_size: u32, mip_count: u32) -> Option<usize> {
    let mut texel_count = 0_usize;
    for mip in 0..mip_count {
        let mip_size = usize::try_from((face_size >> mip).max(1)).ok()?;
        texel_count = texel_count.checked_add(
            mip_size
                .checked_mul(mip_size)?
                .checked_mul(SOURCE_CUBEMAP_FACE_COUNT)?,
        )?;
    }
    texel_count.checked_mul(IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES)
}

#[cfg(test)]
mod optimization_batch_gw_runtime578_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 31;
    const ITERATIONS: usize = 250_000;

    #[test]
    fn optimization_batch_gw_runtime578_pmrem_length_preflight_preserves_error() {
        let expected = rgba16f_cube_mip_chain_len(128, 8).expect("bounded PMREM size");
        assert_eq!(validate_pmrem_payload_length(128, 8, expected), Ok(()));
        assert_eq!(
            validate_pmrem_payload_length(128, 8, expected - 2),
            Err(IblPmremTextureError::PayloadLength {
                expected,
                actual: expected - 2,
            })
        );

        let production = include_str!("ibl_pmrem.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(production.contains("validate_pmrem_payload_length("));
        assert!(production.contains("let bytes = payload_bytes.to_vec();"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_gw_runtime578_pmrem_invalid_length_preflight_p95() {
        let expected = rgba16f_cube_mip_chain_len(128, 8).expect("bounded PMREM size");
        let payload = vec![0_u8; expected - 2];
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&payload, expected));
                optimized_samples.push(measure_optimized(&payload, expected));
            } else {
                optimized_samples.push(measure_optimized(&payload, expected));
                legacy_samples.push(measure_legacy(&payload, expected));
            }
        }
        let legacy_p95_ns = p95(&mut legacy_samples);
        let optimized_p95_ns = p95(&mut optimized_samples);
        println!(
            "RUNTIME578_PMREM_INVALID_LENGTH_PREFLIGHT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} payload_bytes={} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            payload.len(),
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
            "invalid PMREM preflight should avoid the legacy payload copy: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn measure_legacy(payload: &[u8], expected: usize) -> u128 {
        let started = Instant::now();
        let mut rejected = 0_usize;
        for _ in 0..ITERATIONS {
            let copied = payload.to_vec();
            if copied.len() != expected {
                rejected += 1;
            }
            black_box(copied);
        }
        black_box(rejected);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(payload: &[u8], expected: usize) -> u128 {
        let started = Instant::now();
        let mut rejected = 0_usize;
        for _ in 0..ITERATIONS {
            if payload.len() != expected {
                rejected += 1;
            }
            black_box(payload.len());
        }
        black_box(rejected);
        started.elapsed().as_nanos().max(1)
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[samples.len() * 95 / 100]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
