use crate::core::editor_event::{ConsoleMessageFilter, ConsoleSourceFilter};
use crate::ui::host::editor_activity_log::activity_log_jump_action_id;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::console_pane_nodes;
use crate::ui::layouts::windows::workbench_host_window::{
    ConsolePaneViewData, PaneContentSize, PaneData, PanePayload,
};
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::console_output::{
    console_output_line_count, console_output_lines_with_presence, ConsoleOutputPaintMetadata,
    ConsoleOutputViewport, CONSOLE_OUTPUT_BODY_CONTROL_ID, CONSOLE_OUTPUT_LINE_HEIGHT,
    CONSOLE_OUTPUT_LINE_PREFIX, CONSOLE_OUTPUT_MIN_MESSAGE_SLOT_WIDTH,
    CONSOLE_OUTPUT_PROTOTYPE_CONTROL_ID, CONSOLE_OUTPUT_SEVERITY_PREFIX,
    CONSOLE_OUTPUT_SEVERITY_SLOT_WIDTH,
};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::template_runtime::EditorUiHostRuntime;
use crate::ui::workbench::snapshot::{ConsoleOutputLevelCounts, EditorConsoleMessageLevel};
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::template_node_conversion::to_host_contract_template_node;
use super::pane_component_projection::host_template_node;
use super::pane_template_runtime;
use super::template_node_projection::project_nodes;

fn to_host_contract_console_pane(
    data: &ConsolePaneViewData,
    content_size: PaneContentSize,
) -> host_contract::ConsolePaneData {
    let nodes = if data.nodes.row_count() == 0 {
        console_pane_nodes(
            data.status_text.as_ref(),
            UiSize::new(content_size.width.max(0.0), content_size.height.max(0.0)),
        )
    } else {
        data.nodes.clone()
    };
    host_contract::ConsolePaneData {
        nodes: project_nodes(&nodes, to_host_contract_console_legacy_node),
        status_text: std::sync::Arc::clone(&data.status_text),
    }
}

pub(crate) fn to_host_contract_console_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::ConsolePaneData {
    console_template_projection(data, content_size, None)
        .unwrap_or_else(|| to_host_contract_console_pane(&data.native_body.console, content_size))
}

pub(crate) fn to_host_contract_console_pane_from_host_pane_with_runtime(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> host_contract::ConsolePaneData {
    console_template_projection(data, content_size, Some(runtime))
        .unwrap_or_else(|| to_host_contract_console_pane(&data.native_body.console, content_size))
}

fn console_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
) -> Option<host_contract::ConsolePaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    let PanePayload::ConsoleV1(console_payload) = &presentation.body.payload else {
        return None;
    };

    let runtime = pane_template_runtime(runtime)?;
    let projection = runtime.project_pane_body(&presentation.body).ok()?;
    let mut surface = runtime
        .build_shared_surface(&presentation.body.document_id)
        .ok()?;
    surface
        .compute_layout(UiSize::new(
            content_size.width.max(0.0),
            content_size.height.max(0.0),
        ))
        .ok()?;
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .ok()?;
    let status_text = console_payload.status_text.as_ref();

    let mut nodes = host_model
        .nodes
        .into_iter()
        .filter_map(host_template_node)
        .collect::<Vec<_>>();
    project_console_level_counts(&mut nodes, console_payload.counts);
    project_console_level_filter(&mut nodes, console_payload.filter);
    project_console_source_filter(&mut nodes, console_payload.source_filter);
    let output_metadata = project_console_output_lines(
        &mut nodes,
        status_text,
        console_payload.levels.as_ref(),
        console_payload.jump_sequences.as_ref(),
    );

    let nodes = match output_metadata {
        Some(metadata) => ModelRc::with_metadata(nodes, metadata),
        None => model_rc(nodes),
    };
    Some(host_contract::ConsolePaneData {
        nodes,
        status_text: std::sync::Arc::clone(&console_payload.status_text),
    })
}

fn project_console_source_filter(
    nodes: &mut [host_contract::TemplatePaneNodeData],
    filter: ConsoleSourceFilter,
) {
    for (control_id, node_filter) in [
        ("ConsoleSourceAll", ConsoleSourceFilter::All),
        ("ConsoleSourceEditor", ConsoleSourceFilter::Editor),
        ("ConsoleSourceRuntime", ConsoleSourceFilter::Runtime),
        ("ConsoleSourcePlay", ConsoleSourceFilter::Play),
        ("ConsoleSourcePlugin", ConsoleSourceFilter::Plugin),
        ("ConsoleSourceImport", ConsoleSourceFilter::Import),
        ("ConsoleSourceScriptBuild", ConsoleSourceFilter::ScriptBuild),
    ] {
        let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) else {
            continue;
        };
        node.selected = filter == node_filter;
        node.checked = node.selected;
    }
}

fn project_console_level_filter(
    nodes: &mut [host_contract::TemplatePaneNodeData],
    filter: ConsoleMessageFilter,
) {
    for (control_id, node_filter) in [
        ("ConsoleLevelAll", ConsoleMessageFilter::All),
        ("ConsoleLevelError", ConsoleMessageFilter::Error),
        ("ConsoleLevelWarning", ConsoleMessageFilter::Warning),
        ("ConsoleLevelInfo", ConsoleMessageFilter::Info),
    ] {
        let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) else {
            continue;
        };
        node.selected = filter == node_filter;
        node.checked = node.selected;
    }
}

fn project_console_level_counts(
    nodes: &mut [host_contract::TemplatePaneNodeData],
    counts: ConsoleOutputLevelCounts,
) {
    for (control_id, count) in [
        ("ConsoleLevelError", counts.error),
        ("ConsoleLevelWarning", counts.warning),
        ("ConsoleLevelInfo", counts.info),
    ] {
        let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) else {
            continue;
        };
        node.text = count.to_string().into();
        node.value_number = count as f32;
    }
}

fn project_console_output_lines(
    nodes: &mut Vec<host_contract::TemplatePaneNodeData>,
    status_text: &str,
    levels: &[EditorConsoleMessageLevel],
    jump_sequences: &[Option<u64>],
) -> Option<ConsoleOutputPaintMetadata> {
    let viewport = nodes
        .iter()
        .find(|node| node.control_id == CONSOLE_OUTPUT_BODY_CONTROL_ID)
        .map(|node| ConsoleOutputViewport {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        });
    let Some(prototype_index) = nodes
        .iter()
        .position(|node| node.control_id == CONSOLE_OUTPUT_PROTOTYPE_CONTROL_ID)
    else {
        return None;
    };
    let prototype = nodes.remove(prototype_index);
    let line_origin_y = prototype.frame.y;
    let line_count = console_output_line_count(status_text);
    let projects_severity = !levels.is_empty();
    let nodes_per_line = if projects_severity { 2 } else { 1 };
    let output_lines = console_output_lines_with_presence(status_text, projects_severity)
        .enumerate()
        .flat_map(|(line_index, text)| {
            let mut line = prototype.clone();
            let stable_id = format!("{CONSOLE_OUTPUT_LINE_PREFIX}{line_index:04}");
            line.node_id = stable_id.clone().into();
            line.control_id = stable_id.into();
            line.text = text.into();
            line.text_tone = if projects_severity {
                "secondary"
            } else {
                "muted"
            }
            .into();
            line.frame.y = line_origin_y + line_index as f32 * CONSOLE_OUTPUT_LINE_HEIGHT;
            line.frame.height = CONSOLE_OUTPUT_LINE_HEIGHT;
            if let Some(sequence) = jump_sequences.get(line_index).copied().flatten() {
                line.dispatch_kind = "activity_log_jump".into();
                line.action_id = activity_log_jump_action_id(sequence).into();
                line.text_tone = "accent".into();
            }
            let severity = projects_severity.then(|| {
                let level = levels.get(line_index).copied().unwrap_or_default();
                let mut severity = prototype.clone();
                let stable_id = format!("{CONSOLE_OUTPUT_SEVERITY_PREFIX}{line_index:04}");
                severity.node_id = stable_id.clone().into();
                severity.control_id = stable_id.into();
                severity.text = console_output_level_label(level).into();
                severity.text_tone = console_output_text_tone(level).into();
                severity.frame.y = line.frame.y;
                severity.frame.width = CONSOLE_OUTPUT_SEVERITY_SLOT_WIDTH
                    .min((line.frame.width - CONSOLE_OUTPUT_MIN_MESSAGE_SLOT_WIDTH).max(0.0));
                severity.frame.height = CONSOLE_OUTPUT_LINE_HEIGHT;
                line.frame.x += severity.frame.width;
                line.frame.width = (line.frame.width - severity.frame.width).max(0.0);
                severity
            });
            [severity, Some(line)].into_iter().flatten()
        });
    nodes.splice(prototype_index..prototype_index, output_lines);
    viewport.and_then(|viewport| {
        ConsoleOutputPaintMetadata::new_with_nodes_per_line(
            viewport,
            line_origin_y,
            prototype_index,
            line_count,
            nodes_per_line,
        )
    })
}

fn console_output_text_tone(level: EditorConsoleMessageLevel) -> &'static str {
    match level {
        EditorConsoleMessageLevel::Info => "secondary",
        EditorConsoleMessageLevel::Warning => "warning",
        EditorConsoleMessageLevel::Error => "error",
    }
}

fn console_output_level_label(level: EditorConsoleMessageLevel) -> &'static str {
    match level {
        EditorConsoleMessageLevel::Info => "[Info]",
        EditorConsoleMessageLevel::Warning => "[Warning]",
        EditorConsoleMessageLevel::Error => "[Error]",
    }
}

fn to_host_contract_console_legacy_node(
    data: &crate::ui::layouts::views::ViewTemplateNodeData,
) -> host_contract::TemplatePaneNodeData {
    let mut node = to_host_contract_template_node(data);
    if node.control_id == "ConsoleTextPanel" {
        node.control_id = "ConsoleBodySection".into();
    }
    node
}
