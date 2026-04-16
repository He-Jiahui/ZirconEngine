use zircon_editor_ui::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use zircon_scene::NodeId;

use crate::{dispatch_workbench_binding, LayoutCommand, MenuAction};

use super::{
    EditorAssetEvent, EditorAssetSurface, EditorAssetUtilityTab, EditorAssetViewMode, EditorEvent,
    EditorEventEnvelope, EditorEventSource, EditorEventTransient, EditorViewportEvent,
};

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

pub fn slint_asset_folder_selection(folder_id: impl Into<String>) -> EditorEventEnvelope {
    EditorEventEnvelope::new(
        EditorEventSource::Slint,
        EditorEvent::Asset(EditorAssetEvent::SelectFolder {
            folder_id: folder_id.into(),
        }),
    )
}

pub fn slint_asset_item_selection(asset_uuid: impl Into<String>) -> EditorEventEnvelope {
    EditorEventEnvelope::new(
        EditorEventSource::Slint,
        EditorEvent::Asset(EditorAssetEvent::SelectItem {
            asset_uuid: asset_uuid.into(),
        }),
    )
}

pub fn slint_asset_search(query: impl Into<String>) -> EditorEventEnvelope {
    EditorEventEnvelope::new(
        EditorEventSource::Slint,
        EditorEvent::Asset(EditorAssetEvent::SetSearchQuery {
            query: query.into(),
        }),
    )
}

pub fn slint_asset_kind_filter(kind: Option<String>) -> EditorEventEnvelope {
    EditorEventEnvelope::new(
        EditorEventSource::Slint,
        EditorEvent::Asset(EditorAssetEvent::SetKindFilter { kind }),
    )
}

pub fn slint_asset_view_mode(
    surface: EditorAssetSurface,
    view_mode: EditorAssetViewMode,
) -> EditorEventEnvelope {
    EditorEventEnvelope::new(
        EditorEventSource::Slint,
        EditorEvent::Asset(EditorAssetEvent::SetViewMode { surface, view_mode }),
    )
}

pub fn slint_asset_utility_tab(
    surface: EditorAssetSurface,
    tab: EditorAssetUtilityTab,
) -> EditorEventEnvelope {
    EditorEventEnvelope::new(
        EditorEventSource::Slint,
        EditorEvent::Asset(EditorAssetEvent::SetUtilityTab { surface, tab }),
    )
}

pub fn slint_open_asset_browser() -> EditorEventEnvelope {
    EditorEventEnvelope::new(
        EditorEventSource::Slint,
        EditorEvent::Asset(EditorAssetEvent::OpenAssetBrowser),
    )
}

pub fn slint_locate_selected_asset() -> EditorEventEnvelope {
    EditorEventEnvelope::new(
        EditorEventSource::Slint,
        EditorEvent::Asset(EditorAssetEvent::LocateSelectedAsset),
    )
}

pub fn slint_viewport(event: EditorViewportEvent) -> EditorEventEnvelope {
    EditorEventEnvelope::new(EditorEventSource::Slint, EditorEvent::Viewport(event))
}

pub fn slint_transient(update: EditorEventTransient) -> EditorEventEnvelope {
    EditorEventEnvelope::new(EditorEventSource::Slint, EditorEvent::Transient(update))
}

pub fn headless_menu_action(action: MenuAction) -> EditorEventEnvelope {
    EditorEventEnvelope::new(
        EditorEventSource::Headless,
        EditorEvent::WorkbenchMenu(action),
    )
}
