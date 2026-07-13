//! Headless export-pipeline orchestration owned by the editor core.

mod pipeline;
mod preset;
mod stages;

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
mod tests;
