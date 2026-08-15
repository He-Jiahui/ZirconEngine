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
