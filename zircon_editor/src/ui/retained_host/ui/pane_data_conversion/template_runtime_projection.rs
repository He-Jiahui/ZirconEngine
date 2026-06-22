use std::sync::OnceLock;

use crate::ui::layouts::windows::workbench_host_window::{PaneBodyPresentation, PaneContentSize};
use crate::ui::retained_host as host_contract;
use crate::ui::template_runtime::{EditorUiHostRuntime, RetainedUiHostNodeProjection};
use zircon_runtime_interface::ui::layout::UiSize;

use super::pane_component_projection::host_template_node;

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
    let runtime = pane_template_runtime(runtime)?;
    let projection = runtime.project_pane_body(body).ok()?;
    let mut surface = runtime.build_shared_surface(&body.document_id).ok()?;
    surface
        .compute_layout(UiSize::new(
            content_size.width.max(0.0),
            content_size.height.max(0.0),
        ))
        .ok()?;
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .ok()?;

    Some(
        host_model
            .nodes
            .into_iter()
            .filter_map(|node| host_template_node_with_content_fallback(node, content_size))
            .collect(),
    )
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
