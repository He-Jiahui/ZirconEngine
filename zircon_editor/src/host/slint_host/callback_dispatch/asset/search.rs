#[cfg(test)]
use crate::editor_event::{EditorEventRuntime, host_adapter};
#[cfg(test)]
use crate::host::slint_host::event_bridge::SlintDispatchEffects;

#[cfg(test)]
use super::super::common::dispatch_envelope;

#[cfg(test)]
pub(crate) fn dispatch_asset_search(
    runtime: &EditorEventRuntime,
    query: impl Into<String>,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_asset_search(query))
}
