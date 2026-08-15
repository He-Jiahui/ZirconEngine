mod diagnostics_sink;
mod orchestrator;
mod request;

pub use diagnostics_sink::{ScriptBuildDiagnosticsSink, ScriptDiagnosticProjectionReport};
pub use orchestrator::{
    ScriptBuildCompletionError, ScriptBuildEnqueueError, ScriptBuildOrchestrator, ScriptBuildPhase,
    ScriptBuildSnapshot, DEFAULT_SCRIPT_WATCH_DEBOUNCE_MS, DEFAULT_SCRIPT_WATCH_MAX_LATENCY_MS,
    MAX_INCREMENTAL_SCRIPT_WATCH_PATHS, MAX_INCREMENTAL_SCRIPT_WATCH_PATH_BYTES,
};
pub use request::{
    ScriptBuildCompletion, ScriptBuildGeneration, ScriptBuildOutcome, ScriptBuildRequest,
    ScriptBuildRequestId, ScriptBuildStep, ScriptBuildStepDispatch, ScriptBuildTrigger,
};

#[cfg(test)]
mod tests;
