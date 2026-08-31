use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use super::{
    decode_texture_source_image_metadata, decode_texture_source_image_rgba32f,
    texture_source_image_format_identity, AssetImportContext, DecodedTextureImageRgba32F,
    TextureSourceImageMetadata,
};
use crate::asset::artifact::{
    IblBakeArtifactAssetDerivedRead, IblSourceCubemapStagingError, IblSourceCubemapStagingRead,
    IblSourceCubemapStagingStore, IblSourceImageIdentity,
};
use crate::asset::assets::ZcubeSourceCubemap;
use crate::asset::AssetUri;
use crate::core::framework::render::{
    build_source_cubemap_irradiance_cube,
    build_source_cubemap_irradiance_cube_with_parallel_executor,
    rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_parallel_executor_and_timing,
    rebuild_source_cubemap_from_source_mips_with_pmrem_layout_and_timing,
    source_cubemap_face_size_from_equirect_height, source_cubemap_irradiance_mip_level,
    source_cubemap_mip_count, source_cubemap_mip_size, IblBakeArtifactContents,
    IblBakeArtifactRequest, IblBakeKey, SourceCubemapBuildTiming, SourceCubemapIrradianceCube,
    SourceCubemapMipChain, SourceCubemapPrefilterQuality, SOURCE_CUBEMAP_FACE_COUNT,
};
use crate::core::framework::tasks::ParallelSliceExecutor;
use crate::core::resource::io::transaction::PreparedFileWrite;

mod import_settings;
mod restore;
mod source_cubemap_texture;
mod source_identity;
mod source_staging;
mod warm_cache;

use import_settings::{
    environment_ibl_import_mode, requested_artifact_contents, requested_face_size,
    requested_pmrem_layout, EnvironmentIblImportMode,
};
pub use import_settings::{
    ENVIRONMENT_IBL_FACE_SIZE_IMPORT_SETTING, ENVIRONMENT_IBL_IMPORT_SETTING,
    ENVIRONMENT_IBL_IRRADIANCE_CUBE_IMPORT_SETTING, ENVIRONMENT_IBL_PMREM_FACE_SIZE_IMPORT_SETTING,
};
pub use restore::{restore_environment_ibl_source_if_current, EnvironmentIblSourceStagingRestore};
pub(crate) use source_cubemap_texture::{
    prepare_external_source_cubemap_texture, prepare_source_cubemap_texture,
};
pub use source_cubemap_texture::{
    stage_external_source_cubemap_texture, stage_source_cubemap_texture,
};
use source_identity::derive_source_identity_with_format;
use source_staging::EnvironmentIblSourceStagingParallelWorkItems;
use source_staging::EnvironmentIblStagingPhase;
pub use source_staging::{
    EnvironmentIblSourceStagingError, EnvironmentIblSourceStagingOutput,
    EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingStatus,
    EnvironmentIblSourceStagingTiming,
};
use warm_cache::{probe_environment_ibl_warm_cache, EnvironmentIblWarmCacheProbe};

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

/// Encoded IBL cache outputs that remain invisible until their publication owner commits them.
///
/// Project import collects these writes into its own durable generation transaction. Direct
/// importer callers retain the existing standalone IBL bundle transaction through `commit`.
pub(crate) struct PreparedEnvironmentIblSourceStaging {
    store: IblSourceCubemapStagingStore,
    writes: Vec<PreparedFileWrite>,
    report: EnvironmentIblSourceStagingReport,
}

impl PreparedEnvironmentIblSourceStaging {
    fn commit(self) -> Result<EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingError> {
        let Self {
            store,
            writes,
            mut report,
        } = self;
        let commit_started = Instant::now();
        {
            let _phase = EnvironmentIblStagingPhase::BundleCommit.enter();
            store
                .commit_prepared_bundle_writes(writes)
                .map_err(EnvironmentIblSourceStagingError::Stage)?;
        }
        report.add_bundle_commit(commit_started.elapsed());
        report.record_profile_observation();
        Ok(report)
    }

    pub(crate) fn into_file_writes(self) -> Vec<PreparedFileWrite> {
        self.report.record_profile_observation();
        self.writes
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
    let format_identity = texture_source_image_format_identity(context)
        .map_err(EnvironmentIblSourceStagingError::Decode)?;
    environment_ibl_request_for_source_image(
        context,
        IblSourceImageIdentity::new(width, height, format_identity),
    )
}

fn environment_ibl_request_for_source_image(
    context: &AssetImportContext,
    source_image: IblSourceImageIdentity,
) -> Result<Option<IblBakeArtifactRequest>, EnvironmentIblSourceStagingError> {
    let mode = environment_ibl_import_mode(context)?;
    if mode == EnvironmentIblImportMode::Disabled || !mode.applies_to(context) {
        return Ok(None);
    }
    let width = source_image.width();
    let height = source_image.height();
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
    let source_identity = derive_source_identity_with_format(
        &context.source_bytes,
        source_image.format_identity(),
        face_size,
        source_mip_count,
    );
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

fn source_image_identity(metadata: TextureSourceImageMetadata) -> IblSourceImageIdentity {
    IblSourceImageIdentity::new(
        metadata.width(),
        metadata.height(),
        metadata.format_identity(),
    )
}

/// Build or reuse the source `.zcube` and companion `.zribl` for an environment image.
///
/// HDR/EXR sources use automatic mode by default and are staged only when their
/// dimensions are 2:1. Other image formats can opt in with `environment_ibl = true`.
pub fn stage_environment_ibl_source(
    context: &AssetImportContext,
    cache_root: impl AsRef<Path>,
) -> Result<EnvironmentIblSourceStagingReport, EnvironmentIblSourceStagingError> {
    prepare_environment_ibl_source(context, cache_root)?.commit()
}

pub(crate) fn prepare_environment_ibl_source(
    context: &AssetImportContext,
    cache_root: impl AsRef<Path>,
) -> Result<PreparedEnvironmentIblSourceStaging, EnvironmentIblSourceStagingError> {
    let cache_root = cache_root.as_ref();
    let (source_image, mut timing) = match probe_environment_ibl_warm_cache(context, cache_root)? {
        EnvironmentIblWarmCacheProbe::Finished(staging) => return Ok(staging),
        EnvironmentIblWarmCacheProbe::Miss {
            source_image,
            timing,
        } => (source_image, timing),
    };
    let decode_started = Instant::now();
    let image = {
        let _phase = EnvironmentIblStagingPhase::SourceDecode.enter();
        decode_texture_source_image_rgba32f(context)
            .map_err(EnvironmentIblSourceStagingError::Decode)?
    };
    timing.source_decode = timing
        .source_decode
        .saturating_add(decode_started.elapsed());
    prepare_environment_ibl_source_with_builder(
        context,
        cache_root,
        image,
        source_image,
        timing,
        EnvironmentIblBundleProbeState::AlreadyMissed,
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
    prepare_environment_ibl_source_with_parallel_executor(context, cache_root, parallel_executor)?
        .commit()
}

pub(crate) fn prepare_environment_ibl_source_with_parallel_executor<E>(
    context: &AssetImportContext,
    cache_root: impl AsRef<Path>,
    parallel_executor: &E,
) -> Result<PreparedEnvironmentIblSourceStaging, EnvironmentIblSourceStagingError>
where
    E: ParallelSliceExecutor,
{
    let cache_root = cache_root.as_ref();
    let (source_image, mut timing) = match probe_environment_ibl_warm_cache(context, cache_root)? {
        EnvironmentIblWarmCacheProbe::Finished(staging) => return Ok(staging),
        EnvironmentIblWarmCacheProbe::Miss {
            source_image,
            timing,
        } => (source_image, timing),
    };
    let decode_started = Instant::now();
    let image = {
        let _phase = EnvironmentIblStagingPhase::SourceDecode.enter();
        decode_texture_source_image_rgba32f(context)
            .map_err(EnvironmentIblSourceStagingError::Decode)?
    };
    timing.source_decode = timing
        .source_decode
        .saturating_add(decode_started.elapsed());
    prepare_environment_ibl_source_with_parallel_executor_and_decoded_image_with_timing(
        context,
        cache_root,
        image,
        source_image,
        parallel_executor,
        timing,
        EnvironmentIblBundleProbeState::AlreadyMissed,
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
    prepare_environment_ibl_source_with_parallel_executor_and_decoded_image_with_timing(
        context,
        cache_root,
        image,
        source_image_identity(
            decode_texture_source_image_metadata(context)
                .map_err(EnvironmentIblSourceStagingError::Decode)?,
        ),
        parallel_executor,
        EnvironmentIblSourceStagingTiming::default(),
        EnvironmentIblBundleProbeState::Required,
    )
    .and_then(PreparedEnvironmentIblSourceStaging::commit)
}

fn prepare_environment_ibl_source_with_parallel_executor_and_decoded_image_with_timing<E>(
    context: &AssetImportContext,
    cache_root: impl AsRef<Path>,
    image: DecodedTextureImageRgba32F,
    source_image: IblSourceImageIdentity,
    parallel_executor: &E,
    timing: EnvironmentIblSourceStagingTiming,
    bundle_probe_state: EnvironmentIblBundleProbeState,
) -> Result<PreparedEnvironmentIblSourceStaging, EnvironmentIblSourceStagingError>
where
    E: ParallelSliceExecutor,
{
    let irradiance_cube_work_items = AtomicUsize::new(0);
    let irradiance_cube_executor = MeasuredParallelSliceExecutor {
        inner: parallel_executor,
        work_items: &irradiance_cube_work_items,
    };
    prepare_environment_ibl_source_with_builder(
        context,
        cache_root,
        image,
        source_image,
        timing,
        bundle_probe_state,
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

fn prepare_environment_ibl_source_with_builder(
    context: &AssetImportContext,
    cache_root: impl AsRef<Path>,
    image: DecodedTextureImageRgba32F,
    source_image: IblSourceImageIdentity,
    mut timing: EnvironmentIblSourceStagingTiming,
    bundle_probe_state: EnvironmentIblBundleProbeState,
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
) -> Result<PreparedEnvironmentIblSourceStaging, EnvironmentIblSourceStagingError> {
    let classify_started = Instant::now();
    let mode = {
        let _phase = EnvironmentIblStagingPhase::SourceClassify.enter();
        environment_ibl_import_mode(context)?
    };
    timing.source_classify = timing
        .source_classify
        .saturating_add(classify_started.elapsed());
    if mode == EnvironmentIblImportMode::Disabled || !mode.applies_to(context) {
        return Ok(PreparedEnvironmentIblSourceStaging {
            store: IblSourceCubemapStagingStore::new(cache_root.as_ref()),
            writes: Vec::new(),
            report: EnvironmentIblSourceStagingReport::skipped(),
        });
    }

    let identity_started = Instant::now();
    let request = {
        let _phase = EnvironmentIblStagingPhase::SourceIdentity.enter();
        environment_ibl_request_for_source_image(context, source_image)?
    };
    timing.source_identity = identity_started.elapsed();
    let Some(request) = request else {
        return Ok(PreparedEnvironmentIblSourceStaging {
            store: IblSourceCubemapStagingStore::new(cache_root.as_ref()),
            writes: Vec::new(),
            report: EnvironmentIblSourceStagingReport::skipped(),
        });
    };
    let face_size = request.source_face_size();
    let source_mip_count = request.source_mip_count();
    let pmrem_face_size = request.pmrem_face_size();
    let pmrem_mip_count = request.pmrem_mip_count();
    let required_contents = request.required_contents();
    let store = IblSourceCubemapStagingStore::new(cache_root.as_ref());
    let source_zcube_path = store.source_cubemap_path(&request);
    let asset_derived_path = store.asset_derived_store().asset_derived_path(&request);

    let cache_probe_started = Instant::now();
    let staged_bundle = {
        let _phase = EnvironmentIblStagingPhase::CacheProbe.enter();
        let current_bundle_manifest_matches = match bundle_probe_state {
            EnvironmentIblBundleProbeState::Required => {
                store.current_bundle_manifest_matches(&request, source_image)?
            }
            EnvironmentIblBundleProbeState::AlreadyMissed => false,
        };
        if current_bundle_manifest_matches {
            EnvironmentIblStagedBundleState::Current
        } else {
            staged_bundle_state(&store, &request, &context.uri, source_image)?
        }
    };
    timing.cache_probe = timing
        .cache_probe
        .saturating_add(cache_probe_started.elapsed());
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
    let cubemap_started = Instant::now();
    let (cubemap, cubemap_timing) = {
        let _phase = EnvironmentIblStagingPhase::CubemapBuild.enter();
        if let Some(source) = staged_source {
            rebuild_cubemap(
                source.face_size(),
                source.mip_count(),
                source.into_texels(),
                pmrem_face_size,
                pmrem_mip_count,
            )
        } else {
            build_cubemap(&image, face_size, pmrem_face_size, pmrem_mip_count)
        }
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
            let _phase = EnvironmentIblStagingPhase::IrradianceCubeBuild.enter();
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
    let (output, writes) = {
        let _phase = EnvironmentIblStagingPhase::BundleEncode.enter();
        prepare_environment_ibl_staged_outputs(
            &store,
            &request,
            context.uri.clone(),
            &cubemap,
            irradiance_cube.as_ref(),
            source_was_reused,
            source_image,
            parallel_work_items,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EnvironmentIblBundleProbeState {
    Required,
    AlreadyMissed,
}

enum EnvironmentIblStagedBundleState {
    Current,
    SourceOnly(ZcubeSourceCubemap),
    Missing,
}

fn prepare_environment_ibl_staged_outputs(
    store: &IblSourceCubemapStagingStore,
    request: &IblBakeArtifactRequest,
    uri: AssetUri,
    cubemap: &SourceCubemapMipChain,
    irradiance_cube: Option<&SourceCubemapIrradianceCube>,
    source_was_reused: bool,
    source_image: IblSourceImageIdentity,
    parallel_work_items: EnvironmentIblSourceStagingParallelWorkItems,
) -> Result<
    (EnvironmentIblSourceStagingOutput, Vec<PreparedFileWrite>),
    EnvironmentIblSourceStagingError,
> {
    let irradiance_cube_source_sample_visits =
        irradiance_cube_source_sample_visits(cubemap, irradiance_cube);
    if source_was_reused {
        let asset_derived = store
            .asset_derived_store()
            .prepare_source_cubemap_asset_derived_artifact(request, cubemap, irradiance_cube)
            .map_err(EnvironmentIblSourceStagingError::WriteAssetDerived)?;
        let (path, bytes, asset_derived) = asset_derived.into_parts();
        let manifest =
            store.prepare_bundle_manifest_for_existing_source(request, source_image, &bytes)?;
        let output = EnvironmentIblSourceStagingOutput::from_reused_source_and_written_asset(
            &store.source_cubemap_path(request),
            asset_derived.encoded_len(),
            parallel_work_items,
            irradiance_cube_source_sample_visits,
        )?;
        return Ok((output, vec![PreparedFileWrite::new(path, bytes), manifest]));
    }

    let (writes, bundle) = store
        .prepare_source_cubemap_staged_bundle_with_source_image(
            request,
            uri,
            cubemap,
            irradiance_cube,
            source_image,
        )
        .map_err(EnvironmentIblSourceStagingError::Stage)?;
    let output = EnvironmentIblSourceStagingOutput::from_written_bundle(
        &bundle,
        parallel_work_items,
        irradiance_cube_source_sample_visits,
    );
    Ok((output, writes))
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
    source_image: IblSourceImageIdentity,
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
        if matches!(derived, IblBakeArtifactAssetDerivedRead::Hit(_))
            && store.current_bundle_manifest_matches(request, source_image)?
        {
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
mod tests;
