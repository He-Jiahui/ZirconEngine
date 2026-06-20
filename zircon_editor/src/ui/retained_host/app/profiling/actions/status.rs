#[cfg(feature = "profiling")]
pub(super) fn performance_timeline_action_status(
    editor_response: &zircon_runtime_interface::ProfileControlResponse,
    runtime_response: Result<Option<zircon_runtime_interface::ProfileControlResponse>, String>,
) -> String {
    let mut parts = vec![format!("Editor profiling: {}", editor_response.message)];
    match runtime_response {
        Ok(Some(response)) => parts.push(format!("Runtime profiling: {}", response.message)),
        Ok(None) => parts.push("Runtime profiling: unavailable".to_string()),
        Err(error) => parts.push(format!("Runtime profiling: {error}")),
    }
    parts.join("; ")
}
