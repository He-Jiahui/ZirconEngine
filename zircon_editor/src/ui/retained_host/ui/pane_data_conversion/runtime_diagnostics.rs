use std::collections::BTreeMap;
use std::sync::Arc;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    PaneContentSize, PaneData, PanePayload, RuntimeDiagnosticsPanePayload,
};
use crate::ui::retained_host as host_contract;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::UiSurfaceFrame,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

const REFLECTOR_SECTION_PADDING: f32 = 8.0;
const REFLECTOR_LINE_HEIGHT: f32 = 18.0;
const REFLECTOR_LINE_GAP: f32 = 4.0;

pub(crate) fn to_host_contract_runtime_diagnostics_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::RuntimeDiagnosticsPaneData {
    let template_nodes =
        runtime_diagnostics_template_projection(data, content_size).unwrap_or_default();
    let nodes = runtime_debug_reflector_nodes(data, &template_nodes, content_size);

    host_contract::RuntimeDiagnosticsPaneData {
        nodes: model_rc(nodes),
        overlay_primitives: model_rc(Vec::new()),
        preserve_payload_debug_reflector: runtime_debug_reflector_has_active_payload_snapshot(data),
    }
}

pub(crate) fn refresh_runtime_diagnostics_debug_reflector_from_body_surface(
    pane: &mut host_contract::PaneData,
    content_size: PaneContentSize,
) -> bool {
    if pane.kind.as_str() != "RuntimeDiagnostics" {
        return false;
    }
    if pane.runtime_diagnostics.preserve_payload_debug_reflector {
        return false;
    }
    let surface_frame =
        runtime_diagnostics_debug_surface_frame(&pane.runtime_diagnostics, content_size);
    let snapshot = zircon_runtime::ui::surface::debug_surface_frame(&surface_frame);
    let reflector =
        crate::ui::workbench::debug_reflector::EditorUiDebugReflectorModel::from_snapshot(
            &snapshot,
        )
        .with_schedule_sections(&snapshot);
    let template_nodes = runtime_diagnostics_existing_template_nodes(&pane.runtime_diagnostics);
    let nodes = runtime_debug_reflector_nodes_from_model(&template_nodes, &reflector, content_size);

    pane.runtime_diagnostics.nodes = model_rc(nodes);
    pane.runtime_diagnostics.overlay_primitives = model_rc(Vec::new());
    true
}

fn runtime_diagnostics_debug_surface_frame(
    data: &host_contract::RuntimeDiagnosticsPaneData,
    content_size: PaneContentSize,
) -> Arc<UiSurfaceFrame> {
    let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.runtime_diagnostics.reflector"));
    let root_frame = UiFrame::new(
        0.0,
        0.0,
        content_size.width.max(1.0),
        content_size.height.max(1.0),
    );
    surface.tree.insert_root(
        UiTreeNode::new(
            UiNodeId::new(1),
            UiNodePath::new("runtime_diagnostics/root"),
        )
        .with_frame(root_frame)
        .with_clip_to_bounds(true)
        .with_input_policy(UiInputPolicy::Ignore),
    );

    for (row, node) in data.nodes.iter().enumerate() {
        let mut attributes = BTreeMap::new();
        if !node.text.is_empty() {
            attributes.insert(
                "text".to_string(),
                toml::Value::String(node.text.to_string()),
            );
        }
        let interactive = !node.disabled && !node.control_id.is_empty();
        let tree_node = UiTreeNode::new(
            UiNodeId::new(row as u64 + 2),
            UiNodePath::new(format!("runtime_diagnostics/{}", node.node_id)),
        )
        .with_frame(UiFrame::new(
            node.frame.x,
            node.frame.y,
            node.frame.width,
            node.frame.height,
        ))
        .with_input_policy(if interactive {
            UiInputPolicy::Receive
        } else {
            UiInputPolicy::Ignore
        })
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: !node.disabled,
            clickable: interactive,
            hoverable: interactive,
            focusable: false,
            pressed: false,
            checked: false,
            dirty: false,
        })
        .with_template_metadata(UiTemplateNodeMetadata {
            component: node.role.to_string(),
            control_id: Some(node.control_id.to_string()),
            attributes,
            ..UiTemplateNodeMetadata::default()
        });
        let _ = surface.tree.insert_child(UiNodeId::new(1), tree_node);
    }

    surface.rebuild();
    surface.surface_frame()
}

fn runtime_debug_reflector_has_active_payload_snapshot(data: &PaneData) -> bool {
    data.pane_presentation
        .as_ref()
        .and_then(|presentation| match &presentation.body.payload {
            PanePayload::RuntimeDiagnosticsV1(payload) => {
                Some(payload.ui_debug_reflector_has_active_snapshot)
            }
            _ => None,
        })
        .unwrap_or(false)
}

fn runtime_diagnostics_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
) -> Option<Vec<host_contract::TemplatePaneNodeData>> {
    let presentation = data.pane_presentation.as_ref()?;
    if !matches!(
        &presentation.body.payload,
        PanePayload::RuntimeDiagnosticsV1(_)
    ) {
        return None;
    }

    super::project_pane_template_nodes(&presentation.body, content_size)
}

fn runtime_debug_reflector_nodes(
    data: &PaneData,
    template_nodes: &[host_contract::TemplatePaneNodeData],
    content_size: PaneContentSize,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let Some(payload) = data.pane_presentation.as_ref().and_then(|presentation| {
        if let PanePayload::RuntimeDiagnosticsV1(payload) = &presentation.body.payload {
            Some(payload)
        } else {
            None
        }
    }) else {
        return Vec::new();
    };

    let details = payload
        .ui_debug_reflector_details
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let section_lines = payload
        .ui_debug_reflector_sections
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let node_labels = payload
        .ui_debug_reflector_nodes
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let status_lines = runtime_diagnostics_status_lines(payload);
    runtime_debug_reflector_nodes_from_parts(
        template_nodes,
        Some(payload.summary.as_str()),
        &status_lines,
        payload.ui_debug_reflector_summary.as_str(),
        payload.ui_debug_reflector_export_status.as_str(),
        &details,
        &section_lines,
        &node_labels,
        content_size,
    )
}

fn runtime_debug_reflector_nodes_from_model(
    template_nodes: &[host_contract::TemplatePaneNodeData],
    reflector: &crate::ui::workbench::debug_reflector::EditorUiDebugReflectorModel,
    content_size: PaneContentSize,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let node_labels = reflector
        .nodes
        .iter()
        .map(|node| {
            if node.selected {
                format!("> {}", node.label)
            } else {
                node.label.clone()
            }
        })
        .collect::<Vec<_>>();
    let section_lines = reflector.section_display_lines();
    runtime_debug_reflector_nodes_from_parts(
        template_nodes,
        None,
        &[],
        reflector.summary.title.as_str(),
        reflector.summary.export_status.as_str(),
        &reflector.details,
        &section_lines,
        &node_labels,
        content_size,
    )
}

fn runtime_debug_reflector_nodes_from_parts(
    template_nodes: &[host_contract::TemplatePaneNodeData],
    runtime_summary: Option<&str>,
    runtime_status_lines: &[String],
    reflector_summary: &str,
    export_status: &str,
    details: &[String],
    section_lines: &[String],
    node_labels: &[String],
    content_size: PaneContentSize,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let section = template_nodes
        .iter()
        .find(|node| node.control_id.as_str() == "UiDebugReflectorNodeList")
        .map(|node| node.frame.clone())
        .unwrap_or_else(|| host_contract::TemplateNodeFrameData {
            x: 0.0,
            y: 72.0,
            width: content_size.width.max(0.0),
            height: (content_size.height - 72.0).max(0.0),
        });
    let mut nodes = template_text_nodes_from_parts(template_nodes, runtime_summary);
    let mut y = section.y + REFLECTOR_SECTION_PADDING;
    let x = section.x + REFLECTOR_SECTION_PADDING;
    let width = (section.width - REFLECTOR_SECTION_PADDING * 2.0).max(0.0);

    let existing_status_bottom = nodes
        .iter()
        .filter(|node| {
            node.node_id
                .as_str()
                .starts_with("runtime_diagnostics_status_")
        })
        .map(|node| node.frame.y + node.frame.height)
        .fold(y, f32::max);
    if existing_status_bottom > y {
        y = existing_status_bottom + REFLECTOR_LINE_GAP;
    }

    for (index, status) in runtime_status_lines.iter().enumerate() {
        push_label(
            &mut nodes,
            "runtime_diagnostics_status_",
            format!("{index}"),
            format!("RuntimeDiagnosticsStatus.{index}"),
            status,
            x,
            &mut y,
            width,
            index >= 4,
        );
    }

    push_label(
        &mut nodes,
        "runtime_debug_reflector_",
        "summary",
        "UiDebugReflectorSummaryText",
        reflector_summary,
        x,
        &mut y,
        width,
        false,
    );
    push_label(
        &mut nodes,
        "runtime_debug_reflector_",
        "export",
        "UiDebugReflectorExportStatusText",
        export_status,
        x,
        &mut y,
        width,
        true,
    );

    for (index, detail) in details.iter().enumerate() {
        push_label(
            &mut nodes,
            "runtime_debug_reflector_",
            format!("detail_{index}"),
            format!("UiDebugReflectorDetail.{index}"),
            detail,
            x,
            &mut y,
            width,
            true,
        );
    }

    for (index, section_line) in section_lines.iter().enumerate() {
        push_label(
            &mut nodes,
            "runtime_debug_reflector_",
            format!("section_{index}"),
            format!("UiDebugReflectorSection.{index}"),
            section_line,
            x,
            &mut y,
            width,
            !section_line.ends_with(':'),
        );
    }

    for (index, text) in node_labels.iter().enumerate() {
        push_label(
            &mut nodes,
            "runtime_debug_reflector_",
            format!("node_{index}"),
            format!("UiDebugReflectorNode.{index}"),
            text,
            x,
            &mut y,
            width,
            true,
        );
    }

    nodes
}

fn runtime_diagnostics_existing_template_nodes(
    data: &host_contract::RuntimeDiagnosticsPaneData,
) -> Vec<host_contract::TemplatePaneNodeData> {
    data.nodes
        .iter()
        .filter(|node| {
            !node
                .node_id
                .as_str()
                .starts_with("runtime_debug_reflector_")
        })
        .cloned()
        .collect()
}

fn template_text_nodes_from_parts(
    template_nodes: &[host_contract::TemplatePaneNodeData],
    runtime_summary: Option<&str>,
) -> Vec<host_contract::TemplatePaneNodeData> {
    template_nodes
        .iter()
        .cloned()
        .map(|mut node| {
            match node.control_id.as_str() {
                "RuntimeDiagnosticsSummary" => {
                    if let Some(summary) = runtime_summary {
                        node.text = summary.into();
                    }
                }
                _ => {}
            }
            node
        })
        .collect()
}

fn push_label(
    nodes: &mut Vec<host_contract::TemplatePaneNodeData>,
    node_prefix: &str,
    node_suffix: impl Into<String>,
    control_id: impl Into<String>,
    text: &str,
    x: f32,
    y: &mut f32,
    width: f32,
    muted: bool,
) {
    if text.trim().is_empty() {
        return;
    }

    let mut node = host_contract::TemplatePaneNodeData {
        node_id: format!("{node_prefix}{}", node_suffix.into()).into(),
        control_id: control_id.into().into(),
        role: "Label".into(),
        text: text.to_string().into(),
        frame: host_contract::TemplateNodeFrameData {
            x,
            y: *y,
            width,
            height: REFLECTOR_LINE_HEIGHT,
        },
        ..host_contract::TemplatePaneNodeData::default()
    };
    if muted {
        node.text_tone = "muted".into();
    }
    nodes.push(node);
    *y += REFLECTOR_LINE_HEIGHT + REFLECTOR_LINE_GAP;
}

fn runtime_diagnostics_status_lines(payload: &RuntimeDiagnosticsPanePayload) -> Vec<String> {
    const HYBRID_GI_PRIMARY_PREFIXES: [&str; 3] = [
        "Hybrid GI effective:",
        "Hybrid GI budgets:",
        "Hybrid GI fallback:",
    ];
    const HYBRID_GI_ACTIVE_PROBES_PREFIX: &str = "Hybrid GI active probes:";

    let mut lines = Vec::new();
    for prefix in HYBRID_GI_PRIMARY_PREFIXES {
        lines.extend(
            payload
                .detail_items
                .iter()
                .filter(|item| item.starts_with(prefix))
                .cloned(),
        );
    }
    lines.push(payload.render_status.clone());
    lines.extend(
        payload
            .detail_items
            .iter()
            .filter(|item| item.starts_with(HYBRID_GI_ACTIVE_PROBES_PREFIX))
            .cloned(),
    );
    lines.extend([
        payload.physics_status.clone(),
        payload.animation_status.clone(),
    ]);
    lines.extend(
        payload
            .detail_items
            .iter()
            .filter(|item| {
                !HYBRID_GI_PRIMARY_PREFIXES
                    .iter()
                    .any(|prefix| item.starts_with(prefix))
                    && !item.starts_with(HYBRID_GI_ACTIVE_PROBES_PREFIX)
            })
            .cloned(),
    );
    lines
}

#[cfg(test)]
mod tests {
    use crate::ui::layouts::windows::workbench_host_window::RuntimeDiagnosticsPanePayload;

    use super::runtime_diagnostics_status_lines;

    #[test]
    fn hybrid_gi_priority_lines_keep_render_frame_status_in_the_visible_group() {
        let payload = RuntimeDiagnosticsPanePayload {
            summary: "1 runtime systems available".to_string(),
            render_status: "Render: wgpu(vulkan) (1 viewports, 42 frames)".to_string(),
            physics_status: "Physics: unavailable".to_string(),
            animation_status: "Animation: unavailable".to_string(),
            detail_items: vec![
                "Hybrid GI active probes: 0".to_string(),
                "Hybrid GI fallback: baked-lighting-unavailable".to_string(),
                "Hybrid GI budgets: trace=64, cards=256, voxels=64".to_string(),
                "Hybrid GI effective: profile=indoor-static, mode=dynamic-only, quality=high"
                    .to_string(),
                "Virtual Geometry Debug: unavailable".to_string(),
            ],
            ui_debug_reflector_summary: String::new(),
            ui_debug_reflector_nodes: Vec::new(),
            ui_debug_reflector_details: Vec::new(),
            ui_debug_reflector_sections: Vec::new(),
            ui_debug_reflector_export_status: String::new(),
            ui_debug_reflector_overlay_primitives: Vec::new(),
            ui_debug_reflector_has_active_snapshot: false,
        };

        assert_eq!(
            &runtime_diagnostics_status_lines(&payload)[..5],
            [
                "Hybrid GI effective: profile=indoor-static, mode=dynamic-only, quality=high",
                "Hybrid GI budgets: trace=64, cards=256, voxels=64",
                "Hybrid GI fallback: baked-lighting-unavailable",
                "Render: wgpu(vulkan) (1 viewports, 42 frames)",
                "Hybrid GI active probes: 0",
            ]
        );
    }
}
