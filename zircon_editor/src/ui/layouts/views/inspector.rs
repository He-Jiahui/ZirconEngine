use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::build_view_template_nodes;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::snapshot::InspectorSnapshot;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const INSPECTOR_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/inspector.v2.ui.toml";
const INSPECTOR_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.v2.ui.toml";
const INSPECTOR_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.v2.ui.toml";

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
        "InspectorSeparatorRow".to_string(),
        inspector
            .map(|inspector| format!("{} plugin components", inspector.plugin_components.len()))
            .unwrap_or_else(|| "0 plugin components".to_string()),
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
}

fn mark_panel(nodes: &mut [ViewTemplateNodeData], control_id: &str, active: bool) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = active;
        node.focused = active;
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
