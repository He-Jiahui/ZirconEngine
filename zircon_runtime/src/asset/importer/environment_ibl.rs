use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use std::{fs, path::Path};

use super::{decode_texture_source_image_rgba32f, AssetImportContext, DecodedTextureImageRgba32F};
use crate::asset::artifact::{
    IblBakeArtifactAssetDerivedRead, IblSourceCubemapStagingError, IblSourceCubemapStagingRead,
    IblSourceCubemapStagingStore,
};
use crate::asset::assets::{
    decode_external_source_cubemap, external_source_cubemap_container_info,
    ExternalSourceCubemapDecodeError, TextureAsset, TexturePayload, ZcubeSourceCubemap,
};
use crate::asset::AssetUri;
use crate::core::framework::render::{
    build_source_cubemap_irradiance_cube,
    build_source_cubemap_irradiance_cube_with_parallel_executor,
    rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_parallel_executor_and_timing,
    rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_timing,
    source_cubemap_face_size_from_equirect_height, source_cubemap_irradiance_mip_level,
    source_cubemap_mip_count, source_cubemap_mip_size, IblBakeArtifactContents,
    IblBakeArtifactRequest, IblBakeKey, SourceCubemapBuildTiming, SourceCubemapEnvironment,
    SourceCubemapIrradianceCube, SourceCubemapMipChain, SourceCubemapPrefilterQuality,
    SOURCE_CUBEMAP_FACE_COUNT,
};
use crate::core::framework::tasks::ParallelSliceExecutor;

mod import_settings;
mod source_identity;
mod source_staging;

use import_settings::{
    environment_ibl_import_mode, requested_artifact_contents,
    requested_artifact_contents_from_value, requested_face_size, requested_pmrem_layout,
    EnvironmentIblImportMode,
};
pub use import_settings::{
    ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING, ENVIRONMENT_IBL_IMPORT_SETTING,
    ENVIRONMENT_IBL_IRRADIANCE_CUBE_IMPORT_SETTING, ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING,
};
use source_identity::derive_source_identity;
use source_staging::EnvironmentIblSourceStagingParallelWorkItems;
pub use source_staging::{
    EnvironmentIblSourceStagingError, EnvironmentIblSourceStagingOutput,
    EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingStatus,
    EnvironmentIblSourceStagingTiming,
};

struct MeasuredParallelSliceExecutor<'a, E> {
    inner: &'a E,
    work_items: &'a AtomicUsize,
}

impl<E> ParallelSliceExecutor for MeasuredParallelSliceExecutor<'_, E>
where
    E: ParallelSliceExecutor,
{
    fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
    where
        T: Send,
        F: Fn(&mut [T]) + Send + Sync,
    {
        let chunk_size = chunk_size.max(1);
        self.work_items
            .fetch_add(items.len().div_ceil(chunk_size), Ordering::Relaxed);
        self.inner.parallel_for(items, chunk_size, task);
    }
}

/// A validated source and derived IBL bundle restored without decoding source pixels.
pub struct EnvironmentIblSourceStagingRestore {
    environment: SourceCubemapEnvironment,
    report: EnvironmentIblSourceStagingReport,
}

impl EnvironmentIblSourceStagingRestore {
    pub fn environment(&self) -> &SourceCubemapEnvironment {
        &self.environment
    }

    pub fn into_environment(self) -> SourceCubemapEnvironment {
        self.environment
    }

    pub fn report(&self) -> &EnvironmentIblSourceStagingReport {
        &self.report
    }
}

/// Builds the canonical request from source bytes and known equirectangular dimensions.
///
/// This is intentionally shared by normal staging and cache restoration so that warm-cache
/// probing cannot diverge from the importer-owned artifact identity.
pub fn environment_ibl_request_for_dimensions(
    context: &AssetImportContext,
    width: u32,
    height: u32,
) -> Result<Option<IblBakeArtifactRequest>, EnvironmentIblSourceStagingError> {
    let mode = environment_ibl_import_mode(context)?;
    if mode == EnvironmentIblImportMode::Disabled || !mode.applies_to(context) {
        return Ok(None);
    }
    if height.checked_mul(2) != Some(width) {
        if mode == EnvironmentIblImportMode::Automatic {
            return Ok(None);
        }
        return Err(
            EnvironmentIblSourceStagingError::InvalidEquirectangularDimensions { width, height },
        );
    }

    let natural_face_size = source_cubemap_face_size_from_equirect_height(height);
    let face_size = requested_face_size(context, natural_face_size)?;
    let source_mip_count = source_cubemap_mip_count(face_size);
    let (pmrem_face_size, pmrem_mip_count) = requested_pmrem_layout(context, face_size)?;
    let source_identity =
        derive_source_identity(&context.source_bytes, face_size, source_mip_count);
    let required_contents = requested_artifact_contents(context)?;

    Ok(Some(
        IblBakeArtifactRequest::new(
            IblBakeKey::source_cubemap(source_identity.revision(), source_identity.hash_words()),
            face_size,
            source_mip_count,
        )
        .with_pmrem_layout(pmrem_face_size, pmrem_mip_count)
        .with_required_contents(required_contents),
    ))
}

/// Restores a current source and derived IBL bundle without decoding source pixels.
///
/// A missing or rejected cache entry is a rebuildable miss and returns `None`.
/// A derived payload that fails application is deleted before the fallback so normal staging
/// sees a source-only bundle. Filesystem and request-layout failures remain visible errors.
pub fn restore_environment_ibl_source_if_current(
    context: &AssetImportContext,
    cache_root: impl AsRef<Path>,
    equirectangular_width: u32,
    equirectangular_height: u32,
) -> Result<Option<EnvironmentIblSourceStagingRestore>, EnvironmentIblSourceStagingError> {
    let Some(request) = environment_ibl_request_for_dimensions(
        context,
        equirectangular_width,
        equirectangular_height,
    )?
    else {
        return Ok(None);
    };
    let store = IblSourceCubemapStagingStore::new(cache_root.as_ref());
    let source_zcube_path = store.source_cubemap_path(&request);
    let asset_derived_path = store.asset_derived_store().asset_derived_path(&request);
    let environment = match store.read_source_cubemap_environment(&request, context.uri.clone()) {
        Ok(environment) => environment,
        Err(error) => {
            recover_source_restore_error(error, &asset_derived_path)?;
            return Ok(None);
        }
    };
    let output = EnvironmentIblSourceStagingOutput::from_reused_paths(
        &source_zcube_path,
        &asset_derived_path,
    )?;

    Ok(Some(EnvironmentIblSourceStagingRestore {
        environment,
        report: EnvironmentIblSourceStagingReport::current(
            EnvironmentIblSourceStagingStatus::Reused,
            request,
            source_zcube_path,
            asset_derived_path,
            EnvironmentIblSourceStagingTiming::default(),
            output,
        ),
    }))
}

fn source_restore_is_rebuildable_cache_miss(error: &IblSourceCubemapStagingError) -> bool {
    matches!(
        error,
        IblSourceCubemapStagingError::MissingSourceCubemap
            | IblSourceCubemapStagingError::MissingAssetDerived
            | IblSourceCubemapStagingError::RejectedAssetDerived(_)
            | IblSourceCubemapStagingError::DecodeZcube { .. }
    )
}

fn recover_source_restore_error(
    error: IblSourceCubemapStagingError,
    asset_derived_path: &Path,
) -> Result<(), EnvironmentIblSourceStagingError> {
    match error {
        IblSourceCubemapStagingError::ApplyAssetDerived(_) => {
            remove_invalid_asset_derived(asset_derived_path)
        }
        error if source_restore_is_rebuildable_cache_miss(&error) => Ok(()),
        error => Err(EnvironmentIblSourceStagingError::Stage(error)),
    }
}

fn remove_invalid_asset_derived(path: &Path) -> Result<(), EnvironmentIblSourceStagingError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(EnvironmentIblSourceStagingError::RemoveAssetDerived {
            path: path.to_path_buf(),
            source,
        }),
    }
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
    let decode_started = Instant::now();
    let image = decode_texture_source_image_rgba32f(context)
        .map_err(EnvironmentIblSourceStagingError::Decode)?;
    let timing = EnvironmentIblSourceStagingTiming {
        source_decode: decode_started.elapsed(),
        ..Default::default()
    };
    stage_environment_ibl_source_with_builder(
        context,
        cache_root,
        image,
        timing,
        None,
        |image, face_size, pmrem_face_size, pmrem_mip_count| {
            SourceCubemapMipChain::from_equirect_with_pmrem_layout_and_timing(
                face_size,
                pmrem_face_size,
                pmrem_mip_count,
                SourceCubemapPrefilterQuality::Normal,
                |u, v| sample_equirect_bilinear(image, u, v),
            )
        },
        |face_size, mip_count, source_texels, pmrem_face_size, pmrem_mip_count| {
            rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_timing(
                face_size,
                mip_count,
                source_texels,
                pmrem_face_size,
                pmrem_mip_count,
                SourceCubemapPrefilterQuality::Normal,
            )
        },
        build_source_cubemap_irradiance_cube,
    )
}

/// Stages an equirectangular environment through the caller-owned runtime task executor.
///
/// The importer retains cache and artifact ownership; callers provide only the execution owner.
pub fn stage_environment_ibl_source_with_parallel_executor<E>(
    context: &AssetImportContext,
    cache_root: impl AsRef<Path>,
    parallel_executor: &E,
) -> Result<EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingError>
where
    E: ParallelSliceExecutor,
{
    let mode = environment_ibl_import_mode(context)?;
    if mode == EnvironmentIblImportMode::Disabled || !mode.applies_to(context) {
        return Ok(EnvironmentIblSourceStagingReport::skipped());
    }
    let decode_started = Instant::now();
    let image = decode_texture_source_image_rgba32f(context)
        .map_err(EnvironmentIblSourceStagingError::Decode)?;
    let timing = EnvironmentIblSourceStagingTiming {
        source_decode: decode_started.elapsed(),
        ..Default::default()
    };
    stage_environment_ibl_source_with_parallel_executor_and_decoded_image_with_timing(
        context,
        cache_root,
        image,
        parallel_executor,
        timing,
    )
}

/// Stages an equirectangular environment from caller-provided linear HDR pixels.
///
/// This avoids a second source decode when a runtime viewer already decoded the image for
/// exposure or layout inspection. The request key and cache validation still derive from the
/// original import context bytes and settings.
pub fn stage_environment_ibl_source_with_parallel_executor_and_decoded_image<E>(
    context: &AssetImportContext,
    cache_root: impl AsRef<Path>,
    image: DecodedTextureImageRgba32F,
    parallel_executor: &E,
) -> Result<EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingError>
where
    E: ParallelSliceExecutor,
{
    stage_environment_ibl_source_with_parallel_executor_and_decoded_image_with_timing(
        context,
        cache_root,
        image,
        parallel_executor,
        EnvironmentIblSourceStagingTiming::default(),
    )
}

fn stage_environment_ibl_source_with_parallel_executor_and_decoded_image_with_timing<E>(
    context: &AssetImportContext,
    cache_root: impl AsRef<Path>,
    image: DecodedTextureImageRgba32F,
    parallel_executor: &E,
    timing: EnvironmentIblSourceStagingTiming,
) -> Result<EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingError>
where
    E: ParallelSliceExecutor,
{
    let irradiance_cube_work_items = AtomicUsize::new(0);
    let irradiance_cube_executor = MeasuredParallelSliceExecutor {
        inner: parallel_executor,
        work_items: &irradiance_cube_work_items,
    };
    stage_environment_ibl_source_with_builder(
        context,
        cache_root,
        image,
        timing,
        Some(&irradiance_cube_work_items),
        |image, face_size, pmrem_face_size, pmrem_mip_count| {
            SourceCubemapMipChain::from_equirect_with_pmrem_layout_and_parallel_executor_and_timing(
                face_size,
                pmrem_face_size,
                pmrem_mip_count,
                SourceCubemapPrefilterQuality::Normal,
                parallel_executor,
                |u, v| sample_equirect_bilinear(image, u, v),
            )
        },
        |face_size, mip_count, source_texels, pmrem_face_size, pmrem_mip_count| {
            rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_parallel_executor_and_timing(
                face_size,
                mip_count,
                source_texels,
                pmrem_face_size,
                pmrem_mip_count,
                SourceCubemapPrefilterQuality::Normal,
                parallel_executor,
            )
        },
        |cubemap| {
            build_source_cubemap_irradiance_cube_with_parallel_executor(
                cubemap,
                &irradiance_cube_executor,
            )
        },
    )
}

fn stage_environment_ibl_source_with_builder(
    context: &AssetImportContext,
    cache_root: impl AsRef<Path>,
    image: DecodedTextureImageRgba32F,
    mut timing: EnvironmentIblSourceStagingTiming,
    irradiance_cube_parallel_work_items: Option<&AtomicUsize>,
    build_cubemap: impl FnOnce(
        &DecodedTextureImageRgba32F,
        u32,
        u32,
        u32,
    ) -> (SourceCubemapMipChain, SourceCubemapBuildTiming),
    rebuild_cubemap: impl FnOnce(
        u32,
        u32,
        Vec<[f32; 4]>,
        u32,
        u32,
    ) -> (SourceCubemapMipChain, SourceCubemapBuildTiming),
    build_irradiance_cube: impl FnOnce(&SourceCubemapMipChain) -> SourceCubemapIrradianceCube,
) -> Result<EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingError> {
    let mode = environment_ibl_import_mode(context)?;
    if mode == EnvironmentIblImportMode::Disabled || !mode.applies_to(context) {
        return Ok(EnvironmentIblSourceStagingReport::skipped());
    }

    let Some(request) = environment_ibl_request_for_dimensions(context, image.width, image.height)?
    else {
        return Ok(EnvironmentIblSourceStagingReport::skipped());
    };
    let face_size = request.source_face_size();
    let source_mip_count = request.source_mip_count();
    let pmrem_face_size = request.pmrem_face_size();
    let pmrem_mip_count = request.pmrem_mip_count();
    let required_contents = request.required_contents();
    let store = IblSourceCubemapStagingStore::new(cache_root.as_ref());
    let source_zcube_path = store.source_cubemap_path(&request);
    let asset_derived_path = store.asset_derived_store().asset_derived_path(&request);

    let staged_source = match staged_bundle_state(&store, &request, &context.uri)? {
        EnvironmentIblStagedBundleState::Current => {
            let output = EnvironmentIblSourceStagingOutput::from_reused_paths(
                &source_zcube_path,
                &asset_derived_path,
            )?;
            return Ok(EnvironmentIblSourceStagingReport::current(
                EnvironmentIblSourceStagingStatus::Reused,
                request,
                source_zcube_path,
                asset_derived_path,
                timing,
                output,
            ));
        }
        EnvironmentIblStagedBundleState::SourceOnly(source) => Some(source),
        EnvironmentIblStagedBundleState::Missing => None,
    };

    let source_was_reused = staged_source.is_some();
    let cubemap_started = Instant::now();
    let (cubemap, cubemap_timing) = if let Some(source) = staged_source {
        rebuild_cubemap(
            source.face_size(),
            source.mip_count(),
            source.into_texels(),
            pmrem_face_size,
            pmrem_mip_count,
        )
    } else {
        build_cubemap(&image, face_size, pmrem_face_size, pmrem_mip_count)
    };
    timing.cubemap_build = cubemap_started.elapsed();
    timing.equirect_projection = cubemap_timing.equirect_projection();
    timing.source_mip_build = cubemap_timing.source_mip_build();
    timing.pmrem_build = cubemap_timing.pmrem_build();
    timing.sh9_build = cubemap_timing.sh9_build();
    let irradiance_cube = required_contents
        .contains(IblBakeArtifactContents::IEM)
        .then(|| {
            let irradiance_started = Instant::now();
            let irradiance_cube = build_irradiance_cube(&cubemap);
            timing.irradiance_cube_build = irradiance_started.elapsed();
            irradiance_cube
        });
    let write_started = Instant::now();
    let parallel_work_items = EnvironmentIblSourceStagingParallelWorkItems {
        equirect_projection: cubemap_timing.equirect_projection_parallel_work_items(),
        source_mip_build: cubemap_timing.source_mip_build_parallel_work_items(),
        pmrem_build: cubemap_timing.pmrem_build_parallel_work_items(),
        irradiance_cube_build: irradiance_cube_parallel_work_items
            .map(|work_items| work_items.load(Ordering::Relaxed) as u64)
            .unwrap_or(0),
    };
    let output = write_environment_ibl_staged_outputs(
        &store,
        &request,
        context.uri.clone(),
        &cubemap,
        irradiance_cube.as_ref(),
        source_was_reused,
        parallel_work_items,
    )?;
    timing.bundle_write = write_started.elapsed();

    Ok(EnvironmentIblSourceStagingReport::current(
        EnvironmentIblSourceStagingStatus::Written,
        request,
        source_zcube_path,
        asset_derived_path,
        timing,
        output,
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
    let source_identity = derive_source_identity(bytes, info.face_size, info.mip_count);
    let request = IblBakeArtifactRequest::new(
        IblBakeKey::source_cubemap(source_identity.revision(), source_identity.hash_words()),
        info.face_size,
        info.mip_count,
    )
    .with_required_contents(IblBakeArtifactContents::PMREM_SH9);
    let store = IblSourceCubemapStagingStore::new(cache_root.as_ref());
    let source_zcube_path = store.source_cubemap_path(&request);
    let asset_derived_path = store.asset_derived_store().asset_derived_path(&request);

    let staged_source = match staged_bundle_state(&store, &request, &texture.uri)? {
        EnvironmentIblStagedBundleState::Current => {
            let output = EnvironmentIblSourceStagingOutput::from_reused_paths(
                &source_zcube_path,
                &asset_derived_path,
            )?;
            return Ok(EnvironmentIblSourceStagingReport::current(
                EnvironmentIblSourceStagingStatus::Reused,
                request,
                source_zcube_path,
                asset_derived_path,
                EnvironmentIblSourceStagingTiming::default(),
                output,
            ));
        }
        EnvironmentIblStagedBundleState::SourceOnly(source) => Some(source),
        EnvironmentIblStagedBundleState::Missing => None,
    };
    let source_was_reused = staged_source.is_some();
    let (cubemap, mut timing) = if let Some(source) = staged_source {
        let cubemap_started = Instant::now();
        let (cubemap, cubemap_timing) =
            rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_timing(
                source.face_size(),
                source.mip_count(),
                source.into_texels(),
                request.pmrem_face_size(),
                request.pmrem_mip_count(),
                SourceCubemapPrefilterQuality::Normal,
            );
        (
            cubemap,
            EnvironmentIblSourceStagingTiming {
                cubemap_build: cubemap_started.elapsed(),
                pmrem_build: cubemap_timing.pmrem_build(),
                sh9_build: cubemap_timing.sh9_build(),
                ..Default::default()
            },
        )
    } else {
        let decode_started = Instant::now();
        let cubemap = decode_external_source_cubemap(texture)
            .map_err(EnvironmentIblSourceStagingError::ExternalDecode)?
            .ok_or_else(|| {
                EnvironmentIblSourceStagingError::ExternalDecode(
                    ExternalSourceCubemapDecodeError::InvalidPayload {
                        kind: info.kind,
                        reason: "classified cubemap did not decode as an external source"
                            .to_string(),
                    },
                )
            })?;
        (
            cubemap,
            EnvironmentIblSourceStagingTiming {
                source_decode: decode_started.elapsed(),
                ..Default::default()
            },
        )
    };
    let write_started = Instant::now();
    let output = write_environment_ibl_staged_outputs(
        &store,
        &request,
        texture.uri.clone(),
        &cubemap,
        None,
        source_was_reused,
        EnvironmentIblSourceStagingParallelWorkItems::default(),
    )?;
    timing.bundle_write = write_started.elapsed();

    Ok(EnvironmentIblSourceStagingReport::current(
        EnvironmentIblSourceStagingStatus::Written,
        request,
        source_zcube_path,
        asset_derived_path,
        timing,
        output,
    ))
}

enum EnvironmentIblStagedBundleState {
    Current,
    SourceOnly(ZcubeSourceCubemap),
    Missing,
}

fn write_environment_ibl_staged_outputs(
    store: &IblSourceCubemapStagingStore,
    request: &IblBakeArtifactRequest,
    uri: AssetUri,
    cubemap: &SourceCubemapMipChain,
    irradiance_cube: Option<&SourceCubemapIrradianceCube>,
    source_was_reused: bool,
    parallel_work_items: EnvironmentIblSourceStagingParallelWorkItems,
) -> Result<EnvironmentIblSourceStagingOutput, EnvironmentIblSourceStagingError> {
    let irradiance_cube_source_sample_visits =
        irradiance_cube_source_sample_visits(cubemap, irradiance_cube);
    if source_was_reused {
        let asset_derived = store
            .asset_derived_store()
            .write_source_cubemap_asset_derived_artifact(request, cubemap, irradiance_cube)
            .map_err(EnvironmentIblSourceStagingError::WriteAssetDerived)?;
        return EnvironmentIblSourceStagingOutput::from_reused_source_and_written_asset(
            &store.source_cubemap_path(request),
            asset_derived.encoded_len(),
            parallel_work_items,
            irradiance_cube_source_sample_visits,
        );
    }

    let bundle = store
        .write_source_cubemap_staged_bundle(request, uri, cubemap, irradiance_cube)
        .map_err(EnvironmentIblSourceStagingError::Stage)?;
    Ok(EnvironmentIblSourceStagingOutput::from_written_bundle(
        &bundle,
        parallel_work_items,
        irradiance_cube_source_sample_visits,
    ))
}

fn irradiance_cube_source_sample_visits(
    cubemap: &SourceCubemapMipChain,
    irradiance_cube: Option<&SourceCubemapIrradianceCube>,
) -> u64 {
    let Some(irradiance_cube) = irradiance_cube else {
        return 0;
    };
    irradiance_cube_source_sample_visits_for_layout(
        cubemap.source_face_size(),
        cubemap.source_mip_count(),
        irradiance_cube.face_size(),
    )
}

fn irradiance_cube_source_sample_visits_for_layout(
    source_face_size: u32,
    source_mip_count: u32,
    irradiance_cube_face_size: u32,
) -> u64 {
    let source_mip = source_cubemap_irradiance_mip_level(source_face_size, source_mip_count);
    let source_face_size = source_cubemap_mip_size(source_face_size, source_mip);
    let source_texels = u64::from(source_face_size)
        .saturating_mul(u64::from(source_face_size))
        .saturating_mul(SOURCE_CUBEMAP_FACE_COUNT as u64);
    let output_texels = u64::from(irradiance_cube_face_size)
        .saturating_mul(u64::from(irradiance_cube_face_size))
        .saturating_mul(SOURCE_CUBEMAP_FACE_COUNT as u64);

    // Direct cosine IEM scans the selected source cubemap once per output texel.
    source_texels.saturating_mul(output_texels)
}

fn staged_bundle_state(
    store: &IblSourceCubemapStagingStore,
    request: &IblBakeArtifactRequest,
    uri: &AssetUri,
) -> Result<EnvironmentIblStagedBundleState, EnvironmentIblSourceStagingError> {
    let source = match store.read_source_cubemap_zcube(request, uri.clone()) {
        Ok(IblSourceCubemapStagingRead::Hit(source)) => source,
        Ok(IblSourceCubemapStagingRead::Missing)
        | Err(IblSourceCubemapStagingError::DecodeZcube { .. }) => {
            return Ok(EnvironmentIblStagedBundleState::Missing);
        }
        Err(error) => return Err(EnvironmentIblSourceStagingError::Stage(error)),
    };
    let derived = store
        .asset_derived_store()
        .read_asset_derived_artifact(request)
        .map_err(EnvironmentIblSourceStagingError::ReadAssetDerived)?;
    Ok(
        if matches!(derived, IblBakeArtifactAssetDerivedRead::Hit(_)) {
            EnvironmentIblStagedBundleState::Current
        } else {
            EnvironmentIblStagedBundleState::SourceOnly(source)
        },
    )
}

fn sample_equirect_bilinear(image: &DecodedTextureImageRgba32F, u: f32, v: f32) -> [f32; 4] {
    let width = image.width.max(1);
    let height = image.height.max(1);
    let texel_x = u.rem_euclid(1.0) * width as f32 - 0.5;
    // Clamp the pole before deriving interpolation weights, matching a
    // clamp-to-edge sampler instead of blending the first two HDR rows.
    let texel_y = (v * height as f32 - 0.5).clamp(0.0, height.saturating_sub(1) as f32);
    let x0 = texel_x.floor() as i32;
    let y0 = texel_y.floor() as i32;
    let tx = texel_x - texel_x.floor();
    let ty = texel_y - texel_y.floor();
    let x0 = x0.rem_euclid(width as i32) as u32;
    let x1 = (x0 + 1) % width;
    let y0 = y0 as u32;
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

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use super::{
        environment_ibl_request_for_dimensions, recover_source_restore_error,
        requested_artifact_contents_from_value, sample_equirect_bilinear,
        source_restore_is_rebuildable_cache_miss, DecodedTextureImageRgba32F,
        EnvironmentIblSourceStagingOutput, EnvironmentIblSourceStagingTiming,
        MeasuredParallelSliceExecutor,
    };
    use crate::core::framework::render::IblBakeArtifactContents;
    use crate::core::framework::tasks::ParallelSliceExecutor;

    const SOURCE: &str = include_str!("environment_ibl.rs");
    const SOURCE_STAGING_MODULE: &str = include_str!("environment_ibl/source_staging/mod.rs");

    #[test]
    fn source_staging_contract_is_isolated_from_import_orchestration() {
        assert!(SOURCE.contains("mod source_staging;"));
        assert!(SOURCE.contains("pub use source_staging::{"));
        for type_name in [
            "EnvironmentIblSourceStagingError",
            "EnvironmentIblSourceStagingOutput",
            "EnvironmentIblSourceStagingReport",
            "EnvironmentIblSourceStagingStatus",
            "EnvironmentIblSourceStagingTiming",
        ] {
            assert!(
                SOURCE_STAGING_MODULE.contains(type_name),
                "source staging module must retain `{type_name}`"
            );
            assert!(
                !SOURCE.contains(&format!("pub struct {type_name}"))
                    && !SOURCE.contains(&format!("pub enum {type_name}")),
                "entry orchestration must not redefine `{type_name}`"
            );
        }
    }

    #[test]
    fn environment_staging_defaults_to_pmrem_and_sh9_without_iem() {
        assert_eq!(
            requested_artifact_contents_from_value(None)
                .expect("omitted IEM setting should use the MVP artifact"),
            IblBakeArtifactContents::PMREM_SH9
        );
        assert_eq!(
            requested_artifact_contents_from_value(Some(&toml::Value::Boolean(false)))
                .expect("explicitly disabled IEM should use the MVP artifact"),
            IblBakeArtifactContents::PMREM_SH9
        );
    }

    #[test]
    fn environment_staging_requires_explicit_boolean_iem_opt_in() {
        assert_eq!(
            requested_artifact_contents_from_value(Some(&toml::Value::Boolean(true)))
                .expect("IEM opt-in should be accepted"),
            IblBakeArtifactContents::PMREM_SH9_IEM
        );
        assert!(
            requested_artifact_contents_from_value(Some(&toml::Value::String("true".into())))
                .is_err()
        );
    }

    #[test]
    fn known_dimensions_preserve_the_canonical_source_request_identity() {
        let context = crate::asset::importer::AssetImportContext::new(
            "sunset.hdr".into(),
            crate::asset::AssetUri::parse("res://environment/sunset.hdr")
                .expect("test URI should parse"),
            b"unchanged source bytes".to_vec(),
            "environment_ibl = true\nenvironment_ibl_face_size = 128\nenvironment_ibl_pmrem_face_size = 64"
                .parse()
                .expect("test settings should parse"),
        );
        let request = environment_ibl_request_for_dimensions(&context, 1024, 512)
            .expect("valid equirectangular dimensions should build a request")
            .expect("enabled IBL should retain a request");

        assert_eq!(request.source_face_size(), 128);
        assert_eq!(request.source_mip_count(), 8);
        assert_eq!(request.pmrem_face_size(), 64);
        assert_eq!(request.pmrem_mip_count(), 7);
        assert_eq!(
            request.required_contents(),
            IblBakeArtifactContents::PMREM_SH9
        );
    }

    #[test]
    fn canonical_request_rejects_saturated_equirectangular_dimensions() {
        let context = crate::asset::importer::AssetImportContext::new(
            "overflow.hdr".into(),
            crate::asset::AssetUri::parse("res://environment/overflow.hdr")
                .expect("test URI should parse"),
            b"unchanged source bytes".to_vec(),
            "environment_ibl = true"
                .parse()
                .expect("test settings should parse"),
        );

        assert!(environment_ibl_request_for_dimensions(&context, u32::MAX, u32::MAX).is_err());
    }

    #[test]
    fn source_restore_only_swallows_known_rebuildable_cache_misses() {
        use crate::asset::artifact::IblSourceCubemapStagingError;

        for error in [
            IblSourceCubemapStagingError::MissingSourceCubemap,
            IblSourceCubemapStagingError::MissingAssetDerived,
        ] {
            assert!(source_restore_is_rebuildable_cache_miss(&error));
        }
        let apply_error = IblSourceCubemapStagingError::ApplyAssetDerived(
            crate::core::framework::render::SourceCubemapBakeArtifactError::MissingPmrem,
        );
        assert!(!source_restore_is_rebuildable_cache_miss(&apply_error));
    }

    #[test]
    fn apply_asset_derived_restore_failure_removes_only_the_derived_artifact() {
        use crate::asset::artifact::IblSourceCubemapStagingError;
        use crate::core::framework::render::SourceCubemapBakeArtifactError;

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon-environment-ibl-apply-error-{}-{nonce}",
            std::process::id()
        ));
        let source_zcube_path = root.join("current.zcube");
        let asset_derived_path = root.join("broken.zribl");
        std::fs::create_dir_all(&root).expect("test cache directory should be created");
        std::fs::write(&source_zcube_path, b"current source cubemap")
            .expect("test source cubemap should be written");
        std::fs::write(&asset_derived_path, b"bad derived artifact")
            .expect("test derived artifact should be written");

        let recovery = recover_source_restore_error(
            IblSourceCubemapStagingError::ApplyAssetDerived(
                SourceCubemapBakeArtifactError::MissingPmrem,
            ),
            &asset_derived_path,
        );

        assert!(recovery.is_ok());
        assert!(
            !asset_derived_path.exists(),
            "the invalid derived artifact must be removed before fallback staging"
        );
        assert!(
            source_zcube_path.exists(),
            "the current source cubemap must remain available for derived-only rebuild"
        );
        std::fs::remove_dir_all(root).expect("test cache directory should be removed");
    }

    #[test]
    fn environment_equirect_bilinear_sampling_clamps_poles_to_edge_rows() {
        let image = DecodedTextureImageRgba32F {
            width: 2,
            height: 2,
            rgba: vec![
                [1.0, 0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
            ],
        };

        assert_eq!(
            sample_equirect_bilinear(&image, 0.25, 0.0),
            [1.0, 0.0, 0.0, 1.0]
        );
        assert_eq!(
            sample_equirect_bilinear(&image, 0.25, 1.0),
            [0.0, 1.0, 0.0, 1.0]
        );
    }

    #[test]
    fn environment_equirect_bilinear_sampling_wraps_the_horizontal_seam() {
        let image = DecodedTextureImageRgba32F {
            width: 2,
            height: 1,
            rgba: vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]],
        };

        assert_eq!(
            sample_equirect_bilinear(&image, 0.0, 0.5),
            [0.5, 0.5, 0.0, 1.0]
        );
        assert_eq!(
            sample_equirect_bilinear(&image, 1.0, 0.5),
            [0.5, 0.5, 0.0, 1.0]
        );
    }

    #[test]
    fn parallel_environment_staging_uses_its_executor_for_iem_bake() {
        let parallel_staging = SOURCE
            .split("pub fn stage_environment_ibl_source_with_parallel_executor_and_decoded_image")
            .nth(1)
            .expect("parallel environment staging entry point should exist")
            .split("fn stage_environment_ibl_source_with_builder")
            .next()
            .expect("parallel environment staging should end before its shared builder");

        assert!(
            parallel_staging.contains(
                "build_source_cubemap_irradiance_cube_with_parallel_executor(cubemap, &irradiance_cube_executor)"
            ),
            "parallel environment staging must keep optional IEM convolution on the caller executor"
        );
        assert!(parallel_staging.contains("MeasuredParallelSliceExecutor"));
    }

    #[test]
    fn derived_only_staging_reuses_the_existing_source_file() {
        let output_writer = SOURCE
            .split("fn write_environment_ibl_staged_outputs")
            .nth(1)
            .expect("shared staging output writer should exist")
            .split("fn staged_bundle_state")
            .next()
            .expect("output writer should end before staged bundle inspection");
        let reused_source_branch = output_writer
            .split("if source_was_reused")
            .nth(1)
            .expect("derived-only output branch should exist")
            .split("let bundle")
            .next()
            .expect("new-source bundle write should follow the reuse branch");

        assert!(reused_source_branch.contains("write_source_cubemap_asset_derived_artifact"));
        assert!(!reused_source_branch.contains("write_source_cubemap_staged_bundle"));
        assert!(output_writer.contains("write_source_cubemap_staged_bundle"));
    }

    #[derive(Default)]
    struct SerialParallelSliceExecutor;

    impl ParallelSliceExecutor for SerialParallelSliceExecutor {
        fn parallel_for<T, F>(&self, items: &mut [T], chunk_size: usize, task: F)
        where
            T: Send,
            F: Fn(&mut [T]) + Send + Sync,
        {
            for chunk in items.chunks_mut(chunk_size.max(1)) {
                task(chunk);
            }
        }
    }

    #[test]
    fn measured_parallel_executor_reports_submitted_chunk_work() {
        let inner = SerialParallelSliceExecutor;
        let work_items = AtomicUsize::new(0);
        let executor = MeasuredParallelSliceExecutor {
            inner: &inner,
            work_items: &work_items,
        };
        let mut values = [0_u32; 5];

        executor.parallel_for(&mut values, 2, |chunk| {
            for value in chunk {
                *value += 1;
            }
        });

        assert_eq!(values, [1; 5]);
        assert_eq!(work_items.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn environment_staging_reports_subphases_without_double_counting_them() {
        let builder = SOURCE
            .split("fn stage_environment_ibl_source_with_builder")
            .nth(1)
            .expect("shared environment staging builder should exist")
            .split("/// Convert a cmft-style")
            .next()
            .expect("shared environment staging builder should end before external import");

        for phase in [
            "equirect_projection",
            "source_mip_build",
            "pmrem_build",
            "sh9_build",
        ] {
            assert!(
                builder.contains(&format!("timing.{phase} = cubemap_timing.{phase}()")),
                "shared staging builder must copy {phase} from framework attribution"
            );
        }

        let timing = EnvironmentIblSourceStagingTiming {
            source_decode: Duration::from_millis(3),
            cubemap_build: Duration::from_millis(48),
            equirect_projection: Duration::from_millis(7),
            source_mip_build: Duration::from_millis(11),
            pmrem_build: Duration::from_millis(13),
            sh9_build: Duration::from_millis(17),
            irradiance_cube_build: Duration::from_millis(19),
            bundle_write: Duration::from_millis(23),
        };

        assert_eq!(timing.equirect_projection(), Duration::from_millis(7));
        assert_eq!(timing.source_mip_build(), Duration::from_millis(11));
        assert_eq!(timing.pmrem_build(), Duration::from_millis(13));
        assert_eq!(timing.sh9_build(), Duration::from_millis(17));
        let output = EnvironmentIblSourceStagingOutput {
            source_zcube_bytes: 1_024,
            asset_derived_bytes: 2_048,
            equirect_projection_parallel_work_items: 6,
            source_mip_build_parallel_work_items: 12,
            pmrem_build_parallel_work_items: 24,
            irradiance_cube_build_parallel_work_items: 0,
            irradiance_cube_source_sample_visits: 37_748_736,
        };
        assert_eq!(output.source_zcube_bytes(), 1_024);
        assert_eq!(output.asset_derived_bytes(), 2_048);
        assert_eq!(output.parallel_executor_work_items(), 42);
        assert_eq!(output.equirect_projection_parallel_work_items(), 6);
        assert_eq!(output.source_mip_build_parallel_work_items(), 12);
        assert_eq!(output.pmrem_build_parallel_work_items(), 24);
        assert_eq!(output.irradiance_cube_build_parallel_work_items(), 0);
        assert_eq!(
            output.irradiance_cube_source_sample_visits(),
            37_748_736,
            "direct IEM throughput must retain its actual source-sample visit count"
        );
        assert_eq!(
            super::irradiance_cube_source_sample_visits_for_layout(64, 7, 32),
            37_748_736,
            "the canonical 32x32 source mip must report every direct IEM visit"
        );
        assert_eq!(
            super::irradiance_cube_source_sample_visits_for_layout(16, 5, 32),
            9_437_184,
            "a source below the diffuse cap must not be reported as a 32x32 source"
        );
        assert_eq!(
            timing.total(),
            Duration::from_millis(93),
            "cubemap_build owns its diagnostic subphases and total must not add them again"
        );
    }
}
