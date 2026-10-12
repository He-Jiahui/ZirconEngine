use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::windows::workbench_host_window::{
    PaneContentSize, PaneData, PanePayload, RuntimeDiagnosticsPanePayload,
};
use crate::ui::retained_host as host_contract;
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
    let Some(surface_frame) = pane.body_surface_frame.as_ref() else {
        return false;
    };
    let snapshot = zircon_runtime::ui::surface::debug_surface_frame(surface_frame);
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

    let status_lines = runtime_diagnostics_status_lines(payload);
    runtime_debug_reflector_nodes_from_parts(
        template_nodes,
        Some(payload.summary.as_str()),
        &status_lines,
        payload.ui_debug_reflector_summary.as_str(),
        payload.ui_debug_reflector_export_status.as_str(),
        &payload.ui_debug_reflector_details,
        &payload.ui_debug_reflector_sections,
        &payload.ui_debug_reflector_nodes,
        content_size,
    )
}

fn runtime_debug_reflector_nodes_from_model(
    template_nodes: &[host_contract::TemplatePaneNodeData],
    reflector: &crate::ui::workbench::debug_reflector::EditorUiDebugReflectorModel,
    content_size: PaneContentSize,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let mut writer = RuntimeDebugReflectorNodeWriter::new(template_nodes, None, content_size);
    writer.push(
        "summary",
        "UiDebugReflectorSummaryText",
        reflector.summary.title.as_str(),
        false,
    );
    writer.push(
        "export",
        "UiDebugReflectorExportStatusText",
        reflector.summary.export_status.as_str(),
        true,
    );
    for (index, detail) in reflector.details.iter().enumerate() {
        writer.push(
            format!("detail_{index}"),
            format!("UiDebugReflectorDetail.{index}"),
            detail.as_str(),
            true,
        );
    }

    let mut section_index = 0;
    for section in &reflector.sections {
        if section.title.trim().is_empty() {
            continue;
        }
        writer.push(
            format!("section_{section_index}"),
            format!("UiDebugReflectorSection.{section_index}"),
            format!("{}:", section.title),
            false,
        );
        section_index += 1;
        for line in &section.lines {
            writer.push(
                format!("section_{section_index}"),
                format!("UiDebugReflectorSection.{section_index}"),
                format!("  {line}"),
                true,
            );
            section_index += 1;
        }
    }

    for (index, node) in reflector.nodes.iter().enumerate() {
        if node.selected {
            writer.push(
                format!("node_{index}"),
                format!("UiDebugReflectorNode.{index}"),
                format!("> {}", node.label),
                true,
            );
        } else {
            writer.push(
                format!("node_{index}"),
                format!("UiDebugReflectorNode.{index}"),
                node.label.as_str(),
                true,
            );
        }
    }
    writer.finish()
}

fn runtime_debug_reflector_nodes_from_parts(
    template_nodes: &[host_contract::TemplatePaneNodeData],
    runtime_summary: Option<&str>,
    runtime_status_lines: &[&str],
    reflector_summary: &str,
    export_status: &str,
    details: &[String],
    section_lines: &[String],
    node_labels: &[String],
    content_size: PaneContentSize,
) -> Vec<host_contract::TemplatePaneNodeData> {
    let mut writer =
        RuntimeDebugReflectorNodeWriter::new(template_nodes, runtime_summary, content_size);

    for (index, status) in runtime_status_lines.iter().enumerate() {
        writer.push_with_prefix(
            "runtime_diagnostics_status_",
            format!("{index}"),
            format!("RuntimeDiagnosticsStatus.{index}"),
            *status,
            index >= 4,
        );
    }

    writer.push(
        "summary",
        "UiDebugReflectorSummaryText",
        reflector_summary,
        false,
    );
    writer.push(
        "export",
        "UiDebugReflectorExportStatusText",
        export_status,
        true,
    );

    for (index, detail) in details.iter().enumerate() {
        writer.push(
            format!("detail_{index}"),
            format!("UiDebugReflectorDetail.{index}"),
            detail.as_str(),
            true,
        );
    }

    for (index, section_line) in section_lines.iter().enumerate() {
        writer.push(
            format!("section_{index}"),
            format!("UiDebugReflectorSection.{index}"),
            section_line.as_str(),
            !section_line.ends_with(':'),
        );
    }

    for (index, text) in node_labels.iter().enumerate() {
        writer.push(
            format!("node_{index}"),
            format!("UiDebugReflectorNode.{index}"),
            text.as_str(),
            true,
        );
    }

    writer.finish()
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

struct RuntimeDebugReflectorNodeWriter {
    nodes: Vec<host_contract::TemplatePaneNodeData>,
    x: f32,
    y: f32,
    width: f32,
}

impl RuntimeDebugReflectorNodeWriter {
    fn new(
        template_nodes: &[host_contract::TemplatePaneNodeData],
        runtime_summary: Option<&str>,
        content_size: PaneContentSize,
    ) -> Self {
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
        let nodes = template_text_nodes_from_parts(template_nodes, runtime_summary);
        let mut y = section.y + REFLECTOR_SECTION_PADDING;
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
        Self {
            nodes,
            x: section.x + REFLECTOR_SECTION_PADDING,
            y,
            width: (section.width - REFLECTOR_SECTION_PADDING * 2.0).max(0.0),
        }
    }

    fn push(
        &mut self,
        node_suffix: impl Into<String>,
        control_id: impl Into<String>,
        text: impl Into<String>,
        muted: bool,
    ) {
        self.push_with_prefix(
            "runtime_debug_reflector_",
            node_suffix,
            control_id,
            text,
            muted,
        );
    }

    fn push_with_prefix(
        &mut self,
        node_prefix: &str,
        node_suffix: impl Into<String>,
        control_id: impl Into<String>,
        text: impl Into<String>,
        muted: bool,
    ) {
        let text = text.into();
        if text.trim().is_empty() {
            return;
        }

        let mut node = host_contract::TemplatePaneNodeData {
            node_id: format!("{node_prefix}{}", node_suffix.into()).into(),
            control_id: control_id.into().into(),
            role: "Label".into(),
            text: text.into(),
            frame: host_contract::TemplateNodeFrameData {
                x: self.x,
                y: self.y,
                width: self.width,
                height: REFLECTOR_LINE_HEIGHT,
            },
            ..host_contract::TemplatePaneNodeData::default()
        };
        if muted {
            node.text_tone = "muted".into();
        }
        self.nodes.push(node);
        self.y += REFLECTOR_LINE_HEIGHT + REFLECTOR_LINE_GAP;
    }

    fn finish(self) -> Vec<host_contract::TemplatePaneNodeData> {
        self.nodes
    }
}

fn runtime_diagnostics_status_lines(payload: &RuntimeDiagnosticsPanePayload) -> Vec<&str> {
    const HYBRID_GI_PRIMARY_PREFIXES: [&str; 3] = [
        "Hybrid GI effective:",
        "Hybrid GI budgets:",
        "Hybrid GI fallback:",
    ];
    const HYBRID_GI_ACTIVE_PROBES_PREFIX: &str = "Hybrid GI active probes:";

    let mut lines = Vec::with_capacity(payload.detail_items.len() + 3);
    for prefix in HYBRID_GI_PRIMARY_PREFIXES {
        lines.extend(
            payload
                .detail_items
                .iter()
                .filter_map(|item| item.starts_with(prefix).then_some(item.as_str())),
        );
    }
    lines.push(payload.render_status.as_str());
    lines.extend(payload.detail_items.iter().filter_map(|item| {
        item.starts_with(HYBRID_GI_ACTIVE_PROBES_PREFIX)
            .then_some(item.as_str())
    }));
    lines.extend([
        payload.physics_status.as_str(),
        payload.animation_status.as_str(),
    ]);
    lines.extend(payload.detail_items.iter().filter_map(|item| {
        let is_primary = HYBRID_GI_PRIMARY_PREFIXES
            .iter()
            .any(|prefix| item.starts_with(prefix));
        (!is_primary && !item.starts_with(HYBRID_GI_ACTIVE_PROBES_PREFIX)).then_some(item.as_str())
    }));
    lines
}

#[cfg(test)]
#[path = "tests/runtime_diagnostics.rs"]
mod tests;
