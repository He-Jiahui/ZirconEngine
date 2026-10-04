use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::{
    current_host_metrics, current_host_palette, HostMaterialPalette,
};
use super::super::render_commands::HostPaintCommand;
use super::{
    component_variant_contains, matches_any_role, node_background, node_radius, push_quad,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum AgentWorkflowKind {
    Plan,
    ToolCalls,
    Approval,
    Usage,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn agent_workflow_kind(
    component_role: &str,
    role: &str,
) -> Option<AgentWorkflowKind> {
    if matches_any_role(component_role, role, &["mui-x-agent-plan", "AgentPlan"]) {
        Some(AgentWorkflowKind::Plan)
    } else if matches_any_role(component_role, role, &["mui-x-tool-calls", "ToolCalls"]) {
        Some(AgentWorkflowKind::ToolCalls)
    } else if matches_any_role(
        component_role,
        role,
        &["mui-x-agent-approval", "AgentApproval"],
    ) {
        Some(AgentWorkflowKind::Approval)
    } else if matches_any_role(component_role, role, &["mui-x-ai-usage", "AIUsage"]) {
        Some(AgentWorkflowKind::Usage)
    } else {
        None
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_agent_workflow(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    kind: AgentWorkflowKind,
) {
    match kind {
        AgentWorkflowKind::Plan => push_agent_plan(commands, node, rect, clip, order, opacity),
        AgentWorkflowKind::ToolCalls => push_tool_calls(commands, node, rect, clip, order, opacity),
        AgentWorkflowKind::Approval => {
            push_agent_approval(commands, node, rect, clip, order, opacity)
        }
        AgentWorkflowKind::Usage => push_ai_usage(commands, node, rect, clip, order, opacity),
    }
}

fn push_agent_plan(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let palette = current_host_palette();
    push_workflow_surface(
        commands,
        node,
        rect,
        clip,
        order,
        opacity,
        palette.surface_inset,
    );
    let title = first_text(node, node.text.as_str(), node.value_text.as_str());
    if !title.is_empty() {
        push_workflow_text(
            commands,
            rect,
            clip,
            order + 1,
            title.clone(),
            palette.text,
            13.0,
            0.10,
            0.10,
            0.80,
            0.10,
            opacity,
            false,
        );
    }

    let progress = normalized_progress(node);
    push_workflow_quad(
        commands,
        rect,
        clip,
        order + 2,
        0.10,
        0.22,
        0.80,
        0.06,
        palette.surface_selected,
        3.0,
        opacity,
    );
    push_workflow_quad(
        commands,
        rect,
        clip,
        order + 3,
        0.10,
        0.22,
        0.80 * progress,
        0.06,
        palette.accent,
        3.0,
        opacity,
    );

    let steps = workflow_values(node);
    let parsed = steps
        .iter()
        .filter_map(|value| parse_workflow_step(value))
        .take(4)
        .collect::<Vec<_>>();
    if parsed.is_empty() {
        return;
    }
    let row_height = 0.62 / parsed.len() as f32;
    for (index, (status, label)) in parsed.into_iter().enumerate() {
        let y = 0.32 + row_height * index as f32;
        push_workflow_quad(
            commands,
            rect,
            clip,
            order + 4 + index as i32,
            0.08,
            y,
            0.84,
            (row_height - 0.02).max(0.10),
            workflow_step_surface(status, palette),
            3.0,
            opacity,
        );
        push_workflow_text(
            commands,
            rect,
            clip,
            order + 8 + index as i32,
            format!("{}  {}", workflow_step_marker(status), label),
            workflow_step_text(status, palette),
            12.0,
            0.12,
            y + 0.02,
            0.74,
            (row_height - 0.04).max(0.07),
            opacity,
            status == WorkflowStepStatus::Active,
        );
    }
}

fn push_tool_calls(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let palette = current_host_palette();
    push_workflow_surface(
        commands,
        node,
        rect,
        clip,
        order,
        opacity,
        palette.surface_inset,
    );
    let title = first_text(node, node.text.as_str(), node.value_text.as_str());
    if !title.is_empty() {
        push_workflow_text(
            commands,
            rect,
            clip,
            order + 1,
            title.clone(),
            palette.text,
            13.0,
            0.10,
            0.09,
            0.80,
            0.10,
            opacity,
            false,
        );
    }
    let limit = if node.notification_visible_limit == 0 {
        4
    } else {
        node.notification_visible_limit.max(1).min(4)
    };
    let calls = workflow_values(node)
        .iter()
        .filter_map(|value| parse_tool_call(value))
        .take(limit)
        .collect::<Vec<_>>();
    if calls.is_empty() {
        return;
    }
    let row_height = 0.72 / calls.len() as f32;
    for (index, call) in calls.into_iter().enumerate() {
        let y = 0.20 + row_height * index as f32;
        push_workflow_quad(
            commands,
            rect,
            clip,
            order + 2 + index as i32,
            0.08,
            y,
            0.84,
            (row_height - 0.02).max(0.12),
            tool_call_surface(call.status, palette),
            3.0,
            opacity,
        );
        push_workflow_text(
            commands,
            rect,
            clip,
            order + 6 + index as i32,
            format!(
                "{}  {}  ·  {}",
                tool_call_marker(call.status),
                call.tool,
                call.detail
            ),
            palette.text,
            12.0,
            0.12,
            y + 0.02,
            0.76,
            (row_height - 0.04).max(0.08),
            opacity,
            false,
        );
    }
}

fn push_agent_approval(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let palette = current_host_palette();
    let surface = if node.validation_level.as_str() == "error" {
        palette.error_container
    } else {
        palette.surface_inset
    };
    push_workflow_surface(commands, node, rect, clip, order, opacity, surface);
    push_workflow_quad(
        commands,
        rect,
        clip,
        order + 1,
        0.04,
        0.04,
        0.92,
        0.04,
        palette.warning,
        2.0,
        opacity,
    );
    let title = first_text(node, node.text.as_str(), node.value_text.as_str());
    if !title.is_empty() {
        push_workflow_text(
            commands,
            rect,
            clip,
            order + 2,
            title.clone(),
            palette.text,
            13.0,
            0.10,
            0.09,
            0.80,
            0.10,
            opacity,
            true,
        );
    }
    if !node.value_text.trim().is_empty() && node.value_text.as_str() != title {
        push_workflow_text(
            commands,
            rect,
            clip,
            order + 3,
            node.value_text.to_string(),
            palette.text,
            12.0,
            0.10,
            0.24,
            0.80,
            0.20,
            opacity,
            false,
        );
    }
    if !node.label_text.trim().is_empty() {
        push_workflow_text(
            commands,
            rect,
            clip,
            order + 4,
            node.label_text.to_string(),
            palette.text_muted,
            11.0,
            0.10,
            0.47,
            0.80,
            0.12,
            opacity,
            false,
        );
    }
    let options = node
        .options
        .iter()
        .map(|value| value.as_str().to_string())
        .collect::<Vec<_>>();
    if let Some(deny) = options.first() {
        push_workflow_quad(
            commands,
            rect,
            clip,
            order + 5,
            0.48,
            0.72,
            0.19,
            0.16,
            palette.surface,
            3.0,
            opacity,
        );
        push_workflow_text(
            commands,
            rect,
            clip,
            order + 7,
            deny.clone(),
            palette.text,
            12.0,
            0.50,
            0.74,
            0.15,
            0.12,
            opacity,
            true,
        );
    }
    if let Some(allow) = options.get(1) {
        let tone = if component_variant_contains(node, "destructive") {
            palette.error
        } else {
            palette.accent
        };
        push_workflow_quad(
            commands,
            rect,
            clip,
            order + 6,
            0.70,
            0.72,
            0.20,
            0.16,
            tone,
            3.0,
            opacity,
        );
        push_workflow_text(
            commands,
            rect,
            clip,
            order + 8,
            allow.clone(),
            palette.text,
            12.0,
            0.72,
            0.74,
            0.16,
            0.12,
            opacity,
            true,
        );
    }
}

fn push_ai_usage(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let palette = current_host_palette();
    push_workflow_surface(
        commands,
        node,
        rect,
        clip,
        order,
        opacity,
        palette.surface_inset,
    );
    let title = first_text(node, node.text.as_str(), node.value_text.as_str());
    if !title.is_empty() {
        push_workflow_text(
            commands,
            rect,
            clip,
            order + 1,
            title.clone(),
            palette.text,
            13.0,
            0.10,
            0.10,
            0.80,
            0.10,
            opacity,
            false,
        );
    }
    let progress = normalized_progress(node);
    push_workflow_quad(
        commands,
        rect,
        clip,
        order + 2,
        0.08,
        0.52,
        0.84,
        0.08,
        palette.surface_selected,
        3.0,
        opacity,
    );
    push_workflow_quad(
        commands,
        rect,
        clip,
        order + 3,
        0.08,
        0.52,
        0.84 * progress,
        0.08,
        palette.accent,
        3.0,
        opacity,
    );
    if !node.value_text.trim().is_empty() && node.value_text.as_str() != title {
        push_workflow_text(
            commands,
            rect,
            clip,
            order + 4,
            node.value_text.to_string(),
            palette.text_muted,
            11.0,
            0.10,
            0.68,
            0.80,
            0.12,
            opacity,
            false,
        );
    }
}

fn push_workflow_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    fallback: [u8; 4],
) {
    push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        node_background(node).unwrap_or(fallback),
        node.border_width.max(1.0),
        node_radius(node).max(6.0),
        opacity,
    );
}

fn push_workflow_quad(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [u8; 4],
    radius: f32,
    opacity: f32,
) {
    let frame = normalized_frame(rect, x, y, width, height);
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return;
    }
    push_quad(commands, frame, clip, order, color, 0.0, radius, opacity);
}

#[allow(clippy::too_many_arguments)]
fn push_workflow_text(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text: String,
    color: [u8; 4],
    font_size: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    opacity: f32,
    strong: bool,
) {
    let frame = normalized_frame(rect, x, y, width, height);
    if frame.width <= 0.0 || frame.height <= 0.0 || text.trim().is_empty() {
        return;
    }
    let metrics = current_host_metrics();
    let size = font_size.max(1.0);
    let line_height = metrics.line_height(size).max(size);
    if frame.height < line_height * 0.5 {
        return;
    }
    let mut style = UiTextRunPaintStyle::default();
    if strong {
        style.strong = true;
    }
    commands.push(HostPaintCommand::wrapped_text(
        frame,
        Some(clip.clone()),
        order,
        text,
        color,
        size,
        line_height,
        style,
        opacity,
    ));
}

fn normalized_frame(rect: &FrameRect, x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x: rect.x + rect.width * x.clamp(0.0, 1.0),
        y: rect.y + rect.height * y.clamp(0.0, 1.0),
        width: (rect.width * width.clamp(0.0, 1.0)).max(0.0),
        height: (rect.height * height.clamp(0.0, 1.0)).max(0.0),
    }
}

fn first_text<'a>(node: &'a TemplatePaneNodeData, first: &'a str, second: &'a str) -> String {
    if !first.trim().is_empty() {
        first.to_string()
    } else if !second.trim().is_empty() {
        second.to_string()
    } else {
        node.label_text.to_string()
    }
}

fn workflow_values(node: &TemplatePaneNodeData) -> Vec<String> {
    let values = node
        .collection_items
        .iter()
        .map(|value| value.as_str().trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if values.is_empty() && !node.text.trim().is_empty() {
        node.text
            .split('\n')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect()
    } else {
        values
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkflowStepStatus {
    Done,
    Active,
    Next,
}

fn parse_workflow_step(value: &str) -> Option<(WorkflowStepStatus, String)> {
    let (raw_status, label) = value.split_once('|')?;
    let status = match raw_status.trim().to_ascii_lowercase().as_str() {
        "done" => WorkflowStepStatus::Done,
        "active" => WorkflowStepStatus::Active,
        "next" => WorkflowStepStatus::Next,
        _ => return None,
    };
    let label = label.trim();
    (!label.is_empty()).then(|| (status, label.to_string()))
}

fn workflow_step_marker(status: WorkflowStepStatus) -> &'static str {
    match status {
        WorkflowStepStatus::Done => "✓",
        WorkflowStepStatus::Active => "◉",
        WorkflowStepStatus::Next => "○",
    }
}

fn workflow_step_surface(status: WorkflowStepStatus, palette: HostMaterialPalette) -> [u8; 4] {
    match status {
        WorkflowStepStatus::Done => palette.success_container,
        WorkflowStepStatus::Active => palette.info_container,
        WorkflowStepStatus::Next => palette.surface,
    }
}

fn workflow_step_text(status: WorkflowStepStatus, palette: HostMaterialPalette) -> [u8; 4] {
    match status {
        WorkflowStepStatus::Done => palette.success,
        WorkflowStepStatus::Active => palette.text,
        WorkflowStepStatus::Next => palette.text_muted,
    }
}

#[derive(Clone, Copy)]
enum ToolCallStatus {
    Success,
    Pending,
    Failure,
}

struct ToolCall {
    status: ToolCallStatus,
    tool: String,
    detail: String,
}

fn parse_tool_call(value: &str) -> Option<ToolCall> {
    let mut parts = value.split('|');
    let status = match parts.next()?.trim().to_ascii_lowercase().as_str() {
        "success" => ToolCallStatus::Success,
        "pending" => ToolCallStatus::Pending,
        "failure" | "error" => ToolCallStatus::Failure,
        _ => return None,
    };
    let tool = parts.next()?.trim();
    let detail = parts.collect::<Vec<_>>().join("|");
    if tool.is_empty() || detail.trim().is_empty() {
        return None;
    }
    Some(ToolCall {
        status,
        tool: tool.to_string(),
        detail: detail.trim().to_string(),
    })
}

fn tool_call_marker(status: ToolCallStatus) -> &'static str {
    match status {
        ToolCallStatus::Success => "✓",
        ToolCallStatus::Pending => "◌",
        ToolCallStatus::Failure => "!",
    }
}

fn tool_call_surface(status: ToolCallStatus, palette: HostMaterialPalette) -> [u8; 4] {
    match status {
        ToolCallStatus::Success => palette.success_container,
        ToolCallStatus::Pending => palette.info_container,
        ToolCallStatus::Failure => palette.error_container,
    }
}

fn normalized_progress(node: &TemplatePaneNodeData) -> f32 {
    if node.value_percent > 0.0 {
        return node.value_percent.clamp(0.0, 1.0);
    }
    if node.value_number > 1.0 {
        (node.value_number / 100.0).clamp(0.0, 1.0)
    } else {
        node.value_number.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
#[path = "tests/agent_workflow.rs"]
mod tests;
