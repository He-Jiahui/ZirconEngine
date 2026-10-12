//! Headless export-pipeline orchestration owned by the editor core.

#[path = "inventory.rs"]
mod inventory;
#[path = "pipeline.rs"]
mod pipeline;
#[path = "preset.rs"]
mod preset;
#[path = "stages/mod.rs"]
mod stages;

pub(crate) use inventory::{
    file_metadata_identity, persist_bytes_atomically, ExportGenerationInventory,
    FileMetadataIdentity,
};
pub use pipeline::{
    ExportPipelinePlan, ExportPipelinePlanError, ExportPipelineRunError, ExportStageExecutor,
    ExportStageNode, ExportStageOutput, ExportStagePreparation,
};
pub use preset::{ExportPresetStore, ExportPresetStoreError};
pub use stages::{
    zircon_build_stage_plan, CompileHostStage, PlatformBundleLayout, PlatformBundleLayoutError,
    SystemZirconBuildCommandRunner, ZirconBuildCommand, ZirconBuildCommandError,
    ZirconBuildCommandExecution, ZirconBuildCommandRunner, ZirconBuildStageExecutor,
    ZirconBuildStageExecutorError,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
