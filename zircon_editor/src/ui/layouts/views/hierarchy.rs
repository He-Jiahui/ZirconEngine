use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::view_projection::build_view_template_nodes;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::workbench::snapshot::SceneEntry;
use zircon_runtime_interface::ui::layout::UiSize;

use super::ViewTemplateNodeData;

const HIERARCHY_LAYOUT_ASSET_PATH: &str = "/assets/ui/editor/hierarchy.zui";
const HIERARCHY_STYLE_ASSET_PATH: &str = "/assets/ui/theme/editor_base.zui";
const HIERARCHY_STYLE_ASSET_ID: &str = "res://ui/theme/editor_base.zui";
const HIERARCHY_HEADER_PANEL: &str = "HierarchyHeaderPanel";
const HIERARCHY_LIST_PANEL: &str = "HierarchyListPanel";
const HIERARCHY_EMPTY_STATE_MESSAGE: &str = "HierarchyEmptyStateMessage";

pub(crate) fn hierarchy_pane_nodes(
    entries: &[SceneEntry],
    size: UiSize,
) -> ModelRc<ViewTemplateNodeData> {
    let mut text_overrides = BTreeMap::new();
    let active_entry = entries.iter().find(|entry| entry.selected);
    text_overrides.insert(HIERARCHY_HEADER_PANEL.to_string(), "Hierarchy".to_string());
    text_overrides.insert(
        HIERARCHY_EMPTY_STATE_MESSAGE.to_string(),
        if entries.is_empty() {
            "No scene nodes".to_string()
        } else {
            String::new()
        },
    );

    let mut nodes = build_view_template_nodes(
        "hierarchy.template_projection",
        HIERARCHY_LAYOUT_ASSET_PATH,
        &[(HIERARCHY_STYLE_ASSET_ID, HIERARCHY_STYLE_ASSET_PATH)],
        size,
        &text_overrides,
    )
    .unwrap_or_default();
    apply_hierarchy_visual_state(&mut nodes, active_entry.is_some());
    model_rc(nodes)
}

fn apply_hierarchy_visual_state(nodes: &mut [ViewTemplateNodeData], has_selection: bool) {
    if let Some(node) = nodes
        .iter_mut()
        .find(|node| node.control_id == HIERARCHY_LIST_PANEL)
    {
        node.selected = has_selection;
        node.focused = false;
        node.surface_variant = if has_selection {
            "panel".into()
        } else {
            "inset".into()
        };
        node.text_tone = if has_selection {
            "default".into()
        } else {
            "muted".into()
        };
    }

    if let Some(node) = nodes
        .iter_mut()
        .find(|node| node.control_id == HIERARCHY_HEADER_PANEL)
    {
        node.focused = false;
        node.surface_variant = "inset".into();
        node.text_tone = "default".into();
    }

    if let Some(node) = nodes
        .iter_mut()
        .find(|node| node.control_id == HIERARCHY_EMPTY_STATE_MESSAGE)
    {
        node.focused = false;
        node.text_tone = "muted".into();
    }
}
