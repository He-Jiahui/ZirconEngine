use std::sync::OnceLock;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    PaneBodyPresentation, PaneContentSize, PaneData, PanePayload,
};
use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::{EditorUiHostRuntime, RetainedUiHostNodeProjection};
use zircon_runtime_interface::ui::layout::UiSize;

use super::pane_component_projection::host_template_node;

pub(crate) fn to_host_contract_template_v2_pane_from_host_pane_with_runtime(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
) -> host_contract::TemplateV2PaneData {
    let Some(presentation) = data.pane_presentation.as_ref() else {
        clear_template_actions_for_pane(runtime, data.id.as_str());
        return host_contract::TemplateV2PaneData::default();
    };
    if !matches!(&presentation.body.payload, PanePayload::TemplateV2(_)) {
        clear_template_actions_for_pane(runtime, data.id.as_str());
        return host_contract::TemplateV2PaneData::default();
    }

    project_pane_template_nodes_with_runtime_for_pane(
        &presentation.body,
        content_size,
        runtime,
        Some(data.id.as_str()),
    )
    .map(|nodes| host_contract::TemplateV2PaneData {
        nodes: model_rc(nodes),
    })
    .unwrap_or_default()
}

pub(super) fn project_pane_template_nodes(
    body: &PaneBodyPresentation,
    content_size: PaneContentSize,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    project_pane_template_nodes_with_runtime(body, content_size, None)
}

pub(super) fn project_pane_template_nodes_with_runtime(
    body: &PaneBodyPresentation,
    content_size: PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    project_pane_template_nodes_with_runtime_for_pane(body, content_size, runtime, None)
}

fn project_pane_template_nodes_with_runtime_for_pane(
    body: &PaneBodyPresentation,
    content_size: PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
    pane_id: Option<&str>,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    let runtime = pane_template_runtime(runtime)?;
    let nodes = (|| {
        let projection = runtime.project_pane_body(body).ok()?;
        let mut surface = runtime.build_shared_surface(&body.document_id).ok()?;
        runtime
            .apply_pane_component_patches_to_surface(body, &mut surface)
            .ok()?;
        surface
            .compute_layout(UiSize::new(
                content_size.width.max(0.0),
                content_size.height.max(0.0),
            ))
            .ok()?;
        let mut host_model = runtime
            .build_host_model_with_surface(&projection, &surface)
            .ok()?;
        if let Some(pane_id) = pane_id {
            runtime
                .bind_template_actions_for_pane(pane_id, &mut surface, &mut host_model)
                .ok()?;
        }

        Some(
            host_model
                .nodes
                .into_iter()
                .filter_map(|node| host_template_node_with_content_fallback(node, content_size))
                .collect::<Vec<_>>(),
        )
    })();
    if nodes.is_none() {
        if let Some(pane_id) = pane_id {
            runtime.remove_template_actions_for_pane(pane_id);
        }
    }
    nodes
}

fn clear_template_actions_for_pane(runtime: Option<&EditorUiHostRuntime>, pane_id: &str) {
    if let Some(runtime) = runtime {
        runtime.remove_template_actions_for_pane(pane_id);
    }
}

pub(super) fn pane_template_runtime(
    runtime: Option<&EditorUiHostRuntime>,
) -> Option<&EditorUiHostRuntime> {
    match runtime {
        Some(runtime) => Some(runtime),
        None => builtin_host_runtime(),
    }
}

fn host_template_node_with_content_fallback(
    node: RetainedUiHostNodeProjection,
    content_size: PaneContentSize,
) -> Option<host_contract::TemplatePaneNodeData> {
    let control_id = node.control_id.clone();
    let mut node = host_template_node(node)?;
    if matches!(
        control_id.as_deref(),
        Some("HierarchyListPanel")
            | Some("HierarchyTreeSlotAnchor")
            | Some("AnimationEditorBodyPanel")
            | Some("AnimationSequenceContentPanel")
            | Some("AnimationTimelineSlotAnchor")
            | Some("AnimationGraphContentPanel")
            | Some("AnimationGraphCanvasSlotAnchor")
            | Some("ModulePluginListPanel")
            | Some("ModulePluginListSlotAnchor")
            | Some("BuildExportTargetsPanel")
            | Some("BuildExportTargetsSlotAnchor")
    ) && node.frame.width <= 0.0
        && node.frame.height <= 0.0
    {
        node.frame.width = content_size.width.max(0.0);
        node.frame.height = content_size.height.max(0.0);
    }
    Some(node)
}

pub(super) fn builtin_host_runtime() -> Option<&'static EditorUiHostRuntime> {
    static BUILTIN_HOST_RUNTIME: OnceLock<Option<EditorUiHostRuntime>> = OnceLock::new();
    BUILTIN_HOST_RUNTIME
        .get_or_init(|| {
            let mut runtime = EditorUiHostRuntime::default();
            runtime.load_builtin_host_templates().ok()?;
            Some(runtime)
        })
        .as_ref()
}
