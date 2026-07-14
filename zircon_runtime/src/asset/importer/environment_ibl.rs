use std::path::{Path, PathBuf};

use thiserror::Error;

use super::{
    decode_texture_source_image_rgba32f, AssetImportContext, AssetImportError,
    DecodedTextureImageRgba32F,
};
use crate::asset::artifact::{
    IblBakeArtifactAssetDerivedError, IblBakeArtifactAssetDerivedRead,
    IblSourceCubemapStagingError, IblSourceCubemapStagingRead, IblSourceCubemapStagingStore,
};
use crate::asset::assets::{
    decode_external_source_cubemap, external_source_cubemap_container_info,
    ExternalSourceCubemapContainerError, ExternalSourceCubemapDecodeError, TextureAsset,
    TexturePayload,
};
use crate::asset::AssetUri;
use crate::core::framework::render::{
    build_source_cubemap_irradiance_cube, source_cubemap_face_size_from_equirect_height,
    source_cubemap_mip_count, IblBakeArtifactContents, IblBakeArtifactRequest, IblBakeKey,
    SourceCubemapMipChain, SourceCubemapPrefilterQuality, SOURCE_CUBEMAP_MAX_FACE_SIZE,
    SOURCE_CUBEMAP_MIN_FACE_SIZE, SOURCE_CUBEMAP_PMREM_FACE_SIZE, SOURCE_CUBEMAP_PMREM_MIP_COUNT,
};

pub const ENVIRONMENT_IBL_IMPORT_SETTING: &str = "environment_ibl";
pub const ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING: &str = "environment_ibl_face_size";
pub const ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING: &str = "environment_ibl_pmrem_face_size";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvironmentIblSourceStagingStatus {
    Skipped,
    Reused,
    Written,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentIblSourceStagingReport {
    status: EnvironmentIblSourceStagingStatus,
    request: Option<IblBakeArtifactRequest>,
    source_zcube_path: Option<PathBuf>,
    asset_derived_path: Option<PathBuf>,
}

impl EnvironmentIblSourceStagingReport {
    pub const fn status(&self) -> EnvironmentIblSourceStagingStatus {
        self.status
    }

    pub const fn request(&self) -> Option<&IblBakeArtifactRequest> {
        self.request.as_ref()
    }

    pub fn source_zcube_path(&self) -> Option<&Path> {
        self.source_zcube_path.as_deref()
    }

    pub fn asset_derived_path(&self) -> Option<&Path> {
        self.asset_derived_path.as_deref()
    }

    fn skipped() -> Self {
        Self {
            status: EnvironmentIblSourceStagingStatus::Skipped,
            request: None,
            source_zcube_path: None,
            asset_derived_path: None,
        }
    }

    fn current(
        status: EnvironmentIblSourceStagingStatus,
        request: IblBakeArtifactRequest,
        source_zcube_path: PathBuf,
        asset_derived_path: PathBuf,
    ) -> Self {
        Self {
            status,
            request: Some(request),
            source_zcube_path: Some(source_zcube_path),
            asset_derived_path: Some(asset_derived_path),
        }
    }
}

#[derive(Debug, Error)]
pub enum EnvironmentIblSourceStagingError {
    #[error("decode environment source image: {0}")]
    Decode(#[source] AssetImportError),
    #[error("environment IBL import setting `{key}` is invalid: {reason}")]
    InvalidSetting { key: &'static str, reason: String },
    #[error("environment IBL source must be a 2:1 equirectangular image, found {width}x{height}")]
    InvalidEquirectangularDimensions { width: u32, height: u32 },
    #[error("read environment IBL asset-derived artifact: {0}")]
    ReadAssetDerived(#[source] IblBakeArtifactAssetDerivedError),
    #[error("classify external source cubemap: {0}")]
    ExternalContainer(#[source] ExternalSourceCubemapContainerError),
    #[error("decode external source cubemap: {0}")]
    ExternalDecode(#[source] ExternalSourceCubemapDecodeError),
    #[error("stage environment IBL source bundle: {0}")]
    Stage(#[source] IblSourceCubemapStagingError),
}

/// Build or reuse the source `.zcube` and companion `.zribl` for an environment image.
///
/// HDR/EXR sources use automatic mode by default and are staged only when their
/// dimensions are 2:1. Other image formats can opt in with `environment_ibl = true`.
pub fn stage_environment_ibl_source(
    context: &AssetImportContext,
    cache_root: impl AsRef<Path>,
) -> Result<EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingError> {
    let mode = environment_ibl_import_mode(context)?;
    if mode == EnvironmentIblImportMode::Disabled || !mode.applies_to(context) {
        return Ok(EnvironmentIblSourceStagingReport::skipped());
    }

    let image = decode_texture_source_image_rgba32f(context)
        .map_err(EnvironmentIblSourceStagingError::Decode)?;
    if image.width != image.height.saturating_mul(2) {
        if mode == EnvironmentIblImportMode::Automatic {
            return Ok(EnvironmentIblSourceStagingReport::skipped());
        }
        return Err(
            EnvironmentIblSourceStagingError::InvalidEquirectangularDimensions {
                width: image.width,
                height: image.height,
            },
        );
    }

    let natural_face_size = source_cubemap_face_size_from_equirect_height(image.height);
    let face_size = requested_face_size(context, natural_face_size)?;
    let source_mip_count = source_cubemap_mip_count(face_size);
    let (pmrem_face_size, pmrem_mip_count) = requested_pmrem_layout(context, face_size)?;
    let source_hash = source_hash_words(&context.source_bytes, face_size, source_mip_count);
    let request = IblBakeArtifactRequest::new(
        IblBakeKey::source_cubemap(source_revision(&context.source_bytes), source_hash),
        face_size,
        source_mip_count,
    )
    .with_pmrem_layout(pmrem_face_size, pmrem_mip_count)
    .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
    let store = IblSourceCubemapStagingStore::new(cache_root.as_ref());
    let source_zcube_path = store.source_cubemap_path(&request);
    let asset_derived_path = store.asset_derived_store().asset_derived_path(&request);

    if staged_bundle_is_current(&store, &request, &context.uri)? {
        return Ok(EnvironmentIblSourceStagingReport::current(
            EnvironmentIblSourceStagingStatus::Reused,
            request,
            source_zcube_path,
            asset_derived_path,
        ));
    }

    let cubemap = SourceCubemapMipChain::from_equirect_with_pmrem_layout(
        face_size,
        pmrem_face_size,
        pmrem_mip_count,
        SourceCubemapPrefilterQuality::Normal,
        |u, v| sample_equirect_bilinear(&image, u, v),
    );
    let irradiance_cube = build_source_cubemap_irradiance_cube(&cubemap);
    store
        .write_source_cubemap_staged_bundle(
            &request,
            context.uri.clone(),
            &cubemap,
            Some(&irradiance_cube),
        )
        .map_err(EnvironmentIblSourceStagingError::Stage)?;

    Ok(EnvironmentIblSourceStagingReport::current(
        EnvironmentIblSourceStagingStatus::Written,
        request,
        source_zcube_path,
        asset_derived_path,
    ))
}

/// Convert a cmft-style DDS/KTX source cubemap into Zircon source and derived artifacts.
pub fn stage_external_source_cubemap_texture(
    texture: &TextureAsset,
    cache_root: impl AsRef<Path>,
) -> Result<EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingError> {
    let Some(info) = external_source_cubemap_container_info(texture)
        .map_err(EnvironmentIblSourceStagingError::ExternalContainer)?
    else {
        return Ok(EnvironmentIblSourceStagingReport::skipped());
    };
    let TexturePayload::Container { bytes, .. } = &texture.payload else {
        return Ok(EnvironmentIblSourceStagingReport::skipped());
    };
    let request = IblBakeArtifactRequest::new(
        IblBakeKey::source_cubemap(
            source_revision(bytes),
            source_hash_words(bytes, info.face_size, info.mip_count),
        ),
        info.face_size,
        info.mip_count,
    )
    .with_required_contents(IblBakeArtifactContents::PMREM_SH9_IEM);
    let store = IblSourceCubemapStagingStore::new(cache_root.as_ref());
    let source_zcube_path = store.source_cubemap_path(&request);
    let asset_derived_path = store.asset_derived_store().asset_derived_path(&request);

    if staged_bundle_is_current(&store, &request, &texture.uri)? {
        return Ok(EnvironmentIblSourceStagingReport::current(
            EnvironmentIblSourceStagingStatus::Reused,
            request,
            source_zcube_path,
            asset_derived_path,
        ));
    }

    let cubemap = decode_external_source_cubemap(texture)
        .map_err(EnvironmentIblSourceStagingError::ExternalDecode)?
        .ok_or_else(|| {
            EnvironmentIblSourceStagingError::ExternalDecode(
                ExternalSourceCubemapDecodeError::InvalidPayload {
                    kind: info.kind,
                    reason: "classified cubemap did not decode as an external source".to_string(),
                },
            )
        })?;
    let irradiance_cube = build_source_cubemap_irradiance_cube(&cubemap);
    store
        .write_source_cubemap_staged_bundle(
            &request,
            texture.uri.clone(),
            &cubemap,
            Some(&irradiance_cube),
        )
        .map_err(EnvironmentIblSourceStagingError::Stage)?;

    Ok(EnvironmentIblSourceStagingReport::current(
        EnvironmentIblSourceStagingStatus::Written,
        request,
        source_zcube_path,
        asset_derived_path,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnvironmentIblImportMode {
    Disabled,
    Automatic,
    Enabled,
}

impl EnvironmentIblImportMode {
    fn applies_to(self, context: &AssetImportContext) -> bool {
        if self == Self::Enabled {
            return true;
        }
        matches!(
            context
                .source_path
                .extension()
                .and_then(|extension| extension.to_str())
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("hdr" | "exr")
        )
    }
}

fn environment_ibl_import_mode(
    context: &AssetImportContext,
) -> Result<EnvironmentIblImportMode, EnvironmentIblSourceStagingError> {
    let Some(value) = context.import_settings.get(ENVIRONMENT_IBL_IMPORT_SETTING) else {
        return Ok(EnvironmentIblImportMode::Automatic);
    };
    if let Some(enabled) = value.as_bool() {
        return Ok(if enabled {
            EnvironmentIblImportMode::Enabled
        } else {
            EnvironmentIblImportMode::Disabled
        });
    }
    if value
        .as_str()
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("auto"))
    {
        return Ok(EnvironmentIblImportMode::Automatic);
    }
    Err(EnvironmentIblSourceStagingError::InvalidSetting {
        key: ENVIRONMENT_IBL_IMPORT_SETTING,
        reason: "expected true, false, or \"auto\"".to_string(),
    })
}

fn requested_face_size(
    context: &AssetImportContext,
    natural_face_size: u32,
) -> Result<u32, EnvironmentIblSourceStagingError> {
    let Some(value) = context
        .import_settings
        .get(ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING)
    else {
        return Ok(natural_face_size);
    };
    let Some(value) = value.as_integer() else {
        return Err(EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING,
            reason: "expected an integer power-of-two face size".to_string(),
        });
    };
    let face_size = u32::try_from(value).map_err(|_| {
        EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING,
            reason: format!("face size must be in {SOURCE_CUBEMAP_MIN_FACE_SIZE}..={SOURCE_CUBEMAP_MAX_FACE_SIZE}"),
        }
    })?;
    if !face_size.is_power_of_two()
        || !(SOURCE_CUBEMAP_MIN_FACE_SIZE..=SOURCE_CUBEMAP_MAX_FACE_SIZE).contains(&face_size)
    {
        return Err(EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING,
            reason: format!(
                "face size must be a power of two in {SOURCE_CUBEMAP_MIN_FACE_SIZE}..={SOURCE_CUBEMAP_MAX_FACE_SIZE}"
            ),
        });
    }
    Ok(face_size.min(natural_face_size))
}

fn requested_pmrem_layout(
    context: &AssetImportContext,
    source_face_size: u32,
) -> Result<(u32, u32), EnvironmentIblSourceStagingError> {
    let Some(value) = context
        .import_settings
        .get(ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING)
    else {
        return Ok((
            SOURCE_CUBEMAP_PMREM_FACE_SIZE,
            SOURCE_CUBEMAP_PMREM_MIP_COUNT,
        ));
    };
    let Some(value) = value.as_integer() else {
        return Err(EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING,
            reason: "expected an integer power-of-two face size".to_string(),
        });
    };
    let face_size = u32::try_from(value).map_err(|_| {
        EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING,
            reason: format!(
                "face size must be in {SOURCE_CUBEMAP_MIN_FACE_SIZE}..={SOURCE_CUBEMAP_MAX_FACE_SIZE}"
            ),
        }
    })?;
    if !face_size.is_power_of_two()
        || !(SOURCE_CUBEMAP_MIN_FACE_SIZE..=SOURCE_CUBEMAP_MAX_FACE_SIZE).contains(&face_size)
    {
        return Err(EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING,
            reason: format!(
                "face size must be a power of two in {SOURCE_CUBEMAP_MIN_FACE_SIZE}..={SOURCE_CUBEMAP_MAX_FACE_SIZE}"
            ),
        });
    }
    if face_size > source_face_size {
        return Err(EnvironmentIblSourceStagingError::InvalidSetting {
            key: ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING,
            reason: format!(
                "face size {face_size} must not exceed source face size {source_face_size}"
            ),
        });
    }
    Ok((face_size, source_cubemap_mip_count(face_size)))
}

fn staged_bundle_is_current(
    store: &IblSourceCubemapStagingStore,
    request: &IblBakeArtifactRequest,
    uri: &AssetUri,
) -> Result<bool, EnvironmentIblSourceStagingError> {
    let source_is_current = match store.read_source_cubemap_zcube(request, uri.clone()) {
        Ok(IblSourceCubemapStagingRead::Hit(_)) => true,
        Ok(IblSourceCubemapStagingRead::Missing)
        | Err(IblSourceCubemapStagingError::DecodeZcube { .. }) => false,
        Err(error) => return Err(EnvironmentIblSourceStagingError::Stage(error)),
    };
    if !source_is_current {
        return Ok(false);
    }

    let derived = store
        .asset_derived_store()
        .read_asset_derived_artifact(request)
        .map_err(EnvironmentIblSourceStagingError::ReadAssetDerived)?;
    Ok(matches!(derived, IblBakeArtifactAssetDerivedRead::Hit(_)))
}

fn sample_equirect_bilinear(image: &DecodedTextureImageRgba32F, u: f32, v: f32) -> [f32; 4] {
    let width = image.width.max(1);
    let height = image.height.max(1);
    let texel_x = u.rem_euclid(1.0) * width as f32 - 0.5;
    let texel_y = v.clamp(0.0, 1.0) * height as f32 - 0.5;
    let x0 = texel_x.floor() as i32;
    let y0 = texel_y.floor() as i32;
    let tx = texel_x - texel_x.floor();
    let ty = texel_y - texel_y.floor();
    let x0 = x0.rem_euclid(width as i32) as u32;
    let x1 = (x0 + 1) % width;
    let y0 = y0.clamp(0, height.saturating_sub(1) as i32) as u32;
    let y1 = (y0 + 1).min(height - 1);
    let c00 = image.rgba[image_index(width, x0, y0)];
    let c10 = image.rgba[image_index(width, x1, y0)];
    let c01 = image.rgba[image_index(width, x0, y1)];
    let c11 = image.rgba[image_index(width, x1, y1)];
    let mut output = [0.0; 4];
    for channel in 0..3 {
        output[channel] = sanitize_hdr_channel(lerp(
            lerp(c00[channel], c10[channel], tx),
            lerp(c01[channel], c11[channel], tx),
            ty,
        ));
    }
    output[3] = 1.0;
    output
}

fn image_index(width: u32, x: u32, y: u32) -> usize {
    y as usize * width as usize + x as usize
}

fn sanitize_hdr_channel(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 65_504.0)
    } else {
        0.0
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn source_hash_words(bytes: &[u8], face_size: u32, mip_count: u32) -> [u32; 4] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(bytes);
    hasher.update(&face_size.to_le_bytes());
    hasher.update(&mip_count.to_le_bytes());
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    std::array::from_fn(|index| {
        let offset = index * 4;
        u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("four-byte hash word"),
        )
    })
}

fn source_revision(bytes: &[u8]) -> u64 {
    let digest = blake3::hash(bytes);
    u64::from_le_bytes(
        digest.as_bytes()[..8]
            .try_into()
            .expect("eight-byte source revision"),
    )
    .max(1)
}

#[cfg(test)]
mod tests {
    use super::source_hash_words;

    #[test]
    fn environment_source_hash_tracks_imported_cubemap_layout() {
        let source = b"same HDR source bytes";
        assert_ne!(
            source_hash_words(source, 64, 7),
            source_hash_words(source, 128, 8)
        );
        assert_eq!(
            source_hash_words(source, 256, 9),
            source_hash_words(source, 256, 9)
        );
    }
}
