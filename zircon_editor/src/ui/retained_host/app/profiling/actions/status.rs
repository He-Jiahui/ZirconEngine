#[cfg(any(feature = "profiling", test))]
fn single_buffer_profiling_status(editor_message: &str, runtime_message: &str) -> String {
    const EDITOR_PREFIX: &str = "Editor profiling: ";
    const RUNTIME_PREFIX: &str = "; Runtime profiling: ";

    let mut status = String::with_capacity(
        EDITOR_PREFIX.len() + editor_message.len() + RUNTIME_PREFIX.len() + runtime_message.len(),
    );
    status.push_str(EDITOR_PREFIX);
    status.push_str(editor_message);
    status.push_str(RUNTIME_PREFIX);
    status.push_str(runtime_message);
    status
}

#[cfg(feature = "profiling")]
pub(super) fn performance_timeline_action_status(
    editor_response: &zircon_runtime_interface::ProfileControlResponse,
    runtime_response: Result<Option<zircon_runtime_interface::ProfileControlResponse>, String>,
) -> String {
    let runtime_message = match &runtime_response {
        Ok(Some(response)) => response.message.as_str(),
        Ok(None) => "unavailable",
        Err(error) => error.as_str(),
    };
    single_buffer_profiling_status(&editor_response.message, runtime_message)
}

#[cfg(test)]
#[path = "status/single_buffer_status_tests.rs"]
mod single_buffer_status_tests;
