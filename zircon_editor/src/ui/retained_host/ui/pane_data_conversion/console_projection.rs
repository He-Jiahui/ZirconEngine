use std::collections::BTreeMap;

use crate::core::editor_event::{ConsoleMessageFilter, ConsoleSourceFilter};
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::console_pane_nodes;
use crate::ui::layouts::windows::workbench_host_window::{
    ConsolePaneViewData, PaneContentSize, PaneData, PanePayload,
};
use crate::ui::retained_host as host_contract;
use crate::ui::retained_host::console_output::{
    ConsoleOutputPaintMetadata, ConsoleOutputViewport, CONSOLE_OUTPUT_BODY_CONTROL_ID,
    CONSOLE_OUTPUT_LINE_HEIGHT, CONSOLE_OUTPUT_LINE_PREFIX, CONSOLE_OUTPUT_MIN_MESSAGE_SLOT_WIDTH,
    CONSOLE_OUTPUT_OVERSCAN_LINES, CONSOLE_OUTPUT_PROTOTYPE_CONTROL_ID,
    CONSOLE_OUTPUT_SEVERITY_PREFIX, CONSOLE_OUTPUT_SEVERITY_SLOT_WIDTH,
};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter_batch, UiPerfCounter};
use crate::ui::template_runtime::EditorUiHostRuntime;
use crate::ui::workbench::snapshot::{ConsoleOutputLevelCounts, ConsoleOutputSnapshot};
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::template_node_conversion::to_host_contract_template_node;
use super::pane_component_projection::host_template_node;
use super::pane_template_runtime;
use super::template_node_projection::project_nodes;

mod cache;

pub(crate) use cache::ConsolePaneProjectionCache;
use cache::ConsolePaneProjectionCacheKey;

fn to_host_contract_console_pane(
    data: &ConsolePaneViewData,
    content_size: PaneContentSize,
) -> host_contract::ConsolePaneData {
    let nodes = if data.nodes.row_count() == 0 {
        console_pane_nodes(
            data.output.as_ref(),
            UiSize::new(content_size.width.max(0.0), content_size.height.max(0.0)),
        )
    } else {
        data.nodes.clone()
    };
    host_contract::ConsolePaneData {
        nodes: project_nodes(&nodes, to_host_contract_console_legacy_node),
        output: data.output.clone(),
    }
}

pub(crate) fn to_host_contract_console_pane_from_host_pane(
    data: &PaneData,
    content_size: PaneContentSize,
) -> host_contract::ConsolePaneData {
    console_template_projection(data, content_size, None, None)
        .unwrap_or_else(|| to_host_contract_console_pane(&data.native_body.console, content_size))
}

pub(crate) fn to_host_contract_console_pane_from_host_pane_with_runtime(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
) -> host_contract::ConsolePaneData {
    console_template_projection(data, content_size, Some(runtime), None)
        .unwrap_or_else(|| to_host_contract_console_pane(&data.native_body.console, content_size))
}

pub(crate) fn to_host_contract_console_pane_from_host_pane_with_runtime_and_cache(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: &EditorUiHostRuntime,
    cache: &mut ConsolePaneProjectionCache,
) -> host_contract::ConsolePaneData {
    console_template_projection(data, content_size, Some(runtime), Some(cache))
        .unwrap_or_else(|| to_host_contract_console_pane(&data.native_body.console, content_size))
}

fn console_template_projection(
    data: &PaneData,
    content_size: PaneContentSize,
    runtime: Option<&EditorUiHostRuntime>,
    mut cache: Option<&mut ConsolePaneProjectionCache>,
) -> Option<host_contract::ConsolePaneData> {
    let presentation = data.pane_presentation.as_ref()?;
    let PanePayload::ConsoleV1(console_payload) = &presentation.body.payload else {
        return None;
    };

    let runtime = pane_template_runtime(runtime)?;
    let cache_key = runtime
        .retained_document_identity(&presentation.body.document_id)
        .map(|document_identity| ConsolePaneProjectionCacheKey {
            document_identity,
            width_bits: content_size.width.to_bits(),
            height_bits: content_size.height.to_bits(),
        });
    if let (Some(cache), Some(cache_key)) = (cache.as_deref_mut(), cache_key) {
        if let Some(entry) = cache.get(data.id.as_str(), cache_key) {
            if let Some(pane) = reuse_console_projection(&entry.pane, &console_payload.output) {
                cache.publish(data.id.to_string(), cache_key, pane.clone());
                return Some(pane);
            }
        }
    }
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
    let mut nodes = host_model
        .nodes
        .into_iter()
        .filter_map(host_template_node)
        .collect::<Vec<_>>();
    project_console_controls(
        &mut nodes,
        console_payload.output.counts(),
        console_payload.output.filter(),
        console_payload.output.source_filter(),
    );
    let output_metadata = project_console_output_lines(&mut nodes, &console_payload.output);

    let nodes = match output_metadata {
        Some(metadata) => ModelRc::with_metadata(nodes, metadata),
        None => model_rc(nodes),
    };
    let pane = host_contract::ConsolePaneData {
        nodes,
        output: console_payload.output.clone(),
    };
    if let (Some(cache), Some(cache_key)) = (cache, cache_key) {
        cache.publish(data.id.to_string(), cache_key, pane.clone());
    }
    Some(pane)
}

fn project_console_controls(
    nodes: &mut [host_contract::TemplatePaneNodeData],
    counts: ConsoleOutputLevelCounts,
    level_filter: ConsoleMessageFilter,
    source_filter: ConsoleSourceFilter,
) {
    for node in nodes {
        match node.control_id.as_str() {
            "ConsoleSourceAll" => {
                project_console_filter_state(node, source_filter == ConsoleSourceFilter::All)
            }
            "ConsoleSourceEditor" => {
                project_console_filter_state(node, source_filter == ConsoleSourceFilter::Editor)
            }
            "ConsoleSourceRuntime" => {
                project_console_filter_state(node, source_filter == ConsoleSourceFilter::Runtime)
            }
            "ConsoleSourcePlay" => {
                project_console_filter_state(node, source_filter == ConsoleSourceFilter::Play)
            }
            "ConsoleSourcePlugin" => {
                project_console_filter_state(node, source_filter == ConsoleSourceFilter::Plugin)
            }
            "ConsoleSourceImport" => {
                project_console_filter_state(node, source_filter == ConsoleSourceFilter::Import)
            }
            "ConsoleSourceScriptBuild" => project_console_filter_state(
                node,
                source_filter == ConsoleSourceFilter::ScriptBuild,
            ),
            "ConsoleLevelAll" => {
                project_console_filter_state(node, level_filter == ConsoleMessageFilter::All)
            }
            "ConsoleLevelError" => {
                project_console_level_control(
                    node,
                    counts.error,
                    level_filter == ConsoleMessageFilter::Error,
                );
            }
            "ConsoleLevelWarning" => {
                project_console_level_control(
                    node,
                    counts.warning,
                    level_filter == ConsoleMessageFilter::Warning,
                );
            }
            "ConsoleLevelInfo" => {
                project_console_level_control(
                    node,
                    counts.info,
                    level_filter == ConsoleMessageFilter::Info,
                );
            }
            _ => {}
        }
    }
}

fn project_console_level_control(
    node: &mut host_contract::TemplatePaneNodeData,
    count: usize,
    selected: bool,
) {
    node.text = count.to_string().into();
    node.value_number = count as f32;
    project_console_filter_state(node, selected);
}

fn project_console_filter_state(node: &mut host_contract::TemplatePaneNodeData, selected: bool) {
    node.selected = selected;
    node.checked = selected;
}

fn reuse_console_projection(
    cached: &host_contract::ConsolePaneData,
    output: &ConsoleOutputSnapshot,
) -> Option<host_contract::ConsolePaneData> {
    if cached.output.shares_logical_generation_with(output)
        && cached.output.counts() == output.counts()
        && cached.output.filter() == output.filter()
        && cached.output.source_filter() == output.source_filter()
    {
        record_current_ui_perf_counter_batch(|counters| {
            counters.extend_from_slice(&[
                (UiPerfCounter::ConsoleProjectionClonedNodeCount, 0.0),
                (UiPerfCounter::ConsoleProjectionFormattedIdCount, 0.0),
                (UiPerfCounter::ConsoleEnteredLineCount, 0.0),
                (UiPerfCounter::ConsoleExpiredLineCount, 0.0),
                (UiPerfCounter::ConsoleSlotReboundCount, 0.0),
                (UiPerfCounter::ConsoleProjectionGenerationReuseCount, 1.0),
            ]);
        });
        return Some(host_contract::ConsolePaneData {
            nodes: cached.nodes.clone(),
            output: output.clone(),
        });
    }

    let previous_metadata = cached
        .nodes
        .metadata::<ConsoleOutputPaintMetadata>()?
        .clone();
    let next_metadata = previous_metadata.replacing_snapshot(output.clone())?;
    let mut row_patches = console_control_row_patches(&cached.nodes, output);
    let delta = output.line_delta();
    let full_source_replacement = delta.retained == 0 && (delta.entered > 0 || delta.expired > 0);
    let mut rebound_slot_count = 0usize;
    for slot_index in 0..next_metadata.materialized_line_count() {
        if !full_source_replacement
            && previous_metadata.slot_source_id(slot_index, 0.0)
                == next_metadata.slot_source_id(slot_index, 0.0)
        {
            continue;
        }
        patch_console_slot_rows(&cached.nodes, &next_metadata, slot_index, &mut row_patches)?;
        rebound_slot_count += 1;
    }
    let cloned_node_count = row_patches.len();
    let nodes = cached
        .nodes
        .with_row_patches(row_patches)
        .replacing_metadata(next_metadata);
    record_current_ui_perf_counter_batch(|counters| {
        counters.extend_from_slice(&[
            (
                UiPerfCounter::ConsoleProjectionClonedNodeCount,
                cloned_node_count as f64,
            ),
            (UiPerfCounter::ConsoleProjectionFormattedIdCount, 0.0),
            (UiPerfCounter::ConsoleEnteredLineCount, delta.entered as f64),
            (UiPerfCounter::ConsoleExpiredLineCount, delta.expired as f64),
            (
                UiPerfCounter::ConsoleSlotReboundCount,
                rebound_slot_count as f64,
            ),
            (UiPerfCounter::ConsoleProjectionGenerationReuseCount, 1.0),
        ]);
    });
    Some(host_contract::ConsolePaneData {
        nodes,
        output: output.clone(),
    })
}

fn console_control_row_patches(
    nodes: &ModelRc<host_contract::TemplatePaneNodeData>,
    output: &ConsoleOutputSnapshot,
) -> BTreeMap<usize, host_contract::TemplatePaneNodeData> {
    let mut patches = BTreeMap::new();
    for (row, node) in nodes.iter().enumerate() {
        if !console_control_requires_patch(node, output) {
            continue;
        }
        let mut replacement = node.clone();
        project_console_controls(
            std::slice::from_mut(&mut replacement),
            output.counts(),
            output.filter(),
            output.source_filter(),
        );
        patches.insert(row, replacement);
    }
    patches
}

fn console_control_requires_patch(
    node: &host_contract::TemplatePaneNodeData,
    output: &ConsoleOutputSnapshot,
) -> bool {
    let selected = match node.control_id.as_str() {
        "ConsoleSourceAll" => Some(output.source_filter() == ConsoleSourceFilter::All),
        "ConsoleSourceEditor" => Some(output.source_filter() == ConsoleSourceFilter::Editor),
        "ConsoleSourceRuntime" => Some(output.source_filter() == ConsoleSourceFilter::Runtime),
        "ConsoleSourcePlay" => Some(output.source_filter() == ConsoleSourceFilter::Play),
        "ConsoleSourcePlugin" => Some(output.source_filter() == ConsoleSourceFilter::Plugin),
        "ConsoleSourceImport" => Some(output.source_filter() == ConsoleSourceFilter::Import),
        "ConsoleSourceScriptBuild" => {
            Some(output.source_filter() == ConsoleSourceFilter::ScriptBuild)
        }
        "ConsoleLevelAll" => Some(output.filter() == ConsoleMessageFilter::All),
        "ConsoleLevelError" => {
            return node.value_number != output.counts().error as f32
                || node.selected != (output.filter() == ConsoleMessageFilter::Error)
                || node.checked != (output.filter() == ConsoleMessageFilter::Error);
        }
        "ConsoleLevelWarning" => {
            return node.value_number != output.counts().warning as f32
                || node.selected != (output.filter() == ConsoleMessageFilter::Warning)
                || node.checked != (output.filter() == ConsoleMessageFilter::Warning);
        }
        "ConsoleLevelInfo" => {
            return node.value_number != output.counts().info as f32
                || node.selected != (output.filter() == ConsoleMessageFilter::Info)
                || node.checked != (output.filter() == ConsoleMessageFilter::Info);
        }
        _ => None,
    };
    selected.is_some_and(|selected| node.selected != selected || node.checked != selected)
}

fn patch_console_slot_rows(
    nodes: &ModelRc<host_contract::TemplatePaneNodeData>,
    metadata: &ConsoleOutputPaintMetadata,
    slot_index: usize,
    row_patches: &mut BTreeMap<usize, host_contract::TemplatePaneNodeData>,
) -> Option<()> {
    let (logical_index, logical_line) = metadata.logical_line_for_slot(slot_index, 0.0)?;
    let row_start = metadata
        .line_row_start()
        .saturating_add(slot_index.saturating_mul(metadata.nodes_per_line()));
    let message_row = row_start + usize::from(metadata.nodes_per_line() > 1);
    let mut message = nodes.get(message_row)?.clone();
    apply_console_message_slot(&mut message, logical_index, logical_line, metadata);
    row_patches.insert(message_row, message);
    if metadata.nodes_per_line() > 1 {
        let mut severity = nodes.get(row_start)?.clone();
        apply_console_severity_slot(&mut severity, logical_index, logical_line, metadata);
        row_patches.insert(row_start, severity);
    }
    Some(())
}

fn apply_console_message_slot(
    line: &mut host_contract::TemplatePaneNodeData,
    logical_index: usize,
    logical_line: crate::ui::retained_host::console_output::ConsoleOutputLogicalLineRef<'_>,
    metadata: &ConsoleOutputPaintMetadata,
) {
    line.text = logical_line.text().into();
    line.text_tone = logical_line.text_tone().into();
    line.dispatch_kind = logical_line.dispatch_kind().into();
    line.action_id = logical_line.action_id().into();
    line.frame.y = metadata.line_frame_y(logical_index, 0.0);
    line.frame.height = CONSOLE_OUTPUT_LINE_HEIGHT;
}

fn apply_console_severity_slot(
    severity: &mut host_contract::TemplatePaneNodeData,
    logical_index: usize,
    logical_line: crate::ui::retained_host::console_output::ConsoleOutputLogicalLineRef<'_>,
    metadata: &ConsoleOutputPaintMetadata,
) {
    severity.text = logical_line.severity_text().unwrap_or_default().into();
    severity.text_tone = logical_line.severity_tone().into();
    severity.dispatch_kind.clear();
    severity.action_id.clear();
    severity.frame.y = metadata.line_frame_y(logical_index, 0.0);
    severity.frame.height = CONSOLE_OUTPUT_LINE_HEIGHT;
}

fn project_console_output_lines(
    nodes: &mut Vec<host_contract::TemplatePaneNodeData>,
    output: &ConsoleOutputSnapshot,
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
    let prototype = nodes.get(prototype_index)?.clone();
    let line_origin_y = prototype.frame.y;
    let projects_severity = output.has_output();
    let nodes_per_line = if projects_severity { 2 } else { 1 };
    let metadata = ConsoleOutputPaintMetadata::new_virtualized_snapshot(
        viewport?,
        line_origin_y,
        prototype_index,
        output.clone(),
        nodes_per_line,
        CONSOLE_OUTPUT_OVERSCAN_LINES,
    )?;
    let output_slots = (0..metadata.materialized_line_count()).flat_map(|slot_index| {
        let (logical_index, logical_line) = metadata
            .logical_line_for_slot(slot_index, 0.0)
            .expect("initial console slot must resolve a logical line");
        let mut line = prototype.clone();
        let stable_id = format!("{CONSOLE_OUTPUT_LINE_PREFIX}{slot_index:04}");
        line.node_id = stable_id.clone().into();
        line.control_id = stable_id.into();
        apply_console_message_slot(&mut line, logical_index, logical_line, &metadata);
        let severity = projects_severity.then(|| {
            let mut severity = prototype.clone();
            let stable_id = format!("{CONSOLE_OUTPUT_SEVERITY_PREFIX}{slot_index:04}");
            severity.node_id = stable_id.clone().into();
            severity.control_id = stable_id.into();
            apply_console_severity_slot(&mut severity, logical_index, logical_line, &metadata);
            severity.frame.width = CONSOLE_OUTPUT_SEVERITY_SLOT_WIDTH
                .min((line.frame.width - CONSOLE_OUTPUT_MIN_MESSAGE_SLOT_WIDTH).max(0.0));
            line.frame.x += severity.frame.width;
            line.frame.width = (line.frame.width - severity.frame.width).max(0.0);
            severity
        });
        [severity, Some(line)].into_iter().flatten()
    });
    nodes.splice(prototype_index..=prototype_index, output_slots);
    record_current_ui_perf_counter_batch(|counters| {
        counters.extend_from_slice(&[
            (
                UiPerfCounter::ConsoleProjectionClonedNodeCount,
                metadata.materialized_node_count().saturating_add(1) as f64,
            ),
            (
                UiPerfCounter::ConsoleProjectionFormattedIdCount,
                metadata.materialized_node_count() as f64,
            ),
            (
                UiPerfCounter::ConsoleEnteredLineCount,
                output.line_delta().entered as f64,
            ),
            (
                UiPerfCounter::ConsoleExpiredLineCount,
                output.line_delta().expired as f64,
            ),
            (
                UiPerfCounter::ConsoleSlotReboundCount,
                metadata.materialized_line_count() as f64,
            ),
            (UiPerfCounter::ConsoleProjectionGenerationReuseCount, 0.0),
        ]);
    });
    Some(metadata)
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

#[cfg(test)]
#[path = "console_projection/tests.rs"]
mod tests;
