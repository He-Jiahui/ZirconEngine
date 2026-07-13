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
    #[error(
        "failed to materialize editor export: {source}; native staging cleanup also failed: {cleanup}"
    )]
    MaterializeWithCleanup {
        #[source]
        source: io::Error,
        cleanup: NativeDynamicPreparationError,
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
    #[error("desktop export cancelled during {stage}; cleanup failed: {source}")]
    CancelledWithCleanup {
        stage: String,
        #[source]
        source: NativeDynamicPreparationError,
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

    pub(super) fn materialize_with_cleanup(
        source: io::Error,
        cleanup: Option<NativeDynamicPreparationError>,
    ) -> Self {
        match cleanup {
            Some(cleanup) => Self::MaterializeWithCleanup { source, cleanup },
            None => Self::Materialize { source },
        }
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

    pub(super) fn cancelled(
        stage: impl Into<String>,
        cleanup_error: Option<NativeDynamicPreparationError>,
    ) -> Self {
        let stage = stage.into();
        match cleanup_error {
            Some(source) => Self::CancelledWithCleanup { stage, source },
            None => Self::Cancelled { stage },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;
    use std::io;
    use std::path::PathBuf;

    use super::*;

    fn native_cleanup_error(source: io::Error) -> NativeDynamicPreparationError {
        NativeDynamicPreparationError::Cleanup {
            path: PathBuf::from(".native-dynamic-staging"),
            source,
        }
    }

    #[test]
    fn cargo_error_preserves_process_and_io_sources() {
        let error = EditorExportBuildError::cargo(ExportProcessError::io(
            "failed to invoke Cargo",
            "typed cargo test",
            None,
            Some(PathBuf::from("Cargo.toml")),
            io::Error::new(io::ErrorKind::PermissionDenied, "cargo source"),
        ));

        let process = error
            .source()
            .and_then(|source| source.downcast_ref::<ExportProcessError>())
            .expect("Cargo error must retain its process error");
        let source = process
            .source()
            .and_then(|source| source.downcast_ref::<io::Error>())
            .expect("process error must retain its IO error");
        assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn cancellation_cleanup_error_preserves_native_io_source() {
        let error = EditorExportBuildError::cancelled(
            "test cancellation",
            Some(native_cleanup_error(io::Error::new(
                io::ErrorKind::DirectoryNotEmpty,
                "cleanup source",
            ))),
        );

        let native = error
            .source()
            .and_then(|source| source.downcast_ref::<NativeDynamicPreparationError>())
            .expect("cancellation must retain its cleanup error");
        let source = native
            .source()
            .and_then(|source| source.downcast_ref::<io::Error>())
            .expect("native cleanup must retain its IO error");
        assert_eq!(source.kind(), io::ErrorKind::DirectoryNotEmpty);
    }

    #[test]
    fn materialization_error_preserves_io_source() {
        let error = EditorExportBuildError::materialize(io::Error::new(
            io::ErrorKind::WriteZero,
            "materialize source",
        ));

        let source = error
            .source()
            .and_then(|source| source.downcast_ref::<io::Error>())
            .expect("materialization error must retain its IO error");
        assert_eq!(source.kind(), io::ErrorKind::WriteZero);
    }

    #[test]
    fn materialization_error_retains_typed_cleanup_failure() {
        let error = EditorExportBuildError::materialize_with_cleanup(
            io::Error::new(io::ErrorKind::WriteZero, "materialize source"),
            Some(native_cleanup_error(io::Error::new(
                io::ErrorKind::DirectoryNotEmpty,
                "cleanup source",
            ))),
        );

        assert!(matches!(
            error,
            EditorExportBuildError::MaterializeWithCleanup {
                source,
                cleanup: NativeDynamicPreparationError::Cleanup {
                    source: cleanup_source,
                    ..
                },
            } if source.kind() == io::ErrorKind::WriteZero
                && cleanup_source.kind() == io::ErrorKind::DirectoryNotEmpty
        ));
    }
}
