use std::collections::BTreeMap;

use crate::ui::layouts::views::view_projection::{
    build_view_template_node_projection_with_patches, ViewTemplateNodePatch,
};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::snapshot::InspectorSnapshot;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const INSPECTOR_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/inspector.zui";
const INSPECTOR_EMPTY_STATE_CONTROL_ID: &str = "InspectorEmptyState";
const INSPECTOR_EMPTY_STATE_MESSAGE_CONTROL_ID: &str = "InspectorEmptyStateMessage";
const INSPECTOR_NAME_VALUE_CONTROL_ID: &str = "InspectorNameValue";
const INSPECTOR_PARENT_VALUE_CONTROL_ID: &str = "InspectorParentValue";
const INSPECTOR_POSITION_VALUE_CONTROL_ID: &str = "InspectorPositionValue";
const INSPECTOR_COMPONENTS_VALUE_CONTROL_ID: &str = "InspectorComponentsValue";

pub(crate) fn inspector_pane_nodes(
    inspector: Option<&InspectorSnapshot>,
    size: UiSize,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    text_overrides.insert(
        INSPECTOR_NAME_VALUE_CONTROL_ID.to_string(),
        inspector
            .map(|inspector| inspector.name.clone())
            .unwrap_or_else(|| "-".to_string()),
    );
    text_overrides.insert(
        INSPECTOR_PARENT_VALUE_CONTROL_ID.to_string(),
        inspector
            .map(|inspector| inspector.parent.clone())
            .unwrap_or_else(|| "-".to_string()),
    );
    text_overrides.insert(
        INSPECTOR_POSITION_VALUE_CONTROL_ID.to_string(),
        inspector
            .map(|inspector| {
                format!(
                    "{}, {}, {}",
                    inspector.translation[0], inspector.translation[1], inspector.translation[2]
                )
            })
            .unwrap_or_else(|| "-".to_string()),
    );
    text_overrides.insert(
        INSPECTOR_COMPONENTS_VALUE_CONTROL_ID.to_string(),
        inspector
            .map(|inspector| inspector.plugin_components.len().to_string())
            .unwrap_or_else(|| "-".to_string()),
    );
    text_overrides.insert(
        INSPECTOR_EMPTY_STATE_MESSAGE_CONTROL_ID.to_string(),
        inspector
            .map(|_| String::new())
            .unwrap_or_else(|| "No object selected".to_string()),
    );

    let node_patches = inspector_visual_state_patches(inspector.is_some());
    let Ok(projection) = build_view_template_node_projection_with_patches(
        "inspector.template_projection",
        INSPECTOR_LAYOUT_ASSET_PATH,
        &[],
        size,
        &text_overrides,
        &node_patches,
    ) else {
        return ModelRc::default();
    };
    projection.into_model()
}

fn inspector_visual_state_patches(has_selection: bool) -> BTreeMap<String, ViewTemplateNodePatch> {
    let mut patches = BTreeMap::new();
    mark_readout(&mut patches, INSPECTOR_NAME_VALUE_CONTROL_ID, has_selection);
    mark_readout(
        &mut patches,
        INSPECTOR_PARENT_VALUE_CONTROL_ID,
        has_selection,
    );
    mark_readout(
        &mut patches,
        INSPECTOR_POSITION_VALUE_CONTROL_ID,
        has_selection,
    );
    mark_readout(
        &mut patches,
        INSPECTOR_COMPONENTS_VALUE_CONTROL_ID,
        has_selection,
    );
    mark_empty_state(&mut patches, has_selection);
    patches
}

fn mark_empty_state(patches: &mut BTreeMap<String, ViewTemplateNodePatch>, has_selection: bool) {
    patches.insert(
        INSPECTOR_EMPTY_STATE_CONTROL_ID.to_string(),
        ViewTemplateNodePatch {
            selected: Some(false),
            focused: Some(false),
            surface_variant: Some(
                if has_selection {
                    "transparent"
                } else {
                    "inset"
                }
                .to_string(),
            ),
            ..ViewTemplateNodePatch::default()
        },
    );
}

fn mark_readout(
    patches: &mut BTreeMap<String, ViewTemplateNodePatch>,
    control_id: &str,
    active: bool,
) {
    patches.insert(
        control_id.to_string(),
        ViewTemplateNodePatch {
            selected: Some(false),
            text_tone: Some(if active { "default" } else { "muted" }.to_string()),
            ..ViewTemplateNodePatch::default()
        },
    );
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
        let Some(name) = node_by_control_id(&nodes, INSPECTOR_NAME_VALUE_CONTROL_ID) else {
            return;
        };
        let Some(message) = node_by_control_id(&nodes, "InspectorEmptyStateMessage") else {
            return;
        };

        assert_eq!(header.text.to_string(), "Inspector");
        assert_eq!(header.text_tone.to_string(), "default");
        assert_eq!(header.surface_variant.to_string(), "transparent");
        assert!(!header.selected);
        assert!(!header.focused);
        assert_eq!(
            header.frame.height,
            EditorDensityTokens::WORKBENCH_ROW_HEIGHT
        );
        assert_eq!(name.frame.height, EditorDensityTokens::WORKBENCH_ROW_HEIGHT);
        assert!(!name.selected);
        assert_eq!(name.value_text.to_string(), "-");
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
            render_layer_mask: 1,
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

        assert!(!header.selected);
        assert!(!header.focused);
        assert_eq!(empty_state.surface_variant.to_string(), "transparent");
        assert!(message.text.is_empty());

        for (control_id, expected_value) in [
            (INSPECTOR_NAME_VALUE_CONTROL_ID, "Camera"),
            (INSPECTOR_PARENT_VALUE_CONTROL_ID, "Root"),
            (INSPECTOR_POSITION_VALUE_CONTROL_ID, "1.0, 2.0, 3.0"),
            (INSPECTOR_COMPONENTS_VALUE_CONTROL_ID, "0"),
        ] {
            let Some(readout) = node_by_control_id(&nodes, control_id) else {
                return;
            };
            assert_eq!(readout.value_text.to_string(), expected_value);
            assert!(
                !readout.selected,
                "{control_id} must not impersonate selection"
            );
        }
    }
}
