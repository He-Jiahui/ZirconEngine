use std::io;

use thiserror::Error;
use zircon_runtime::asset::project::ProjectManifestError;
use zircon_runtime::plugin::ExportBuildPlanError;
use zircon_runtime::scene::world::SceneProjectError;
use zircon_runtime_interface::export::ExportStage;

use crate::core::export::ExportPresetStoreError;
use crate::core::export::PlatformBundleLayoutError;

use super::super::super::export_process_support::ExportProcessError;
use super::super::super::native_dynamic_export_preparation::NativeDynamicPreparationError;

#[derive(Debug, Error)]
pub enum EditorExportBuildError {
    #[error("export build failed for profile {}: {}", report.plan.profile.name, report.failure_reason().unwrap_or("unknown failure"))]
    ReportFailed {
        report: Box<super::report::EditorExportBuildReport>,
    },
    #[error(transparent)]
    Plan(#[from] ExportBuildPlanError),
    #[error("failed to resolve the export project root: {0}")]
    ProjectRoot(#[from] SceneProjectError),
    #[error("unknown desktop export profile `{profile_name}`")]
    UnknownProfile { profile_name: String },
    #[error(transparent)]
    Preset(#[from] ExportPresetStoreError),
    #[error(transparent)]
    PlatformBundleLayout(#[from] PlatformBundleLayoutError),
    #[error(
        "export preset `{preset_name}` target mode {preset_mode:?} does not match profile `{profile_name}` target mode {profile_mode}"
    )]
    PresetTargetModeMismatch {
        preset_name: String,
        profile_name: String,
        preset_mode: zircon_runtime_interface::export::ExportTargetMode,
        profile_mode: &'static str,
    },
    #[error("core export stage {stage:?} does not have a production executor")]
    CoreUnsupportedStage { stage: ExportStage },
    #[error("core PlatformBundle execution is missing the CompileHost record")]
    CoreMissingCompileHostRecord,
    #[error("failed to encode the export preset fingerprint: {source}")]
    CorePresetFingerprint {
        #[source]
        source: zircon_runtime_interface::serialization::WriteError,
    },
    #[error("failed to fingerprint export inputs or outputs: {source}")]
    CoreArtifactFingerprint {
        #[source]
        source: io::Error,
    },
    #[error("{failure}; failed to persist the core export failure receipt: {persistence}")]
    CoreFailureReceipt {
        #[source]
        failure: Box<EditorExportBuildError>,
        persistence: Box<EditorExportBuildError>,
    },
    #[error("failed to load export project manifest {}: {source}", path.display())]
    ProjectManifest {
        path: std::path::PathBuf,
        #[source]
        source: ProjectManifestError,
    },
    #[error("failed to materialize editor export: {source}")]
    Materialize {
        #[source]
        source: io::Error,
    },
    #[error(transparent)]
    Process(#[from] ExportProcessError),
    #[error("Cargo export build failed: {source}")]
    Cargo {
        #[source]
        source: ExportProcessError,
    },
    #[error(transparent)]
    NativePreparation(#[from] NativeDynamicPreparationError),
    #[error("desktop export cancelled during {stage}")]
    Cancelled { stage: String },
    #[error("export wizard plan for profile {profile} failed: {}", diagnostics.join("; "))]
    WizardPlanFailed {
        profile: String,
        diagnostics: Vec<String>,
    },
    #[error("export wizard stage {stage:?} failed with exit code {exit_code:?}")]
    WizardStageFailed {
        stage: ExportStage,
        exit_code: Option<i32>,
    },
    #[error("export wizard job {job_id} returned non-terminal status {status}")]
    WizardNonTerminal {
        job_id: String,
        status: &'static str,
    },
}

impl EditorExportBuildError {
    pub(super) fn materialize(source: io::Error) -> Self {
        Self::Materialize { source }
    }

    pub(in crate::ui) fn unknown_profile(profile_name: impl Into<String>) -> Self {
        Self::UnknownProfile {
            profile_name: profile_name.into(),
        }
    }

    pub(in crate::ui) fn project_manifest(
        path: std::path::PathBuf,
        source: ProjectManifestError,
    ) -> Self {
        Self::ProjectManifest { path, source }
    }

    pub(super) fn cargo(source: ExportProcessError) -> Self {
        Self::Cargo { source }
    }

    pub(super) fn cancelled(stage: impl Into<String>) -> Self {
        Self::Cancelled {
            stage: stage.into(),
        }
    }
}

#[cfg(test)]
#[path = "tests/error.rs"]
mod tests;
