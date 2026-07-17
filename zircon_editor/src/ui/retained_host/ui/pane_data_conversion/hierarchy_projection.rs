use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    HierarchyPaneViewData, PaneContentSize, PaneData, PanePayload, SceneNodeData,
};
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::template_runtime::EditorUiHostRuntime;

use super::super::template_node_conversion::to_host_contract_template_node;
use super::model_projection::map_model_rc;
use super::template_node_projection::project_nodes;

pub(super) fn to_host_contract_hierarchy_pane(
    data: &HierarchyPaneViewData,
) -> host_contract::HierarchyPaneData {
    host_contract::HierarchyPaneData {
        nodes: project_nodes(&data.nodes, to_host_contract_template_node),
        hierarchy_nodes: to_host_contract_scene_nodes(&data.hierarchy_nodes),
    }
}

pub(crate) fn to_host_contract_hierarchy_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::HierarchyPaneData {
    hierarchy_template_projection(data, content_size, None)
        .unwrap_or_else(|| to_host_contract_hierarchy_pane(&data.native_body.hierarchy))
}

pub(crate) fn to_host_contract_hierarchy_pane_from_host_pane_with_runtime(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> host_contract::HierarchyPaneData {
    hierarchy_template_projection(data, content_size, Some(runtime))
        .unwrap_or_else(|| to_host_contract_hierarchy_pane(&data.native_body.hierarchy))
}

fn hierarchy_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
) -> Option<host_contract::HierarchyPaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    let PanePayload::HierarchyV1(payload) = &presentation.body.payload else {
        return None;
    };

    let mut nodes =
        super::project_pane_template_nodes_with_runtime(&presentation.body, content_size, runtime)?;
    apply_hierarchy_template_state(
        &mut nodes,
        !payload.nodes.is_empty(),
        payload.nodes.iter().any(|node| node.selected),
    );
    Some(host_contract::HierarchyPaneData {
        nodes: model_rc(nodes),
        hierarchy_nodes: model_rc(
            payload
                .nodes
                .iter()
                .map(|node| host_contract::SceneNodeData {
                    id: node.node_id.to_string().into(),
                    name: node.name.clone().into(),
                    depth: i32::try_from(node.depth).unwrap_or(i32::MAX),
                    selected: node.selected,
                })
                .collect(),
        ),
    })
}

fn to_host_contract_scene_node(node: &SceneNodeData) -> host_contract::SceneNodeData {
    host_contract::SceneNodeData {
        id: node.id.clone(),
        name: node.name.clone(),
        depth: node.depth,
        selected: node.selected,
    }
}

fn to_host_contract_scene_nodes(
    nodes: &ModelRc<SceneNodeData>,
) -> ModelRc<host_contract::SceneNodeData> {
    map_model_rc(nodes, to_host_contract_scene_node)
}

fn apply_hierarchy_template_state(
    nodes: &mut [host_contract::TemplatePaneNodeData],
    has_nodes: bool,
    has_selection: bool,
) {
    if let Some(panel) = nodes
        .iter_mut()
        .find(|node| node.control_id.as_str() == "HierarchyListPanel")
    {
        panel.selected = has_selection;
        panel.focused = has_selection;
        panel.surface_variant = if has_selection {
            "panel".into()
        } else {
            "inset".into()
        };
        panel.text_tone = if has_selection {
            "default".into()
        } else {
            "muted".into()
        };
        panel.disabled = !has_nodes;
    }

    if let Some(select_root) = nodes
        .iter_mut()
        .find(|node| node.control_id.as_str() == "SelectRoot")
    {
        select_root.selected = has_selection;
        select_root.focused = has_selection;
        select_root.surface_variant = if has_selection {
            "panel".into()
        } else {
            "inset".into()
        };
        select_root.text_tone = if has_nodes {
            "default".into()
        } else {
            "muted".into()
        };
        select_root.disabled = !has_nodes;
    }
}
