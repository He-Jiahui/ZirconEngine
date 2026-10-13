//! Stable export-pipeline contracts shared by editor, runtime, and plugins.

#[path = "artifact.rs"]
mod artifact;
#[path = "preset.rs"]
mod preset;
#[path = "report.rs"]
mod report;
#[path = "stage.rs"]
mod stage;

pub use artifact::{ExportArtifactRef, ExportDigest, ExportStageIo};
pub use preset::{
    load_export_preset, valid_export_profile_name, ExportCookCompression, ExportCookOptions,
    ExportFileMode,
    ExportPluginSubset, ExportPreset, ExportPresetLoadError, ExportPresetValidationError,
    ExportTargetMode,
};
pub use report::{ExportPipelineReport, ExportStageRecord, ExportStageStatus};
pub use stage::{ExportStage, ParseExportStageError};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
