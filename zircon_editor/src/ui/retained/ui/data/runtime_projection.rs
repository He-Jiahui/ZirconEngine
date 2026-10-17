//! 模板 Pane 先投影文档、计算布局并绑定动作，再发布宿主节点；任一步失败都清除旧动作以免继续触发过期绑定。
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

pub(super) fn project_pane_template_nodes_with_runtime_for_pane(
    body: &PaneBodyPresentation,
    content_size: PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
    pane_id: Option<&str>,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    let runtime = pane_template_runtime(runtime)?;
    let nodes = (|| {
        let mut projection = runtime.project_pane_body(body).ok()?;
        let mut surface = runtime.build_shared_surface(&body.document_id).ok()?;
        runtime
            .apply_pane_component_patches_to_surface(body, &mut surface)
            .ok()?;
        let detail_layout = module_plugin_detail_layout_attributes(body, content_size.width);
        runtime
            .apply_module_plugin_detail_layout(
                &body.document_id,
                &mut projection.root,
                &mut surface,
                detail_layout,
            )
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
    let mut node = host_template_node(node)?;
    if matches!(
        node.control_id.as_str(),
        "HierarchyListPanel"
            | "HierarchyTreeSlotAnchor"
            | "AnimationEditorBodyPanel"
            | "AnimationSequenceContentPanel"
            | "AnimationTimelineSlotAnchor"
            | "AnimationGraphContentPanel"
            | "AnimationGraphCanvasSlotAnchor"
            | "ModulePluginListPanel"
            | "ModulePluginListSlotAnchor"
            | "BuildExportTargetsPanel"
            | "BuildExportTargetsSlotAnchor"
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

impl EditorUiHostRuntime {
    /// The built-in retained projector and its callbacks share the same registry.
    pub(crate) fn retained_builtin_pane_runtime() -> Option<&'static Self> {
        builtin_host_runtime()
    }
}

fn module_plugin_detail_layout_attributes(
    body: &PaneBodyPresentation,
    width: f32,
) -> std::collections::BTreeMap<String, std::collections::BTreeMap<String, toml::Value>> {
    use crate::ui::retained_host::host_contract::{
        measure_runtime_text_width, paint_theme::current_host_metrics,
    };
    use std::collections::BTreeMap;
    use toml::Value;
    let PanePayload::ModulePluginsV1(payload) = &body.payload else {
        return BTreeMap::new();
    };
    let Some(detail) = payload.details.as_ref() else {
        return BTreeMap::new();
    };
    let metrics = current_host_metrics();
    let line_height = metrics.line_height(metrics.font_body);
    let available = (width - metrics.gap_m * 2.0).max(metrics.font_body * 2.0);
    let fixed_height = |height: f32| {
        Value::Table(toml::Table::from_iter([
            (
                "width".into(),
                Value::Table(toml::Table::from_iter([(
                    "stretch".into(),
                    Value::String("Stretch".into()),
                )])),
            ),
            (
                "height".into(),
                Value::Table(toml::Table::from_iter([
                    ("min".into(), Value::Float(height as f64)),
                    ("preferred".into(), Value::Float(height as f64)),
                    ("max".into(), Value::Float(height as f64)),
                    ("stretch".into(), Value::String("Fixed".into())),
                ])),
            ),
        ]))
    };
    let mut patches = BTreeMap::new();
    for (control, source) in [
        ("ModulePluginDetailsTitle", &detail.title),
        ("ModulePluginFeatureName", &detail.selected_name),
        ("ModulePluginFeatureIdentity", &detail.selected_identity),
        ("ModulePluginFeatureStates", &detail.selected_states),
        ("ModulePluginFeatureReason", &detail.selected_reason),
        ("ModulePluginFeatureEmpty", &detail.empty_label),
    ] {
        // Break long qualified IDs as well as prose. Full source values remain in
        // the payload; only presentation gains line breaks, using the paint font.
        let mut lines = Vec::new();
        for paragraph in source.split('\n') {
            let mut line = String::new();
            for ch in paragraph.chars() {
                let mut candidate = line.clone();
                candidate.push(ch);
                if !line.is_empty()
                    && measure_runtime_text_width(&candidate, metrics.font_body) > available
                {
                    lines.push(std::mem::take(&mut line));
                }
                line.push(ch);
            }
            lines.push(line);
        }
        let height = (lines.len() as f32 * line_height + metrics.gap_s * 2.0).max(line_height);
        patches.insert(
            control.into(),
            BTreeMap::from([
                ("text".into(), Value::String(lines.join("\n"))),
                ("layout".into(), fixed_height(height)),
            ]),
        );
    }
    let row_height = line_height * 5.0 + metrics.gap_s * 2.0;
    patches.insert(
        "ModulePluginFeatureRows".into(),
        BTreeMap::from([
            (
                "layout".into(),
                fixed_height(detail.rows.len().max(1) as f32 * row_height),
            ),
            (
                "visibility".into(),
                Value::String(
                    if detail.rows.is_empty() {
                        "Collapsed"
                    } else {
                        "Visible"
                    }
                    .into(),
                ),
            ),
        ]),
    );
    patches
}
