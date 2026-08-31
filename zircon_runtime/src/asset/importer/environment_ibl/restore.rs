use std::fs;
use std::path::Path;

use crate::asset::artifact::{IblSourceCubemapStagingError, IblSourceCubemapStagingStore};
use crate::core::framework::render::SourceCubemapEnvironment;

use super::{
    environment_ibl_request_for_dimensions, AssetImportContext, EnvironmentIblSourceStagingError,
    EnvironmentIblSourceStagingOutput, EnvironmentIblSourceStagingReport,
    EnvironmentIblSourceStagingStatus, EnvironmentIblSourceStagingTiming,
};

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

/// Restores a current source and derived IBL bundle without decoding source pixels.
///
/// Rebuildable cache failures invalidate only the artifacts needed to make the next staging
/// attempt progress. Filesystem and request-layout failures remain visible errors.
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
    let bundle_manifest_path = store.bundle_manifest_path(&request);
    let environment = match store.read_source_cubemap_environment(&request, context.uri.clone()) {
        Ok(environment) => environment,
        Err(error) => {
            recover_source_restore_error(
                error,
                &source_zcube_path,
                &asset_derived_path,
                &bundle_manifest_path,
            )?;
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

pub(super) fn source_restore_is_rebuildable_cache_miss(
    error: &IblSourceCubemapStagingError,
) -> bool {
    matches!(
        error,
        IblSourceCubemapStagingError::MissingSourceCubemap
            | IblSourceCubemapStagingError::MissingAssetDerived
            | IblSourceCubemapStagingError::MissingBundleManifest
            | IblSourceCubemapStagingError::RejectedAssetDerived(_)
            | IblSourceCubemapStagingError::RejectedBundleManifest(_)
            | IblSourceCubemapStagingError::BundleManifestRequestMismatch
            | IblSourceCubemapStagingError::BundlePayloadStampMismatch { .. }
            | IblSourceCubemapStagingError::DecodeZcube { .. }
    )
}

pub(super) fn recover_source_restore_error(
    error: IblSourceCubemapStagingError,
    source_zcube_path: &Path,
    asset_derived_path: &Path,
    bundle_manifest_path: &Path,
) -> Result<(), EnvironmentIblSourceStagingError> {
    match error {
        IblSourceCubemapStagingError::ApplyAssetDerived(_)
        | IblSourceCubemapStagingError::RejectedAssetDerived(_)
        | IblSourceCubemapStagingError::BundlePayloadStampMismatch {
            payload: "asset-derived.zribl",
        } => invalidate_staged_artifacts(&[bundle_manifest_path, asset_derived_path]),
        IblSourceCubemapStagingError::DecodeZcube { .. }
        | IblSourceCubemapStagingError::BundlePayloadStampMismatch {
            payload: "source.zcube",
        } => invalidate_staged_artifacts(&[bundle_manifest_path, source_zcube_path]),
        IblSourceCubemapStagingError::RejectedBundleManifest(_)
        | IblSourceCubemapStagingError::BundleManifestRequestMismatch => {
            invalidate_staged_artifacts(&[
                bundle_manifest_path,
                source_zcube_path,
                asset_derived_path,
            ])
        }
        error if source_restore_is_rebuildable_cache_miss(&error) => Ok(()),
        error => Err(EnvironmentIblSourceStagingError::Stage(error)),
    }
}

fn invalidate_staged_artifacts(paths: &[&Path]) -> Result<(), EnvironmentIblSourceStagingError> {
    for path in paths {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(
                    EnvironmentIblSourceStagingError::RemoveInvalidStagedArtifact {
                        path: path.to_path_buf(),
                        source,
                    },
                );
            }
        }
    }
    Ok(())
}
