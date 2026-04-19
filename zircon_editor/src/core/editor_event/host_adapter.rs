use crate::ui::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use zircon_runtime::scene::NodeId;

use crate::{dispatch_workbench_binding, LayoutCommand};

#[cfg(test)]
use super::EditorAssetEvent;
use super::{EditorEvent, EditorEventEnvelope, EditorEventSource, EditorViewportEvent};

pub fn slint_menu_action(action_id: &str) -> Result<EditorEventEnvelope, String> {
    if let Some(name) = action_id.strip_prefix("SavePreset.") {
        let name = if name.is_empty() { "current" } else { name };
        return Ok(EditorEventEnvelope::new(
            EditorEventSource::Slint,
            EditorEvent::Layout(LayoutCommand::SavePreset {
                name: name.to_string(),
            }),
        ));
    }

    if let Some(name) = action_id.strip_prefix("LoadPreset.") {
        let name = if name.is_empty() { "current" } else { name };
        return Ok(EditorEventEnvelope::new(
            EditorEventSource::Slint,
            EditorEvent::Layout(LayoutCommand::LoadPreset {
                name: name.to_string(),
            }),
        ));
    }

    let binding = EditorUiBinding::new(
        "WorkbenchMenuBar",
        action_id,
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action(action_id),
    );
    let event = dispatch_workbench_binding(&binding).map_err(|error| error.to_string())?;
    let crate::WorkbenchHostEvent::Menu(action) = event;
    Ok(EditorEventEnvelope::new(
        EditorEventSource::Slint,
        EditorEvent::WorkbenchMenu(action),
    ))
}

pub fn slint_hierarchy_selection(node_id: NodeId) -> EditorEventEnvelope {
    EditorEventEnvelope::new(
        EditorEventSource::Slint,
        EditorEvent::Selection(crate::SelectionHostEvent::SelectSceneNode { node_id }),
    )
}

#[cfg(test)]
pub fn slint_asset_item_selection(asset_uuid: impl Into<String>) -> EditorEventEnvelope {
    EditorEventEnvelope::new(
        EditorEventSource::Slint,
        EditorEvent::Asset(EditorAssetEvent::SelectItem {
            asset_uuid: asset_uuid.into(),
        }),
    )
}

#[cfg(test)]
pub fn slint_asset_search(query: impl Into<String>) -> EditorEventEnvelope {
    EditorEventEnvelope::new(
        EditorEventSource::Slint,
        EditorEvent::Asset(EditorAssetEvent::SetSearchQuery {
            query: query.into(),
        }),
    )
}

pub fn slint_viewport(event: EditorViewportEvent) -> EditorEventEnvelope {
    EditorEventEnvelope::new(EditorEventSource::Slint, EditorEvent::Viewport(event))
}
