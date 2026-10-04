mod diagnostics_sink;
mod orchestrator;
mod request;
mod settings;

pub use diagnostics_sink::{ScriptBuildDiagnosticsSink, ScriptDiagnosticProjectionReport};
pub use orchestrator::{
    ScriptBuildCompletionError, ScriptBuildEnqueueError, ScriptBuildOrchestrator, ScriptBuildPhase,
    ScriptBuildSnapshot, MAX_INCREMENTAL_SCRIPT_WATCH_PATHS,
    MAX_INCREMENTAL_SCRIPT_WATCH_PATH_BYTES,
};
pub use request::{
    ScriptBuildCompletion, ScriptBuildGeneration, ScriptBuildOutcome, ScriptBuildRequest,
    ScriptBuildRequestId, ScriptBuildStep, ScriptBuildStepDispatch, ScriptBuildTrigger,
};
pub(crate) use settings::register_script_build_settings;
pub use settings::{
    ScriptBuildBatchPolicy, DEFAULT_SCRIPT_WATCH_DEBOUNCE_MS, DEFAULT_SCRIPT_WATCH_MAX_LATENCY_MS,
    MAXIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS, MINIMUM_SCRIPT_BUILD_BATCH_WINDOW_MS,
    SCRIPT_BUILD_BATCH_WINDOW_MS_KEY, SCRIPT_BUILD_BATCH_WINDOW_STEP_MS,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
