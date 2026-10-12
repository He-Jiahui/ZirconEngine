//! Live semantic state over source-owned toolbar geometry and style.
use super::data::{HostPaneInteractionStateData, SceneViewportChromeData, TemplatePaneNodeData};
use crate::ui::retained_host::primitives::ModelRc;

pub(super) fn control_enabled(viewport: &SceneViewportChromeData, source_id: &str) -> bool {
    match source_id {
        "EnterPlayMode" => viewport.toolbar_enter_play_enabled,
        "ExitPlayMode" => viewport.toolbar_exit_play_enabled,
        _ => true,
    }
}

pub(super) fn mounted_control_id(viewport: &SceneViewportChromeData, source_id: &str) -> String {
    format!("{}::{}", viewport.toolbar_surface_key, source_id)
}

pub(super) fn live_toolbar_nodes(
    viewport: &SceneViewportChromeData,
    interaction: Option<&HostPaneInteractionStateData>,
) -> ModelRc<TemplatePaneNodeData> {
    viewport
        .toolbar_template_nodes
        .map_preserving_metadata(|source| {
            let mut node = source.clone();
            let source_id = source.control_id.as_str();
            let (value, selected) = match source_id {
                "ActivateSceneMode" => (
                    viewport.mode.to_string(),
                    viewport.mode.as_str() != "Select",
                ),
                "SetTransformSpace" => (
                    viewport.transform_space.to_string(),
                    viewport.transform_space.as_str() != "Local",
                ),
                "SetPivotMode" => (
                    viewport.pivot_mode.to_string(),
                    viewport.pivot_mode.as_str() != "Centroid",
                ),
                "SetDisplayMode" => (
                    viewport.display_mode.to_string(),
                    viewport.display_mode.as_str() != "Shaded",
                ),
                "SetGridMode" => (
                    viewport.grid_mode.to_string(),
                    viewport.grid_mode.as_str() != "VisibleNoSnap",
                ),
                "SetProjectionMode" => (
                    viewport.projection_mode.to_string(),
                    viewport.projection_mode.as_str() == "Orthographic",
                ),
                "AlignView" => (
                    viewport.view_orientation.to_string(),
                    viewport.view_orientation.as_str() != "NegZ",
                ),
                "SetTranslateSnap" => (
                    viewport.translate_snap_label.to_string(),
                    viewport.translate_snap > 0.0,
                ),
                "SetRotateSnapDegrees" => (
                    viewport.rotate_snap_label.to_string(),
                    viewport.rotate_snap_deg > 0.0,
                ),
                "SetScaleSnap" => (
                    viewport.scale_snap_label.to_string(),
                    viewport.scale_snap > 0.0,
                ),
                "SetPreviewLighting" => (
                    viewport.preview_lighting.to_string(),
                    viewport.preview_lighting,
                ),
                "SetPreviewSkybox" => {
                    (viewport.preview_skybox.to_string(), viewport.preview_skybox)
                }
                "SetGizmosEnabled" => {
                    (viewport.gizmos_enabled.to_string(), viewport.gizmos_enabled)
                }
                "EnterPlayMode" => (
                    viewport.toolbar_is_playing.to_string(),
                    viewport.toolbar_is_playing,
                ),
                "ExitPlayMode" => (
                    viewport.toolbar_is_playing.to_string(),
                    viewport.toolbar_is_playing,
                ),
                _ => (
                    source.value_text.to_string(),
                    source.selected || source.checked,
                ),
            };
            node.value_text = value.into();
            node.selected = selected;
            node.checked = selected;
            node.disabled |= !control_enabled(viewport, source_id);
            node.control_id = mounted_control_id(viewport, source_id).into();
            node.node_id = format!("{}::{}", viewport.toolbar_surface_key, source.node_id).into();
            node.instance_path =
                format!("{}::{}", viewport.toolbar_surface_key, source.instance_path).into();
            if !source.parent_node_id.is_empty() {
                node.parent_node_id = format!(
                    "{}::{}",
                    viewport.toolbar_surface_key, source.parent_node_id
                )
                .into();
            }
            if !source.parent_instance_path.is_empty() {
                node.parent_instance_path = format!(
                    "{}::{}",
                    viewport.toolbar_surface_key, source.parent_instance_path
                )
                .into();
            }
            if let Some(interaction) = interaction {
                node.focused |= node.control_id == interaction.focused_template_control_id;
                node.focus_visible_known = true;
                node.focus_visible = node.focused && interaction.template_focus_visible;
                node.pressed |= node.control_id == interaction.pressed_template_control_id;
            }
            node
        })
}

#[cfg(test)]
#[path = "tests/viewport_chrome_state.rs"]
mod tests;
