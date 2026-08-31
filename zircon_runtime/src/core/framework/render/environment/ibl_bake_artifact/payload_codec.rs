use std::ops::Range;

use super::super::{
    source_cubemap_sample_count, SourceCubemapIrradianceSh9, SOURCE_CUBEMAP_FACE_COUNT,
    SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT, SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE,
};
use super::{
    IblBakeArtifactContents, IblBakeArtifactDescriptor, IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES,
    IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ArtifactPayloadRanges {
    pub(super) pmrem: Option<Range<usize>>,
    pub(super) sh9: Option<Range<usize>>,
    pub(super) iem: Option<Range<usize>>,
    pub(super) total_size: usize,
}

pub(super) fn artifact_payload_ranges(
    descriptor: IblBakeArtifactDescriptor,
) -> ArtifactPayloadRanges {
    let mut cursor = 0;
    let pmrem = if descriptor
        .contents()
        .contains(IblBakeArtifactContents::PMREM)
    {
        let byte_len = source_cubemap_sample_count(descriptor.face_size(), descriptor.mip_count())
            * IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES;
        let range = cursor..cursor + byte_len;
        cursor += byte_len;
        Some(range)
    } else {
        None
    };

    let sh9 = if descriptor.contents().contains(IblBakeArtifactContents::SH9) {
        let range = cursor..cursor + IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES;
        cursor += IBL_BAKE_ARTIFACT_SH9_SIZE_BYTES;
        Some(range)
    } else {
        None
    };

    let iem = if descriptor.contents().contains(IblBakeArtifactContents::IEM) {
        let texel_count = SOURCE_CUBEMAP_FACE_COUNT
            * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize
            * SOURCE_CUBEMAP_IRRADIANCE_CUBE_FACE_SIZE as usize;
        let byte_len = texel_count * IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES;
        let range = cursor..cursor + byte_len;
        cursor += byte_len;
        Some(range)
    } else {
        None
    };

    ArtifactPayloadRanges {
        pmrem,
        sh9,
        iem,
        total_size: cursor,
    }
}

pub(super) fn push_sh9(bytes: &mut Vec<u8>, coefficients: &SourceCubemapIrradianceSh9) {
    for coefficient in coefficients {
        for channel in *coefficient {
            bytes.extend_from_slice(&channel.to_le_bytes());
        }
    }
}

pub(super) fn decode_sh9(bytes: &[u8]) -> SourceCubemapIrradianceSh9 {
    let mut coefficients = [[0.0; 4]; SOURCE_CUBEMAP_IRRADIANCE_COEFFICIENT_COUNT];
    let mut cursor = 0;
    for coefficient in &mut coefficients {
        for channel in coefficient {
            *channel = f32::from_le_bytes(read_dynamic_bytes::<4>(bytes, &mut cursor));
        }
    }
    coefficients
}

fn read_dynamic_bytes<const N: usize>(bytes: &[u8], cursor: &mut usize) -> [u8; N] {
    let mut value = [0; N];
    let next = *cursor + N;
    value.copy_from_slice(&bytes[*cursor..next]);
    *cursor = next;
    value
}
