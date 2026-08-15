use std::path::Path;

use crate::asset::artifact::IblSourceCubemapStagedBundleReport;

use super::EnvironmentIblSourceStagingError;

/// Chunk submissions attributed to one parallel source-IBL staging attempt.
///
/// These are dispatch-shape counters only. They do not report worker
/// utilization, execution overlap, or CPU time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::asset::importer::environment_ibl) struct EnvironmentIblSourceStagingParallelWorkItems
{
    pub(in crate::asset::importer::environment_ibl) equirect_projection: u64,
    pub(in crate::asset::importer::environment_ibl) source_mip_build: u64,
    pub(in crate::asset::importer::environment_ibl) pmrem_build: u64,
    pub(in crate::asset::importer::environment_ibl) irradiance_cube_build: u64,
}

/// Persisted output and executor work attributed to one source staging attempt.
///
/// Byte counts describe the current `.zcube` and asset-derived `.zribl` files.
/// Work items count chunks submitted through the caller-owned parallel executor.
/// Cache reuse and serial import paths deliberately report zero for every phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnvironmentIblSourceStagingOutput {
    pub(in crate::asset::importer::environment_ibl) source_zcube_bytes: u64,
    pub(in crate::asset::importer::environment_ibl) asset_derived_bytes: u64,
    pub(in crate::asset::importer::environment_ibl) equirect_projection_parallel_work_items: u64,
    pub(in crate::asset::importer::environment_ibl) source_mip_build_parallel_work_items: u64,
    pub(in crate::asset::importer::environment_ibl) pmrem_build_parallel_work_items: u64,
    pub(in crate::asset::importer::environment_ibl) irradiance_cube_build_parallel_work_items: u64,
    pub(in crate::asset::importer::environment_ibl) irradiance_cube_source_sample_visits: u64,
}

impl EnvironmentIblSourceStagingOutput {
    pub const fn source_zcube_bytes(&self) -> u64 {
        self.source_zcube_bytes
    }

    pub const fn asset_derived_bytes(&self) -> u64 {
        self.asset_derived_bytes
    }

    pub const fn parallel_executor_work_items(&self) -> u64 {
        self.equirect_projection_parallel_work_items
            .saturating_add(self.source_mip_build_parallel_work_items)
            .saturating_add(self.pmrem_build_parallel_work_items)
            .saturating_add(self.irradiance_cube_build_parallel_work_items)
    }

    /// Chunks submitted while projecting the equirectangular source.
    pub const fn equirect_projection_parallel_work_items(&self) -> u64 {
        self.equirect_projection_parallel_work_items
    }

    /// Chunks submitted while filtering the source-mip pyramid.
    pub const fn source_mip_build_parallel_work_items(&self) -> u64 {
        self.source_mip_build_parallel_work_items
    }

    /// Chunks submitted while filtering the independent PMREM result.
    pub const fn pmrem_build_parallel_work_items(&self) -> u64 {
        self.pmrem_build_parallel_work_items
    }

    /// Chunks submitted while convolving the optional irradiance cube.
    pub const fn irradiance_cube_build_parallel_work_items(&self) -> u64 {
        self.irradiance_cube_build_parallel_work_items
    }

    /// Direct cosine IEM source/output candidate iterations for this staging attempt.
    ///
    /// The v11/v12 evidence key predates this clarification. The convolution
    /// evaluates `n dot l` before loading a source texel, so this deterministic
    /// layout-derived count is an upper-bound work metric, not a hardware or
    /// actual texture-read counter.
    pub const fn irradiance_cube_source_sample_visits(&self) -> u64 {
        self.irradiance_cube_source_sample_visits
    }

    pub(in crate::asset::importer::environment_ibl) fn from_written_bundle(
        bundle: &IblSourceCubemapStagedBundleReport,
        parallel_work_items: EnvironmentIblSourceStagingParallelWorkItems,
        irradiance_cube_source_sample_visits: u64,
    ) -> Self {
        Self {
            source_zcube_bytes: bundle.source_zcube().encoded_len() as u64,
            asset_derived_bytes: bundle.asset_derived().encoded_len() as u64,
            equirect_projection_parallel_work_items: parallel_work_items.equirect_projection,
            source_mip_build_parallel_work_items: parallel_work_items.source_mip_build,
            pmrem_build_parallel_work_items: parallel_work_items.pmrem_build,
            irradiance_cube_build_parallel_work_items: parallel_work_items.irradiance_cube_build,
            irradiance_cube_source_sample_visits,
        }
    }

    pub(in crate::asset::importer::environment_ibl) fn from_reused_paths(
        source_zcube_path: &Path,
        asset_derived_path: &Path,
    ) -> Result<Self, EnvironmentIblSourceStagingError> {
        Ok(Self {
            source_zcube_bytes: staging_output_file_len(source_zcube_path)?,
            asset_derived_bytes: staging_output_file_len(asset_derived_path)?,
            ..Self::default()
        })
    }

    pub(in crate::asset::importer::environment_ibl) fn from_reused_source_and_written_asset(
        source_zcube_path: &Path,
        asset_derived_bytes: usize,
        parallel_work_items: EnvironmentIblSourceStagingParallelWorkItems,
        irradiance_cube_source_sample_visits: u64,
    ) -> Result<Self, EnvironmentIblSourceStagingError> {
        Ok(Self {
            source_zcube_bytes: staging_output_file_len(source_zcube_path)?,
            asset_derived_bytes: asset_derived_bytes as u64,
            equirect_projection_parallel_work_items: parallel_work_items.equirect_projection,
            source_mip_build_parallel_work_items: parallel_work_items.source_mip_build,
            pmrem_build_parallel_work_items: parallel_work_items.pmrem_build,
            irradiance_cube_build_parallel_work_items: parallel_work_items.irradiance_cube_build,
            irradiance_cube_source_sample_visits,
        })
    }
}

fn staging_output_file_len(path: &Path) -> Result<u64, EnvironmentIblSourceStagingError> {
    std::fs::metadata(path)
        .map(|metadata| metadata.len())
        .map_err(|source| EnvironmentIblSourceStagingError::OutputMetadata {
            path: path.to_path_buf(),
            source,
        })
}
