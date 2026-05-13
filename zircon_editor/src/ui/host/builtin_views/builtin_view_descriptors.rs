use crate::ui::workbench::view::ViewDescriptor;

use super::super::asset_editor_sessions::ui_asset_editor_view_descriptor;
use super::super::editor_capabilities::EditorCapabilitySnapshot;
use super::super::editor_subsystems::{
    EDITOR_SUBSYSTEM_ANIMATION_AUTHORING, EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING,
    EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS, EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
};
use super::super::startup::welcome_view_descriptor;
use super::activity_views::activity_view_descriptors::activity_view_descriptors;
use super::activity_windows::activity_window_descriptors::activity_window_descriptors;

pub(crate) fn builtin_view_descriptors(snapshot: &EditorCapabilitySnapshot) -> Vec<ViewDescriptor> {
    let mut descriptors = activity_view_descriptors();
    descriptors.extend(activity_window_descriptors());
    descriptors.push(ui_asset_editor_view_descriptor());
    descriptors.push(welcome_view_descriptor());
    descriptors
        .into_iter()
        .map(with_builtin_required_capabilities)
        .filter(|descriptor| snapshot.allows_all(&descriptor.required_capabilities))
        .collect()
}

pub(crate) fn with_builtin_required_capabilities(descriptor: ViewDescriptor) -> ViewDescriptor {
    let capability = match descriptor.descriptor_id.0.as_str() {
        "editor.animation_sequence" | "editor.animation_graph" => {
            Some(EDITOR_SUBSYSTEM_ANIMATION_AUTHORING)
        }
        "editor.animation_timeline" | "editor.animation.timeline" | "editor.animation.graph" => {
            Some(EDITOR_SUBSYSTEM_ANIMATION_AUTHORING)
        }
        "editor.ui_asset"
        | "editor.ui_component_showcase"
        | "editor.material_demo_window"
        | "editor.ui.designer"
        | "editor.ui.source" => Some(EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING),
        "editor.runtime_diagnostics" | "editor.debug_observatory" => {
            Some(EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS)
        }
        "editor.diagnostics_window" => Some(EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS),
        "editor.animation_editor_window" => Some(EDITOR_SUBSYSTEM_ANIMATION_AUTHORING),
        "editor.ui_asset_editor_window" => Some(EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING),
        "editor.workbench_window"
        | "editor.prefab"
        | "editor.prefab_editor_window"
        | "editor.material_editor_window" => Some(EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING),
        _ => None,
    };
    if let Some(capability) = capability {
        descriptor.with_required_capabilities([capability])
    } else {
        descriptor
    }
}
