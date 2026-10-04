use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiRenderCommand, UiResolvedStyle},
    tree::UiTemplateNodeMetadata,
};

use super::shared::{
    bool_attribute, collection_window, component_matches, normalized_state, number_attribute, quad,
    rich_text, string_array, text, text_attribute, text_color, ACCENT, BORDER, ERROR, INFO,
    SUCCESS, SURFACE_INSET, TEXT_MUTED, TEXT_SECONDARY, WARNING,
};

// These are local flow-spacing tokens, not viewport coordinates. Rendered
// frames are derived from the already-arranged owner frame below.
const FLOW_INSET: f32 = 8.0;
const FLOW_GAP: f32 = 4.0;

pub(super) fn supports(metadata: &UiTemplateNodeMetadata) -> bool {
    component_matches(
        metadata,
        &[
            "AgentPlan",
            "mui-x-agent-plan",
            "ToolCalls",
            "mui-x-tool-calls",
            "AgentApproval",
            "mui-x-agent-approval",
            "AIUsage",
            "mui-x-ai-usage",
        ],
    )
}

pub(super) fn render(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
) -> Vec<UiRenderCommand> {
    if component_matches(metadata, &["AgentPlan", "mui-x-agent-plan"]) {
        return agent_plan(
            node_id, metadata, frame, clip_frame, z_index, opacity, base_style,
        );
    }
    if component_matches(metadata, &["ToolCalls", "mui-x-tool-calls"]) {
        return tool_calls(
            node_id, metadata, frame, clip_frame, z_index, opacity, base_style,
        );
    }
    if component_matches(metadata, &["AgentApproval", "mui-x-agent-approval"]) {
        return agent_approval(
            node_id, metadata, frame, clip_frame, z_index, opacity, base_style,
        );
    }
    ai_usage(
        node_id, metadata, frame, clip_frame, z_index, opacity, base_style,
    )
}

fn agent_plan(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
) -> Vec<UiRenderCommand> {
    let content = flow_content_frame(frame);
    let title = text_attribute(metadata, &["text", "title"]);
    let title_height = title
        .as_ref()
        .map(|_| flow_line_height(base_style, 11.0))
        .unwrap_or_default();
    let rows_top = content.y + title_height + title.as_ref().map_or(0.0, |_| FLOW_GAP);
    let progress_height = FLOW_GAP;
    let rows_available = (content.bottom() - rows_top - progress_height - FLOW_GAP).max(0.0);
    let row_baseline = flow_line_height(base_style, 10.0);
    let physical_capacity = (rows_available > 0.0)
        .then(|| ((rows_available + FLOW_GAP) / (row_baseline + FLOW_GAP)).floor() as usize)
        .unwrap_or_default();
    let items = collection_window(metadata, "collection_items", physical_capacity);
    let row_height = (!items.is_empty()).then_some(row_baseline);
    let mut commands = Vec::new();
    if let Some(title) = title {
        commands.push(text(
            node_id,
            UiFrame::new(content.x, content.y, content.width, title_height),
            clip_frame,
            z_index.saturating_add(1),
            title,
            text_color(base_style),
            11.0,
            base_style,
            opacity,
        ));
    }
    for (index, item) in items.into_iter().enumerate() {
        let (state, label) = split_item(&item);
        let row_height = row_height.expect("nonempty item list has a row height");
        let y = rows_top + index as f32 * (row_height + FLOW_GAP);
        let marker_extent = (row_baseline * 0.45).max(1.0);
        let marker = UiFrame::new(
            content.x,
            y + (row_height - marker_extent) * 0.5,
            marker_extent,
            marker_extent,
        );
        commands.push(quad(
            node_id,
            marker,
            clip_frame,
            z_index.saturating_add(2 + index as i32 * 2),
            status_color(state),
            None,
            0.0,
            FLOW_GAP,
            base_style,
            opacity,
        ));
        commands.push(text(
            node_id,
            UiFrame::new(
                marker.right() + FLOW_GAP * 1.5,
                y,
                (content.right() - marker.right() - FLOW_GAP * 1.5).max(1.0),
                row_height,
            ),
            clip_frame,
            z_index.saturating_add(3 + index as i32 * 2),
            label,
            if is_terminal_success(state) {
                SUCCESS
            } else if is_failure(state) {
                ERROR
            } else {
                TEXT_SECONDARY
            },
            10.0,
            base_style,
            opacity,
        ));
    }
    let progress = progress_fraction(metadata);
    let track = UiFrame::new(
        content.x,
        content.bottom() - progress_height,
        content.width,
        progress_height,
    );
    commands.push(quad(
        node_id,
        track,
        clip_frame,
        z_index.saturating_add(30),
        SURFACE_INSET,
        None,
        0.0,
        FLOW_GAP * 0.5,
        base_style,
        opacity,
    ));
    commands.push(quad(
        node_id,
        UiFrame::new(
            track.x,
            track.y,
            (track.width * progress).max(1.0),
            track.height,
        ),
        clip_frame,
        z_index.saturating_add(31),
        if progress >= 1.0 { SUCCESS } else { ACCENT },
        None,
        0.0,
        FLOW_GAP * 0.5,
        base_style,
        opacity,
    ));
    commands
}

fn tool_calls(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
) -> Vec<UiRenderCommand> {
    let content = flow_content_frame(frame);
    let title = text_attribute(metadata, &["text", "title"]);
    let title_height = title
        .as_ref()
        .map(|_| flow_line_height(base_style, 11.0))
        .unwrap_or_default();
    let rows_top = content.y + title_height + title.as_ref().map_or(0.0, |_| FLOW_GAP);
    let row_baseline = flow_line_height(base_style, 10.0);
    let rows_available = (content.bottom() - rows_top).max(0.0);
    let physical_capacity = (rows_available > 0.0)
        .then(|| ((rows_available + FLOW_GAP) / (row_baseline + FLOW_GAP)).floor() as usize)
        .unwrap_or_default();
    let items = collection_window(metadata, "collection_items", physical_capacity);
    let row_height = (!items.is_empty()).then_some(row_baseline);
    let mut commands = Vec::new();
    if let Some(title) = title {
        commands.push(text(
            node_id,
            UiFrame::new(content.x, content.y, content.width, title_height),
            clip_frame,
            z_index.saturating_add(1),
            title,
            text_color(base_style),
            11.0,
            base_style,
            opacity,
        ));
    }
    for (index, item) in items.into_iter().enumerate() {
        let parts = item.split('|').map(str::trim).collect::<Vec<_>>();
        let state = parts.first().copied().unwrap_or("normal");
        let tool = parts.get(1).copied().unwrap_or_default();
        let detail = parts.get(2).copied().unwrap_or_default();
        if tool.is_empty() {
            continue;
        }
        let row_height = row_height.expect("nonempty item list has a row height");
        let y = rows_top + index as f32 * (row_height + FLOW_GAP);
        let row = UiFrame::new(content.x, y, content.width, row_height);
        commands.push(quad(
            node_id,
            row,
            clip_frame,
            z_index.saturating_add(2 + index as i32 * 3),
            SURFACE_INSET,
            Some(BORDER.to_string()),
            1.0,
            FLOW_GAP,
            base_style,
            opacity,
        ));
        commands.push(quad(
            node_id,
            UiFrame::new(row.x, row.y, FLOW_GAP * 0.75, row.height),
            clip_frame,
            z_index.saturating_add(3 + index as i32 * 3),
            status_color(state),
            None,
            0.0,
            FLOW_GAP * 0.5,
            base_style,
            opacity,
        ));
        let value = rich_tool_call_text(tool, detail);
        commands.push(rich_text(
            node_id,
            UiFrame::new(
                row.x + FLOW_GAP * 2.25,
                row.y + FLOW_GAP * 0.75,
                (row.width - FLOW_GAP * 4.5).max(1.0),
                (row.height - FLOW_GAP * 1.5).max(1.0),
            ),
            clip_frame,
            z_index.saturating_add(4 + index as i32 * 3),
            value,
            TEXT_SECONDARY,
            10.0,
            base_style,
            opacity,
        ));
    }
    commands
}

fn agent_approval(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
) -> Vec<UiRenderCommand> {
    let title = text_attribute(metadata, &["text", "title"]);
    let body = text_attribute(metadata, &["value_text", "message", "body"]);
    let scope = text_attribute(metadata, &["label_text", "scope"]);
    let options = string_array(metadata, "options");
    let rejected = normalized_state(metadata, "approval_state")
        .is_some_and(|value| value == "denied")
        || bool_attribute(metadata, "destructive").unwrap_or(false);
    let accent = if rejected { WARNING } else { SUCCESS };
    let content = flow_content_frame(frame);
    let title_height = title
        .as_ref()
        .map(|_| flow_line_height(base_style, 11.0))
        .unwrap_or_default();
    let mut cursor_y = content.y;
    let action_labels = options;
    let action_height = (!action_labels.is_empty())
        .then(|| flow_line_height(base_style, 9.0) + FLOW_GAP * 1.5)
        .unwrap_or_default();
    let action_y = content.bottom() - action_height;
    let action_gap = (!action_labels.is_empty())
        .then_some(FLOW_GAP)
        .unwrap_or_default();
    let text_bottom = action_y - action_gap;
    let scope_height = scope
        .as_ref()
        .map(|_| flow_line_height(base_style, 9.0))
        .unwrap_or_default();
    let mut commands = Vec::new();
    if let Some(title) = title {
        commands.push(text(
            node_id,
            UiFrame::new(content.x, cursor_y, content.width, title_height),
            clip_frame,
            z_index.saturating_add(1),
            title,
            accent,
            11.0,
            base_style,
            opacity,
        ));
        cursor_y += title_height + FLOW_GAP;
    }
    if let Some(body) = body {
        let scope_gap = scope.as_ref().map_or(0.0, |_| FLOW_GAP);
        let body_height = (text_bottom - cursor_y - scope_height - scope_gap).max(0.0);
        commands.push(text(
            node_id,
            UiFrame::new(content.x, cursor_y, content.width, body_height),
            clip_frame,
            z_index.saturating_add(2),
            body,
            text_color(base_style),
            10.0,
            base_style,
            opacity,
        ));
    }
    if let Some(scope) = scope {
        commands.push(text(
            node_id,
            UiFrame::new(
                content.x,
                (text_bottom - scope_height).max(cursor_y),
                content.width,
                scope_height,
            ),
            clip_frame,
            z_index.saturating_add(3),
            scope,
            TEXT_SECONDARY,
            9.0,
            base_style,
            opacity,
        ));
    }
    let action_count = action_labels.len();
    let action_width = (action_count > 0).then(|| {
        ((content.width - FLOW_GAP * (action_count.saturating_sub(1) as f32)) / action_count as f32)
            .max(1.0)
    });
    for (index, label) in action_labels.into_iter().enumerate() {
        let action_width = action_width.expect("action labels have a shared width");
        let action = UiFrame::new(
            content.x + index as f32 * (action_width + FLOW_GAP),
            action_y,
            action_width,
            action_height,
        );
        let filled = index + 1 == action_count;
        commands.push(quad(
            node_id,
            action,
            clip_frame,
            z_index.saturating_add(4 + index as i32 * 2),
            if filled { accent } else { SURFACE_INSET },
            (!filled).then(|| BORDER.to_string()),
            if filled { 0.0 } else { 1.0 },
            FLOW_GAP,
            base_style,
            opacity,
        ));
        commands.push(text(
            node_id,
            UiFrame::new(
                action.x + FLOW_GAP,
                action.y + FLOW_GAP * 0.75,
                (action.width - FLOW_GAP * 2.0).max(1.0),
                (action.height - FLOW_GAP * 1.5).max(1.0),
            ),
            clip_frame,
            z_index.saturating_add(5 + index as i32 * 2),
            label,
            if filled {
                SURFACE_INSET
            } else {
                TEXT_SECONDARY
            },
            9.0,
            base_style,
            opacity,
        ));
    }
    commands
}

fn ai_usage(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
) -> Vec<UiRenderCommand> {
    let title = text_attribute(metadata, &["text", "title"]);
    let detail = text_attribute(metadata, &["value_text", "detail", "value_label"]);
    let state = normalized_state(metadata, "component_variant").unwrap_or_else(|| "normal".into());
    let color = status_color(&state);
    let content = flow_content_frame(frame);
    let title_height = title
        .as_ref()
        .map(|_| flow_line_height(base_style, 11.0))
        .unwrap_or_default();
    let track_height = FLOW_GAP * 1.75;
    let track_y = content.y + title_height + title.as_ref().map_or(0.0, |_| FLOW_GAP);
    let track = UiFrame::new(content.x, track_y, content.width, track_height);
    let fraction = progress_fraction(metadata);
    let mut commands = Vec::new();
    if let Some(title) = title {
        commands.push(text(
            node_id,
            UiFrame::new(content.x, content.y, track.width, title_height),
            clip_frame,
            z_index.saturating_add(1),
            title,
            text_color(base_style),
            11.0,
            base_style,
            opacity,
        ));
    }
    commands.push(quad(
        node_id,
        track,
        clip_frame,
        z_index.saturating_add(2),
        SURFACE_INSET,
        Some(BORDER.to_string()),
        1.0,
        FLOW_GAP,
        base_style,
        opacity,
    ));
    commands.push(quad(
        node_id,
        UiFrame::new(
            track.x,
            track.y,
            (track.width * fraction).max(1.0),
            track.height,
        ),
        clip_frame,
        z_index.saturating_add(3),
        color,
        None,
        0.0,
        FLOW_GAP,
        base_style,
        opacity,
    ));
    if let Some(detail) = detail {
        commands.push(text(
            node_id,
            UiFrame::new(
                content.x,
                track.bottom() + FLOW_GAP,
                track.width,
                (content.bottom() - track.bottom() - FLOW_GAP).max(0.0),
            ),
            clip_frame,
            z_index.saturating_add(4),
            detail,
            TEXT_SECONDARY,
            10.0,
            base_style,
            opacity,
        ));
    }
    commands
}

fn flow_content_frame(frame: UiFrame) -> UiFrame {
    UiFrame::new(
        frame.x + FLOW_INSET,
        frame.y + FLOW_INSET,
        (frame.width - FLOW_INSET * 2.0).max(1.0),
        (frame.height - FLOW_INSET * 2.0).max(1.0),
    )
}

fn flow_line_height(base_style: &UiResolvedStyle, requested_font_size: f32) -> f32 {
    base_style.line_height.max(requested_font_size).max(1.0)
}

// 工具名与详情共用一条富文本命令，让强调样式仍参与同一文本布局。
fn rich_tool_call_text(tool: &str, detail: &str) -> String {
    // BUG: [CR-R02-runtime_wave5_surface_widget_render-0003] 工具名或详情含转义字符时，此编码交给不识别转义的 MarkdownInlineV1；例如详情 *x* 会留下反斜杠并误作斜体，parser/markdown.rs 的扫描未跳过转义标记。
    let tool = escape_markdown_inline(tool);
    if detail.is_empty() {
        format!("**{tool}**")
    } else {
        format!("**{tool}**  |  {}", escape_markdown_inline(detail))
    }
}

fn escape_markdown_inline(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(character, '\\' | '*' | '_' | '[' | ']' | '(' | ')' | '`') {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

fn progress_fraction(metadata: &UiTemplateNodeMetadata) -> f32 {
    let value = number_attribute(metadata, "value").unwrap_or(0.0);
    let min = number_attribute(metadata, "min").unwrap_or(0.0);
    let max = number_attribute(metadata, "max").unwrap_or(1.0);
    let span = max - min;
    if span.is_finite() && span.abs() > f32::EPSILON {
        ((value - min) / span).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn split_item(item: &str) -> (&str, &str) {
    item.split_once('|')
        .map(|(state, label)| (state.trim(), label.trim()))
        .unwrap_or(("normal", item.trim()))
}

fn status_color(state: &str) -> &'static str {
    match state.trim().to_ascii_lowercase().as_str() {
        "done" | "success" | "complete" | "approved" => SUCCESS,
        "active" | "running" | "pending" | "selected" => INFO,
        "warning" | "review" | "blocked" => WARNING,
        "failure" | "error" | "denied" | "exceeded" => ERROR,
        _ => TEXT_MUTED,
    }
}

fn is_terminal_success(state: &str) -> bool {
    matches!(
        state.trim().to_ascii_lowercase().as_str(),
        "done" | "success" | "complete" | "approved"
    )
}

fn is_failure(state: &str) -> bool {
    matches!(
        state.trim().to_ascii_lowercase().as_str(),
        "failure" | "error" | "denied" | "blocked"
    )
}

#[cfg(test)]
#[path = "tests/agent_workflow.rs"]
mod tests;
