#[cfg(test)]
use crate::editor_event::{host_adapter, EditorEventRuntime};
#[cfg(test)]
use crate::host::slint_host::event_bridge::SlintDispatchEffects;

#[cfg(test)]
use super::super::common::dispatch_envelope;

#[cfg(test)]
#[cfg(test)]
pub(crate) fn dispatch_asset_item_selection(
    runtime: &EditorEventRuntime,
    asset_uuid: impl Into<String>,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(
        runtime,
        host_adapter::slint_asset_item_selection(asset_uuid),
    )
}
