//! Headless export-pipeline orchestration owned by the editor core.

mod inventory;
mod pipeline;
mod preset;
mod stages;

pub(crate) use inventory::{
    ExportGenerationInventory, FileMetadataIdentity, file_metadata_identity,
    persist_bytes_atomically,
};
pub use pipeline::{
    ExportPipelinePlan, ExportPipelinePlanError, ExportPipelineRunError, ExportStageExecutor,
    ExportStageNode, ExportStageOutput, ExportStagePreparation,
};
pub use preset::{ExportPresetStore, ExportPresetStoreError};
pub use stages::{
    CompileHostStage, PlatformBundleLayout, PlatformBundleLayoutError,
    SystemZirconBuildCommandRunner, ZirconBuildCommand, ZirconBuildCommandError,
    ZirconBuildCommandExecution, ZirconBuildCommandRunner, ZirconBuildStageExecutor,
    ZirconBuildStageExecutorError, zircon_build_stage_plan,
};

#[cfg(test)]
mod tests;
