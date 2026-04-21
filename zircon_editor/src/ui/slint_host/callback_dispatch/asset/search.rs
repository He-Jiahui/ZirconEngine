#[cfg(test)]
use crate::core::editor_event::{
    EditorAssetEvent, EditorEvent, EditorEventEnvelope, EditorEventRuntime, EditorEventSource,
};
#[cfg(test)]
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;

#[cfg(test)]
use super::super::common::dispatch_envelope;

#[cfg(test)]
pub(crate) fn dispatch_asset_search(
    runtime: &EditorEventRuntime,
    query: impl Into<String>,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::Slint,
            EditorEvent::Asset(EditorAssetEvent::SetSearchQuery {
                query: query.into(),
            }),
        ),
    )
}
