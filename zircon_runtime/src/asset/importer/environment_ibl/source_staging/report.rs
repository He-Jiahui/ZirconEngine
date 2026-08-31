use std::path::{Path, PathBuf};

use crate::core::framework::render::IblBakeArtifactRequest;

use super::{
    EnvironmentIblSourceStagingOutput, EnvironmentIblSourceStagingStatus,
    EnvironmentIblSourceStagingTiming,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentIblSourceStagingReport {
    status: EnvironmentIblSourceStagingStatus,
    request: Option<IblBakeArtifactRequest>,
    source_zcube_path: Option<PathBuf>,
    asset_derived_path: Option<PathBuf>,
    timing: EnvironmentIblSourceStagingTiming,
    output: EnvironmentIblSourceStagingOutput,
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

    pub const fn timing(&self) -> EnvironmentIblSourceStagingTiming {
        self.timing
    }

    pub const fn output(&self) -> EnvironmentIblSourceStagingOutput {
        self.output
    }

    pub(in crate::asset::importer::environment_ibl) fn add_bundle_commit(
        &mut self,
        duration: std::time::Duration,
    ) {
        self.timing.bundle_commit = self.timing.bundle_commit.saturating_add(duration);
    }

    pub(in crate::asset::importer::environment_ibl) fn record_profile_observation(&self) {
        #[cfg(feature = "profiling")]
        {
            use crate::core::runtime::diagnostics::profiling::{
                capture_active, record_counter_batch,
            };

            if !capture_active() {
                return;
            }
            let source_layout = self
                .request
                .as_ref()
                .map(|request| (request.source_face_size(), request.source_mip_count()))
                .unwrap_or_default();
            record_counter_batch(
                "asset",
                &[
                    ("asset.environment_ibl.attempt_count", 1.0),
                    (
                        "asset.environment_ibl.reused_count",
                        if matches!(self.status, EnvironmentIblSourceStagingStatus::Reused) {
                            1.0
                        } else {
                            0.0
                        },
                    ),
                    (
                        "asset.environment_ibl.written_count",
                        if matches!(self.status, EnvironmentIblSourceStagingStatus::Written) {
                            1.0
                        } else {
                            0.0
                        },
                    ),
                    (
                        "asset.environment_ibl.source_face_size",
                        f64::from(source_layout.0),
                    ),
                    (
                        "asset.environment_ibl.source_mip_count",
                        f64::from(source_layout.1),
                    ),
                    (
                        "asset.environment_ibl.source_zcube_bytes",
                        self.output.source_zcube_bytes() as f64,
                    ),
                    (
                        "asset.environment_ibl.asset_derived_bytes",
                        self.output.asset_derived_bytes() as f64,
                    ),
                    (
                        "asset.environment_ibl.source_classify_us",
                        duration_us(self.timing.source_classify()),
                    ),
                    (
                        "asset.environment_ibl.source_identity_us",
                        duration_us(self.timing.source_identity()),
                    ),
                    (
                        "asset.environment_ibl.cache_probe_us",
                        duration_us(self.timing.cache_probe()),
                    ),
                    (
                        "asset.environment_ibl.source_decode_us",
                        duration_us(self.timing.source_decode()),
                    ),
                    (
                        "asset.environment_ibl.cubemap_build_us",
                        duration_us(self.timing.cubemap_build()),
                    ),
                    (
                        "asset.environment_ibl.equirect_projection_us",
                        duration_us(self.timing.equirect_projection()),
                    ),
                    (
                        "asset.environment_ibl.source_mip_build_us",
                        duration_us(self.timing.source_mip_build()),
                    ),
                    (
                        "asset.environment_ibl.pmrem_build_us",
                        duration_us(self.timing.pmrem_build()),
                    ),
                    (
                        "asset.environment_ibl.sh9_build_us",
                        duration_us(self.timing.sh9_build()),
                    ),
                    (
                        "asset.environment_ibl.irradiance_cube_build_us",
                        duration_us(self.timing.irradiance_cube_build()),
                    ),
                    (
                        "asset.environment_ibl.bundle_encode_us",
                        duration_us(self.timing.bundle_encode()),
                    ),
                    (
                        "asset.environment_ibl.bundle_commit_us",
                        duration_us(self.timing.bundle_commit()),
                    ),
                    (
                        "asset.environment_ibl.total_us",
                        duration_us(self.timing.total()),
                    ),
                    (
                        "asset.environment_ibl.parallel_work_items",
                        self.output.parallel_executor_work_items() as f64,
                    ),
                    (
                        "asset.environment_ibl.irradiance_cube_source_sample_visits",
                        self.output.irradiance_cube_source_sample_visits() as f64,
                    ),
                ],
            );
        }
    }

    pub(in crate::asset::importer::environment_ibl) fn skipped() -> Self {
        Self {
            status: EnvironmentIblSourceStagingStatus::Skipped,
            request: None,
            source_zcube_path: None,
            asset_derived_path: None,
            timing: EnvironmentIblSourceStagingTiming::default(),
            output: EnvironmentIblSourceStagingOutput::default(),
        }
    }

    pub(in crate::asset::importer::environment_ibl) fn current(
        status: EnvironmentIblSourceStagingStatus,
        request: IblBakeArtifactRequest,
        source_zcube_path: PathBuf,
        asset_derived_path: PathBuf,
        timing: EnvironmentIblSourceStagingTiming,
        output: EnvironmentIblSourceStagingOutput,
    ) -> Self {
        Self {
            status,
            request: Some(request),
            source_zcube_path: Some(source_zcube_path),
            asset_derived_path: Some(asset_derived_path),
            timing,
            output,
        }
    }
}

#[cfg(feature = "profiling")]
fn duration_us(duration: std::time::Duration) -> f64 {
    duration.as_secs_f64() * 1_000_000.0
}
