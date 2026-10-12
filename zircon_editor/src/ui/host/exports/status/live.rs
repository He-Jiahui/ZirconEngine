use super::super::super::editor_manager::EditorManager;
use super::super::super::project_session_transition::ProjectSessionTransitionGate;
use super::super::reports::EditorPluginStatusReport;

impl EditorManager {
    /// Holds project lifecycle authority through the native loader mutation.
    /// A retired watch cannot load its old project after close or a new admission.
    pub(crate) fn execute_native_live_plugin_for_session<T>(
        &self,
        expected_session: &(std::path::PathBuf, String, u64),
        execute: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        execute_for_matching_project_session(
            &self.project_session_transition,
            expected_session,
            || self.active_project_session_focus_target(),
            execute,
        )
    }

    /// Publishes one new immutable row generation for a completed live operation.
    /// The retained host verifies project-session identity before calling this for a watch result.
    pub(crate) fn publish_native_live_plugin_status(
        &self,
        expected_session: &(std::path::PathBuf, String, u64),
        plugin_id: &str,
        operation: &str,
        outcome: &Result<String, String>,
        live_loaded: &Result<bool, String>,
    ) -> Result<bool, String> {
        let _transition = self
            .begin_project_session_transition()
            .map_err(|error| error.to_string())?;
        if self.active_project_session_focus_target().as_ref() != Some(expected_session) {
            return Ok(false);
        }
        Ok(self.update_project_plugin_status(|previous| {
            project_live_status_report(previous, plugin_id, operation, outcome, live_loaded)
        }))
    }
}

fn execute_for_matching_project_session<T>(
    transition: &ProjectSessionTransitionGate,
    expected_session: &(std::path::PathBuf, String, u64),
    active_session: impl FnOnce() -> Option<(std::path::PathBuf, String, u64)>,
    execute: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let _transition = transition.enter().map_err(|_| {
        "project session lifecycle is quarantined after a prior transition panic".to_string()
    })?;
    if active_session().as_ref() != Some(expected_session) {
        return Err("native live plugin action targets a retired project session".to_string());
    }
    execute()
}

fn project_live_status_report(
    previous: &EditorPluginStatusReport,
    plugin_id: &str,
    operation: &str,
    outcome: &Result<String, String>,
    live_loaded: &Result<bool, String>,
) -> EditorPluginStatusReport {
    const DIAGNOSTIC_PREFIX: &str = "native.live.";

    let mut next = previous.clone();
    let Some(plugin) = next
        .plugins
        .iter_mut()
        .find(|plugin| plugin.plugin_id == plugin_id)
    else {
        next.diagnostics.push(format!(
            "native.live.{operation}.status_unavailable: plugin `{plugin_id}` has no current project row"
        ));
        return next;
    };

    plugin
        .diagnostics
        .retain(|diagnostic| !diagnostic.starts_with(DIAGNOSTIC_PREFIX));
    plugin.load_state = match live_loaded {
        Ok(true) => "loaded".to_string(),
        Ok(false) => "unloaded".to_string(),
        Err(error) => {
            plugin.diagnostics.push(format!(
                "native.live.state_query_failed: {error}; retry or refresh the project"
            ));
            "live state unknown".to_string()
        }
    };
    match outcome {
        Ok(message) => plugin
            .diagnostics
            .push(format!("native.live.{operation}.completed: {message}")),
        Err(error) => plugin.diagnostics.push(format!(
            "native.live.{operation}.failed: {error}; retry is available"
        )),
    }
    next
}

#[cfg(test)]
#[path = "live/tests/cases.rs"]
mod tests;
