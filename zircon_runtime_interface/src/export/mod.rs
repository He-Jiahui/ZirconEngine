//! Stable export-pipeline contracts shared by editor, runtime, and plugins.

mod artifact;
mod preset;
mod report;
mod stage;

pub use artifact::{ExportArtifactRef, ExportDigest, ExportStageIo};
pub use preset::{
    load_export_preset, ExportCookCompression, ExportCookOptions, ExportFileMode,
    ExportPluginSubset, ExportPreset, ExportPresetLoadError, ExportPresetValidationError,
    ExportTargetMode,
};
pub use report::{ExportPipelineReport, ExportStageRecord, ExportStageStatus};
pub use stage::{ExportStage, ParseExportStageError};

#[cfg(test)]
mod tests;
