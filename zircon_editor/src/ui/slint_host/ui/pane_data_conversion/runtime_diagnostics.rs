use std::collections::BTreeMap;

use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{PaneContentSize, PaneData, PanePayload};
use crate::ui::slint_host as host_contract;
use slint::Model;
use zircon_runtime::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::UiFrame,
    surface::{UiDebugOverlayPrimitive, UiSurfaceFrame},
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
        overlay_primitives: model_rc(
            runtime_debug_reflector_overlay_primitives(data)
                .into_iter()
                .map(|primitive| to_host_contract_ui_debug_overlay_primitive(&primitive))
                .collect(),
        ),
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
        );
    let template_nodes = runtime_diagnostics_existing_template_nodes(&pane.runtime_diagnostics);
    let nodes = runtime_debug_reflector_nodes_from_model(&template_nodes, &reflector, content_size);

    pane.runtime_diagnostics.nodes = model_rc(nodes);
    pane.runtime_diagnostics.overlay_primitives = model_rc(
        snapshot
            .overlay_primitives
            .into_iter()
            .map(|primitive| to_host_contract_ui_debug_overlay_primitive(&primitive))
            .collect(),
    );
    true
}

fn runtime_diagnostics_debug_surface_frame(
    data: &host_contract::RuntimeDiagnosticsPaneData,
    content_size: PaneContentSize,
) -> UiSurfaceFrame {
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

    for row in 0..data.nodes.row_count() {
        let Some(node) = data.nodes.row_data(row) else {
            continue;
        };
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

fn runtime_debug_reflector_overlay_primitives(data: &PaneData) -> Vec<UiDebugOverlayPrimitive> {
    data.pane_presentation
        .as_ref()
        .and_then(|presentation| match &presentation.body.payload {
            PanePayload::RuntimeDiagnosticsV1(payload) => {
                Some(payload.ui_debug_reflector_overlay_primitives.clone())
            }
            _ => None,
        })
        .unwrap_or_default()
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

pub(crate) fn to_host_contract_ui_debug_overlay_primitive(
    primitive: &UiDebugOverlayPrimitive,
) -> host_contract::UiDebugOverlayPrimitiveData {
    host_contract::UiDebugOverlayPrimitiveData {
        kind: primitive.kind,
        node_id: primitive
            .node_id
            .map(|node_id| node_id.0.to_string())
            .unwrap_or_default()
            .into(),
        frame: host_contract::FrameRect {
            x: primitive.frame.x,
            y: primitive.frame.y,
            width: primitive.frame.width,
            height: primitive.frame.height,
        },
        label: primitive.label.clone().unwrap_or_default().into(),
        severity: primitive.severity.clone().unwrap_or_default().into(),
    }
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
    let node_labels = payload
        .ui_debug_reflector_nodes
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    runtime_debug_reflector_nodes_from_parts(
        template_nodes,
        payload.ui_debug_reflector_summary.as_str(),
        payload.ui_debug_reflector_export_status.as_str(),
        &details,
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
    runtime_debug_reflector_nodes_from_parts(
        template_nodes,
        reflector.summary.title.as_str(),
        reflector.summary.export_status.as_str(),
        &reflector.details,
        &node_labels,
        content_size,
    )
}

fn runtime_debug_reflector_nodes_from_parts(
    template_nodes: &[host_contract::TemplatePaneNodeData],
    summary: &str,
    export_status: &str,
    details: &[String],
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
    let mut nodes = template_text_nodes_from_parts(template_nodes, summary, export_status, details);
    let mut y = section.y + REFLECTOR_SECTION_PADDING;
    let x = section.x + REFLECTOR_SECTION_PADDING;
    let width = (section.width - REFLECTOR_SECTION_PADDING * 2.0).max(0.0);

    push_label(
        &mut nodes,
        "summary",
        "UiDebugReflectorSummaryText",
        summary,
        x,
        &mut y,
        width,
        false,
    );
    push_label(
        &mut nodes,
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
            format!("detail_{index}"),
            format!("UiDebugReflectorDetail.{index}"),
            detail,
            x,
            &mut y,
            width,
            true,
        );
    }

    for (index, text) in node_labels.iter().enumerate() {
        push_label(
            &mut nodes,
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
    (0..data.nodes.row_count())
        .filter_map(|row| data.nodes.row_data(row))
        .filter(|node| {
            !node
                .node_id
                .as_str()
                .starts_with("runtime_debug_reflector_")
        })
        .collect()
}

fn template_text_nodes_from_parts(
    template_nodes: &[host_contract::TemplatePaneNodeData],
    summary: &str,
    export_status: &str,
    details: &[String],
) -> Vec<host_contract::TemplatePaneNodeData> {
    template_nodes
        .iter()
        .cloned()
        .map(|mut node| {
            match node.control_id.as_str() {
                "UiDebugReflectorSummary" => {
                    node.text = summary.into();
                }
                "UiDebugReflectorExportStatus" => {
                    node.text = export_status.into();
                    node.text_tone = "muted".into();
                }
                "UiDebugReflectorDetail" => {
                    node.text = details.first().cloned().unwrap_or_default().into();
                    node.text_tone = "muted".into();
                }
                _ => {}
            }
            node
        })
        .collect()
}

fn push_label(
    nodes: &mut Vec<host_contract::TemplatePaneNodeData>,
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
        node_id: format!("runtime_debug_reflector_{}", node_suffix.into()).into(),
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
