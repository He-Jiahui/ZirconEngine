use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::build_view_template_nodes;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::snapshot::InspectorSnapshot;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const INSPECTOR_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/inspector.zui";
const INSPECTOR_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.zui";
const INSPECTOR_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.zui";
const INSPECTOR_EMPTY_STATE_CONTROL_ID: &str = "InspectorEmptyState";
const INSPECTOR_EMPTY_STATE_MESSAGE_CONTROL_ID: &str = "InspectorEmptyStateMessage";

pub(crate) fn inspector_pane_nodes(
    inspector: Option<&InspectorSnapshot>,
    size: UiSize,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    text_overrides.insert(
        "InspectorHeaderPanel".to_string(),
        inspector
            .map(|inspector| format!("Inspector • {}", inspector.name))
            .unwrap_or_else(|| "Inspector • No selection".to_string()),
    );
    text_overrides.insert(
        "InspectorNameRow".to_string(),
        inspector
            .map(|inspector| format!("Name • {}", inspector.name))
            .unwrap_or_else(|| "Name • -".to_string()),
    );
    text_overrides.insert(
        "InspectorParentRow".to_string(),
        inspector
            .map(|inspector| format!("Parent • {}", inspector.parent))
            .unwrap_or_else(|| "Parent • -".to_string()),
    );
    text_overrides.insert(
        "InspectorPositionRow".to_string(),
        inspector
            .map(|inspector| {
                format!(
                    "Position • {}, {}, {}",
                    inspector.translation[0], inspector.translation[1], inspector.translation[2]
                )
            })
            .unwrap_or_else(|| "Position • -, -, -".to_string()),
    );
    text_overrides.insert(
        "InspectorActionsRow".to_string(),
        inspector
            .map(|inspector| format!("{} components", inspector.plugin_components.len()))
            .unwrap_or_else(|| "No components".to_string()),
    );
    text_overrides.insert(
        INSPECTOR_EMPTY_STATE_MESSAGE_CONTROL_ID.to_string(),
        inspector
            .map(|_| String::new())
            .unwrap_or_else(|| "No object selected".to_string()),
    );

    let mut nodes = build_view_template_nodes(
        "inspector.template_projection",
        INSPECTOR_LAYOUT_ASSET_PATH,
        &[(INSPECTOR_STYLE_ASSET_ID, INSPECTOR_STYLE_ASSET_PATH)],
        size,
        &text_overrides,
    )
    .unwrap_or_default();
    apply_inspector_visual_state(&mut nodes, inspector.is_some());
    model_rc(nodes)
}

fn apply_inspector_visual_state(nodes: &mut [ViewTemplateNodeData], has_selection: bool) {
    mark_panel(nodes, "InspectorHeaderPanel", has_selection);
    mark_panel(nodes, "InspectorActionsRow", has_selection);
    mark_row(nodes, "InspectorNameRow", has_selection);
    mark_row(nodes, "InspectorParentRow", has_selection);
    mark_row(nodes, "InspectorPositionRow", has_selection);
    mark_row(nodes, "InspectorSeparatorRow", has_selection);
    mark_empty_state(nodes, has_selection);
}

fn mark_panel(nodes: &mut [ViewTemplateNodeData], control_id: &str, active: bool) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = active;
        node.focused = false;
        node.surface_variant = if active {
            "panel".into()
        } else {
            "inset".into()
        };
        node.text_tone = if active {
            "default".into()
        } else {
            "muted".into()
        };
    }
}

fn mark_empty_state(nodes: &mut [ViewTemplateNodeData], has_selection: bool) {
    if let Some(node) = nodes
        .iter_mut()
        .find(|node| node.control_id == INSPECTOR_EMPTY_STATE_CONTROL_ID)
    {
        node.selected = false;
        node.focused = false;
        node.surface_variant = if has_selection {
            "frame_only".into()
        } else {
            "inset".into()
        };
    }
}

fn mark_row(nodes: &mut [ViewTemplateNodeData], control_id: &str, active: bool) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = active;
        node.text_tone = if active {
            "default".into()
        } else {
            "muted".into()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::design_tokens::EditorDensityTokens;

    fn projected_nodes(inspector: Option<&InspectorSnapshot>) -> Vec<ViewTemplateNodeData> {
        let pane = inspector_pane_nodes(inspector, UiSize::new(360.0, 520.0));
        (0..pane.row_count())
            .filter_map(|row| pane.row_data(row))
            .collect()
    }

    fn node_by_control_id<'a>(
        nodes: &'a [ViewTemplateNodeData],
        control_id: &str,
    ) -> Option<&'a ViewTemplateNodeData> {
        nodes.iter().find(|node| node.control_id == control_id)
    }

    #[test]
    fn no_selection_projects_a_muted_centered_empty_state() {
        let nodes = projected_nodes(None);

        assert!(nodes
            .iter()
            .any(|node| node.control_id == "InspectorEmptyState"));
        assert!(nodes
            .iter()
            .any(|node| node.control_id == "InspectorEmptyStateMessage"));

        let Some(header) = node_by_control_id(&nodes, "InspectorHeaderPanel") else {
            return;
        };
        let Some(empty_state) = node_by_control_id(&nodes, "InspectorEmptyState") else {
            return;
        };
        let Some(name) = node_by_control_id(&nodes, "InspectorNameRow") else {
            return;
        };
        let Some(message) = node_by_control_id(&nodes, "InspectorEmptyStateMessage") else {
            return;
        };

        assert_eq!(header.text.to_string(), "Inspector • No selection");
        assert_eq!(header.text_tone.to_string(), "muted");
        assert!(!header.selected);
        assert!(!header.focused);
        assert_eq!(
            header.frame.height,
            EditorDensityTokens::WORKBENCH_ROW_HEIGHT
        );
        assert_eq!(name.frame.height, EditorDensityTokens::WORKBENCH_ROW_HEIGHT);
        assert_eq!(empty_state.surface_variant.to_string(), "inset");
        assert_eq!(message.text.to_string(), "No object selected");
        assert_eq!(message.text_align.to_string(), "center");
        assert!(empty_state.frame.height > 120.0);
    }

    #[test]
    fn selection_hides_empty_state_without_synthesizing_keyboard_focus() {
        let inspector = InspectorSnapshot {
            id: zircon_runtime::scene::NodeId::default(),
            name: "Camera".to_string(),
            parent: "Root".to_string(),
            translation: ["1.0".to_string(), "2.0".to_string(), "3.0".to_string()],
            scale: ["1.0".to_string(), "1.0".to_string(), "1.0".to_string()],
            plugin_components: Vec::new(),
        };
        let nodes = projected_nodes(Some(&inspector));

        let Some(header) = node_by_control_id(&nodes, "InspectorHeaderPanel") else {
            return;
        };
        let Some(empty_state) = node_by_control_id(&nodes, "InspectorEmptyState") else {
            return;
        };
        let Some(message) = node_by_control_id(&nodes, "InspectorEmptyStateMessage") else {
            return;
        };

        assert!(header.selected);
        assert!(!header.focused);
        assert_eq!(empty_state.surface_variant.to_string(), "frame_only");
        assert!(message.text.is_empty());
    }
}
