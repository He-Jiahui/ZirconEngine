use zircon_scene::NodeId;

use crate::editor_event::{
    host_adapter, EditorAssetEvent, EditorAssetSurface, EditorAssetUtilityTab,
    EditorAssetViewMode, EditorEvent, EditorEventEnvelope, EditorEventRuntime, EditorEventSource,
    EditorInspectorEvent, EditorViewportEvent,
};
use crate::LayoutCommand;

use super::event_bridge::{apply_record_effects, SlintDispatchEffects};

fn dispatch_envelope(
    runtime: &EditorEventRuntime,
    envelope: EditorEventEnvelope,
) -> Result<SlintDispatchEffects, String> {
    let record = runtime.dispatch_envelope(envelope)?;
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(&mut effects, &record);
    Ok(effects)
}

pub(crate) fn dispatch_menu_action(
    runtime: &EditorEventRuntime,
    action: &str,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_menu_action(action)?)
}

pub(crate) fn dispatch_layout_command(
    runtime: &EditorEventRuntime,
    command: LayoutCommand,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(EditorEventSource::Slint, crate::EditorEvent::Layout(command)),
    )
}

pub(crate) fn dispatch_hierarchy_selection(
    runtime: &EditorEventRuntime,
    node_id: NodeId,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_hierarchy_selection(node_id))
}

pub(crate) fn dispatch_asset_folder_selection(
    runtime: &EditorEventRuntime,
    folder_id: impl Into<String>,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_asset_folder_selection(folder_id))
}

pub(crate) fn dispatch_asset_item_selection(
    runtime: &EditorEventRuntime,
    asset_uuid: impl Into<String>,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_asset_item_selection(asset_uuid))
}

pub(crate) fn dispatch_asset_search(
    runtime: &EditorEventRuntime,
    query: impl Into<String>,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_asset_search(query))
}

pub(crate) fn dispatch_asset_kind_filter(
    runtime: &EditorEventRuntime,
    kind: Option<String>,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_asset_kind_filter(kind))
}

pub(crate) fn dispatch_asset_view_mode(
    runtime: &EditorEventRuntime,
    surface: EditorAssetSurface,
    view_mode: EditorAssetViewMode,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_asset_view_mode(surface, view_mode))
}

pub(crate) fn dispatch_asset_utility_tab(
    runtime: &EditorEventRuntime,
    surface: EditorAssetSurface,
    tab: EditorAssetUtilityTab,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_asset_utility_tab(surface, tab))
}

pub(crate) fn dispatch_asset_activate_reference(
    runtime: &EditorEventRuntime,
    asset_uuid: impl Into<String>,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::Slint,
            EditorEvent::Asset(EditorAssetEvent::ActivateReference {
                asset_uuid: asset_uuid.into(),
            }),
        ),
    )
}

pub(crate) fn dispatch_open_asset_browser(
    runtime: &EditorEventRuntime,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_open_asset_browser())
}

pub(crate) fn dispatch_locate_selected_asset(
    runtime: &EditorEventRuntime,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_locate_selected_asset())
}

pub(crate) fn dispatch_viewport_event(
    runtime: &EditorEventRuntime,
    event: EditorViewportEvent,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(runtime, host_adapter::slint_viewport(event))
}

pub(crate) fn dispatch_inspector_apply(
    runtime: &EditorEventRuntime,
    event: EditorInspectorEvent,
) -> Result<SlintDispatchEffects, String> {
    dispatch_envelope(
        runtime,
        EditorEventEnvelope::new(
            EditorEventSource::Slint,
            crate::EditorEvent::Inspector(event),
        ),
    )
}
