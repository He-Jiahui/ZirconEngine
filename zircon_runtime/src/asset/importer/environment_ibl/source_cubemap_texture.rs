use std::path::Path;
use std::time::Instant;

use crate::asset::artifact::{IblSourceCubemapStagingStore, IblSourceImageIdentity};
use crate::asset::assets::{
    decode_external_source_cubemap_texels, decode_zcube_source_cubemap_texture,
    external_source_cubemap_container_info, zcube_source_cubemap_texture_info,
    ExternalSourceCubemapContainerInfo, ExternalSourceCubemapContainerKind, TextureAsset,
    TexturePayload,
};
use crate::core::framework::render::{
    rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_timing, IblBakeArtifactContents,
    IblBakeArtifactRequest, IblBakeKey, SourceCubemapPrefilterQuality,
};

use super::source_identity::derive_source_identity;
use super::{
    prepare_environment_ibl_staged_outputs, staged_bundle_state, EnvironmentIblSourceStagingError,
    EnvironmentIblSourceStagingOutput, EnvironmentIblSourceStagingParallelWorkItems,
    EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingStatus,
    EnvironmentIblSourceStagingTiming, EnvironmentIblStagedBundleState, EnvironmentIblStagingPhase,
    PreparedEnvironmentIblSourceStaging,
};

const SOURCE_INPUT_FORMAT_ZCUBE: u32 = 0x1000_0001;
const SOURCE_INPUT_FORMAT_DDS: u32 = 0x1000_0002;
const SOURCE_INPUT_FORMAT_KTX1: u32 = 0x1000_0003;
const SOURCE_INPUT_FORMAT_KTX2: u32 = 0x1000_0004;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SourceCubemapTextureKind {
    CapturedZcube { face_size: u32, mip_count: u32 },
    External(ExternalSourceCubemapContainerInfo),
}

impl SourceCubemapTextureKind {
    fn face_size(&self) -> u32 {
        match self {
            Self::CapturedZcube { face_size, .. } => *face_size,
            Self::External(info) => info.face_size,
        }
    }

    fn mip_count(&self) -> u32 {
        match self {
            Self::CapturedZcube { mip_count, .. } => *mip_count,
            Self::External(info) => info.mip_count,
        }
    }

    fn source_image_identity(&self) -> IblSourceImageIdentity {
        let format_identity = match self {
            Self::CapturedZcube { .. } => SOURCE_INPUT_FORMAT_ZCUBE,
            Self::External(info) => match info.kind {
                ExternalSourceCubemapContainerKind::Dds => SOURCE_INPUT_FORMAT_DDS,
                ExternalSourceCubemapContainerKind::Ktx1 => SOURCE_INPUT_FORMAT_KTX1,
                ExternalSourceCubemapContainerKind::Ktx2 => SOURCE_INPUT_FORMAT_KTX2,
            },
        };
        IblSourceImageIdentity::new(self.face_size(), self.face_size(), format_identity)
    }
}

pub(super) fn source_cubemap_texture_kind(
    texture: &TextureAsset,
) -> Result<Option<SourceCubemapTextureKind>, EnvironmentIblSourceStagingError> {
    if let Some(info) = zcube_source_cubemap_texture_info(texture)
        .map_err(EnvironmentIblSourceStagingError::SourceZcube)?
    {
        return Ok(Some(SourceCubemapTextureKind::CapturedZcube {
            face_size: info.face_size,
            mip_count: info.mip_count,
        }));
    }
    external_source_cubemap_container_info(texture)
        .map(|info| info.map(SourceCubemapTextureKind::External))
        .map_err(EnvironmentIblSourceStagingError::ExternalContainer)
}

/// Convert a cmft-style DDS/KTX source cubemap into Zircon source and derived artifacts.
pub fn stage_external_source_cubemap_texture(
    texture: &TextureAsset,
    cache_root: impl AsRef<Path>,
) -> Result<EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingError> {
    prepare_external_source_cubemap_texture(texture, cache_root)?.commit()
}

/// Build or reuse derived IBL for either a captured `.zcube` or supported DDS/KTX source cubemap.
pub fn stage_source_cubemap_texture(
    texture: &TextureAsset,
    cache_root: impl AsRef<Path>,
) -> Result<EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingError> {
    prepare_source_cubemap_texture(texture, cache_root)?.commit()
}

pub(crate) fn prepare_external_source_cubemap_texture(
    texture: &TextureAsset,
    cache_root: impl AsRef<Path>,
) -> Result<PreparedEnvironmentIblSourceStaging, EnvironmentIblSourceStagingError> {
    let classify_started = Instant::now();
    let info = {
        let _phase = EnvironmentIblStagingPhase::SourceClassify.enter();
        external_source_cubemap_container_info(texture)
            .map_err(EnvironmentIblSourceStagingError::ExternalContainer)?
    };
    let Some(info) = info else {
        return Ok(skipped_source_cubemap_texture(cache_root));
    };
    prepare_classified_source_cubemap_texture(
        texture,
        cache_root,
        SourceCubemapTextureKind::External(info),
        EnvironmentIblSourceStagingTiming {
            source_classify: classify_started.elapsed(),
            ..Default::default()
        },
    )
}

pub(crate) fn prepare_source_cubemap_texture(
    texture: &TextureAsset,
    cache_root: impl AsRef<Path>,
) -> Result<PreparedEnvironmentIblSourceStaging, EnvironmentIblSourceStagingError> {
    let classify_started = Instant::now();
    let kind = {
        let _phase = EnvironmentIblStagingPhase::SourceClassify.enter();
        source_cubemap_texture_kind(texture)?
    };
    let Some(kind) = kind else {
        return Ok(skipped_source_cubemap_texture(cache_root));
    };
    prepare_classified_source_cubemap_texture(
        texture,
        cache_root,
        kind,
        EnvironmentIblSourceStagingTiming {
            source_classify: classify_started.elapsed(),
            ..Default::default()
        },
    )
}

fn skipped_source_cubemap_texture(
    cache_root: impl AsRef<Path>,
) -> PreparedEnvironmentIblSourceStaging {
    PreparedEnvironmentIblSourceStaging {
        store: IblSourceCubemapStagingStore::new(cache_root.as_ref()),
        writes: Vec::new(),
        report: EnvironmentIblSourceStagingReport::skipped(),
    }
}

fn prepare_classified_source_cubemap_texture(
    texture: &TextureAsset,
    cache_root: impl AsRef<Path>,
    kind: SourceCubemapTextureKind,
    mut timing: EnvironmentIblSourceStagingTiming,
) -> Result<PreparedEnvironmentIblSourceStaging, EnvironmentIblSourceStagingError> {
    let TexturePayload::Container { bytes, .. } = &texture.payload else {
        return Ok(skipped_source_cubemap_texture(cache_root));
    };
    let source_image = kind.source_image_identity();
    let identity_started = Instant::now();
    let source_identity = {
        let _phase = EnvironmentIblStagingPhase::SourceIdentity.enter();
        derive_source_identity(bytes, kind.face_size(), kind.mip_count())
    };
    timing.source_identity = identity_started.elapsed();
    let request = IblBakeArtifactRequest::new(
        IblBakeKey::source_cubemap(source_identity.revision(), source_identity.hash_words()),
        kind.face_size(),
        kind.mip_count(),
    )
    .with_required_contents(IblBakeArtifactContents::PMREM_SH9);
    let store = IblSourceCubemapStagingStore::new(cache_root.as_ref());
    let source_zcube_path = store.source_cubemap_path(&request);
    let asset_derived_path = store.asset_derived_store().asset_derived_path(&request);

    let cache_probe_started = Instant::now();
    let staged_bundle = {
        let _phase = EnvironmentIblStagingPhase::CacheProbe.enter();
        staged_bundle_state(&store, &request, &texture.uri, source_image)?
    };
    timing.cache_probe = cache_probe_started.elapsed();
    let staged_source = match staged_bundle {
        EnvironmentIblStagedBundleState::Current => {
            let output = EnvironmentIblSourceStagingOutput::from_reused_paths(
                &source_zcube_path,
                &asset_derived_path,
            )?;
            return Ok(PreparedEnvironmentIblSourceStaging {
                store,
                writes: Vec::new(),
                report: EnvironmentIblSourceStagingReport::current(
                    EnvironmentIblSourceStagingStatus::Reused,
                    request,
                    source_zcube_path,
                    asset_derived_path,
                    timing,
                    output,
                ),
            });
        }
        EnvironmentIblStagedBundleState::SourceOnly(source) => Some(source),
        EnvironmentIblStagedBundleState::Missing => None,
    };
    let source_was_reused = staged_source.is_some();
    let (source_face_size, source_mip_count, source_texels) = if let Some(source) = staged_source {
        (source.face_size(), source.mip_count(), source.into_texels())
    } else {
        match &kind {
            SourceCubemapTextureKind::CapturedZcube { .. } => {
                let decode_started = Instant::now();
                let source = {
                    let _phase = EnvironmentIblStagingPhase::SourceDecode.enter();
                    decode_zcube_source_cubemap_texture(texture)
                        .map_err(EnvironmentIblSourceStagingError::SourceZcube)?
                };
                timing.source_decode = decode_started.elapsed();
                (source.face_size(), source.mip_count(), source.into_texels())
            }
            SourceCubemapTextureKind::External(info) => {
                let decode_started = Instant::now();
                let source_texels = {
                    let _phase = EnvironmentIblStagingPhase::SourceDecode.enter();
                    decode_external_source_cubemap_texels(texture, info)
                        .map_err(EnvironmentIblSourceStagingError::ExternalDecode)?
                };
                timing.source_decode = decode_started.elapsed();
                (info.face_size, info.mip_count, source_texels)
            }
        }
    };
    let cubemap_started = Instant::now();
    let (cubemap, cubemap_timing) = {
        let _phase = EnvironmentIblStagingPhase::CubemapBuild.enter();
        rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_timing(
            source_face_size,
            source_mip_count,
            source_texels,
            request.pmrem_face_size(),
            request.pmrem_mip_count(),
            SourceCubemapPrefilterQuality::Normal,
        )
    };
    timing.cubemap_build = cubemap_started.elapsed();
    timing.pmrem_build = cubemap_timing.pmrem_build();
    timing.sh9_build = cubemap_timing.sh9_build();
    let write_started = Instant::now();
    let (output, writes) = {
        let _phase = EnvironmentIblStagingPhase::BundleEncode.enter();
        prepare_environment_ibl_staged_outputs(
            &store,
            &request,
            texture.uri.clone(),
            &cubemap,
            None,
            source_was_reused,
            source_image,
            EnvironmentIblSourceStagingParallelWorkItems::default(),
        )?
    };
    timing.bundle_encode = write_started.elapsed();

    Ok(PreparedEnvironmentIblSourceStaging {
        store,
        writes,
        report: EnvironmentIblSourceStagingReport::current(
            EnvironmentIblSourceStagingStatus::Written,
            request,
            source_zcube_path,
            asset_derived_path,
            timing,
            output,
        ),
    })
}
