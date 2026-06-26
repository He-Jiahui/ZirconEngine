use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
    tree::UiTemplateNodeMetadata,
};

use super::painter_state::UiRenderPainterStateSource;

const SEGMENTED_BACKGROUND: &str = "#1d2327";
const SEGMENTED_BORDER: &str = "#323a41";
const SEGMENTED_SELECTED_SURFACE: &str = "#173942";
const SEGMENTED_SELECTED_BORDER: &str = "#2aa6b8";
const SEGMENTED_HOVER: &str = "#2a3036";
const SEGMENTED_PRESSED: &str = "#20262b";
const SEGMENTED_DISABLED: &str = "#191d22";
const TEXT: &str = "#e6f1f4";
const TEXT_MUTED: &str = "#8fa3ac";
const TEXT_DISABLED: &str = "#58656c";
const GROUP_LABEL: &str = "#a1acb2";
const FONT_SIZE: f32 = 11.0;
const LINE_HEIGHT: f32 = FONT_SIZE * 1.2;
const SEGMENT_TEXT_INSET_X: f32 = 8.0;
const SEGMENT_TEXT_INSET_Y: f32 = 5.0;
const SEGMENT_RADIUS: f32 = 5.0;
const SELECTED_INSET: f32 = 2.0;
const GROUP_LABEL_HEIGHT: f32 = 14.0;
const GROUP_LABEL_GAP: f32 = 4.0;
const TAB_FONT_SIZE: f32 = 12.0;
const TAB_LINE_HEIGHT: f32 = TAB_FONT_SIZE * 1.2;
const TAB_TEXT_INSET_X: f32 = 12.0;
const TAB_UNDERLINE_HEIGHT: f32 = 2.0;

pub(super) fn segmented_control_suppresses_owner_text(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_segmented_or_tab)
}

pub(super) fn segmented_control_render_commands(
    node_id: UiNodeId,
    metadata: Option<&UiTemplateNodeMetadata>,
    state_flags: &UiStateFlags,
    component_state: Option<&UiComponentState>,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let Some(metadata) = metadata else {
        return Vec::new();
    };
    if frame.width <= 1.0 || frame.height <= 1.0 {
        return Vec::new();
    }

    let state = SegmentedRenderState::resolve(metadata, state_flags, component_state);
    match control_kind(metadata) {
        Some(SegmentedControlKind::SegmentedControl) => segmented_commands(
            node_id, metadata, &state, frame, clip_frame, z_index, opacity,
        ),
        Some(SegmentedControlKind::Tab) => tab_commands(
            node_id, metadata, &state, frame, clip_frame, z_index, opacity,
        ),
        None => Vec::new(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SegmentedControlKind {
    SegmentedControl,
    Tab,
}

#[derive(Clone, Copy)]
struct SegmentedRenderState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
    active: bool,
}

impl SegmentedRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let component_flags = component_state.map(|state| &state.flags);
        let checked = component_flags.is_some_and(|flags| flags.checked)
            || state_flags.checked
            || bool_attribute(metadata, "checked").unwrap_or(false);
        let selected = checked
            || component_flags.is_some_and(|flags| flags.selected)
            || bool_attribute(metadata, "selected").unwrap_or(false);
        let mut painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        painter_state.checked = checked;
        painter_state.selected = selected;
        let family = UiPainterFamily::Tab;
        Self {
            family,
            visual_state: painter_state.resolved_state_for_family(family),
            active: selected,
        }
    }

    fn unavailable(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
        )
    }

    fn pressed(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Pressed)
    }

    fn hot(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Hovered
                | UiPainterResolvedState::Focused
                | UiPainterResolvedState::Open
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
        )
    }
}

fn segmented_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let options = segmented_options(metadata);
    if options.is_empty() {
        return Vec::new();
    }

    let mut commands = Vec::new();
    if let Some(label) = group_label(metadata) {
        commands.push(text_command(
            node_id,
            UiFrame::new(frame.x, frame.y, frame.width, GROUP_LABEL_HEIGHT),
            clip_frame,
            z_index.saturating_add(3),
            label,
            group_label_color(metadata, state),
            FONT_SIZE,
            LINE_HEIGHT,
            state,
            opacity,
        ));
    }

    let body = segmented_body_frame(metadata, frame);
    commands.push(quad_command(
        node_id,
        body,
        clip_frame,
        z_index.saturating_add(1),
        segmented_background(metadata, state),
        Some(segmented_border(metadata, state)),
        border_width(metadata),
        corner_radius(metadata),
        state,
        opacity,
    ));

    let selected = selected_segment_value(metadata);
    for (index, option) in options.iter().enumerate() {
        let segment = segment_frame(body, index, options.len());
        if index > 0 {
            commands.push(quad_command(
                node_id,
                UiFrame::new(
                    segment.x,
                    segment.y + 4.0,
                    1.0,
                    (segment.height - 8.0).max(1.0),
                ),
                clip_frame,
                z_index.saturating_add(2),
                divider_color(metadata, state),
                None,
                0.0,
                0.0,
                state,
                opacity,
            ));
        }
        let option_selected = option_is_selected(option, selected.as_deref());
        if option_selected {
            push_selected_segment(
                &mut commands,
                node_id,
                metadata,
                state,
                segment,
                clip_frame,
                z_index.saturating_add(3),
                opacity,
            );
        }
        commands.push(text_command(
            node_id,
            UiFrame::new(
                segment.x + SEGMENT_TEXT_INSET_X,
                segment.y + SEGMENT_TEXT_INSET_Y,
                (segment.width - SEGMENT_TEXT_INSET_X * 2.0).max(1.0),
                (segment.height - SEGMENT_TEXT_INSET_Y * 2.0).max(LINE_HEIGHT),
            ),
            clip_frame,
            z_index.saturating_add(5),
            option_label(option),
            option_text_color(metadata, state, option_selected),
            FONT_SIZE,
            LINE_HEIGHT,
            state,
            opacity,
        ));
    }
    commands
}

fn tab_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mut commands = Vec::new();
    if let Some(background) = tab_background(metadata, state) {
        commands.push(quad_command(
            node_id,
            frame,
            clip_frame,
            z_index.saturating_add(1),
            background,
            None,
            0.0,
            0.0,
            state,
            opacity,
        ));
    }
    if state.active {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                frame.x,
                frame.y + (frame.height - TAB_UNDERLINE_HEIGHT).max(0.0),
                frame.width,
                TAB_UNDERLINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(3),
            selected_underline(metadata, state),
            None,
            0.0,
            0.0,
            state,
            opacity,
        ));
    }
    if let Some(label) = tab_label(metadata) {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                frame.x + TAB_TEXT_INSET_X,
                frame.y + (frame.height - TAB_LINE_HEIGHT).max(0.0) * 0.5,
                (frame.width - TAB_TEXT_INSET_X * 2.0).max(1.0),
                TAB_LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(4),
            label,
            tab_text_color(metadata, state),
            TAB_FONT_SIZE,
            TAB_LINE_HEIGHT,
            state,
            opacity,
        ));
    }
    commands
}

fn push_selected_segment(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
    segment: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) {
    let selected = inset_frame(segment, SELECTED_INSET);
    commands.push(quad_command(
        node_id,
        selected,
        clip_frame,
        z_index,
        selected_surface(metadata, state),
        selected_border_width(metadata)
            .gt(&0.0)
            .then(|| selected_border(metadata, state)),
        selected_border_width(metadata),
        (corner_radius(metadata) - 1.0).max(0.0),
        state,
        opacity,
    ));
    let underline_height = selected_underline_height(metadata);
    if underline_height > 0.0 {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                selected.x,
                selected.y + (selected.height - underline_height).max(0.0),
                selected.width,
                underline_height.min(selected.height).max(1.0),
            ),
            clip_frame,
            z_index.saturating_add(1),
            selected_underline(metadata, state),
            None,
            0.0,
            0.0,
            state,
            opacity,
        ));
    }
}

fn control_kind(metadata: &UiTemplateNodeMetadata) -> Option<SegmentedControlKind> {
    match metadata.component.as_str() {
        "SegmentedControl" | "Segmented" => Some(SegmentedControlKind::SegmentedControl),
        "Tab" | "PanelTab" => Some(SegmentedControlKind::Tab),
        _ => None,
    }
}

fn is_segmented_or_tab(metadata: &UiTemplateNodeMetadata) -> bool {
    control_kind(metadata).is_some()
}

fn segmented_body_frame(metadata: &UiTemplateNodeMetadata, frame: UiFrame) -> UiFrame {
    let label_block = group_label(metadata)
        .map(|_| GROUP_LABEL_HEIGHT + GROUP_LABEL_GAP)
        .unwrap_or(0.0);
    UiFrame::new(
        frame.x + number_attribute(metadata, "layout_offset_x").unwrap_or(0.0),
        frame.y + label_block + number_attribute(metadata, "layout_offset_y").unwrap_or(0.0),
        frame.width,
        (frame.height - label_block).max(1.0),
    )
}

fn segment_frame(frame: UiFrame, index: usize, count: usize) -> UiFrame {
    let count = count.max(1);
    let width = frame.width / count as f32;
    let x = frame.x + width * index as f32;
    UiFrame::new(
        x,
        frame.y,
        if index + 1 == count {
            frame.x + frame.width - x
        } else {
            width
        }
        .max(1.0),
        frame.height,
    )
}

fn inset_frame(frame: UiFrame, inset: f32) -> UiFrame {
    UiFrame::new(
        frame.x + inset,
        frame.y + inset,
        (frame.width - inset * 2.0).max(1.0),
        (frame.height - inset * 2.0).max(1.0),
    )
}

fn segmented_options(metadata: &UiTemplateNodeMetadata) -> Vec<String> {
    metadata
        .attributes
        .get("options")
        .and_then(Value::as_array)
        .map(|options| {
            options
                .iter()
                .filter_map(option_string)
                .filter(|option| !option.trim().is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn option_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.to_string()),
        Value::Table(table) => ["label", "text", "value", "id", "name"]
            .iter()
            .find_map(|key| table.get(*key))
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    }
}

fn selected_segment_value(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["value", "value_text", "selected", "text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
}

fn option_is_selected(option: &str, selected: Option<&str>) -> bool {
    selected.is_some_and(|selected| option.trim().eq_ignore_ascii_case(selected))
}

fn option_label(option: &str) -> String {
    let trimmed = option.trim();
    let mut chars = trimmed.chars();
    match chars.next() {
        Some(first) => {
            let mut label = first.to_ascii_uppercase().to_string();
            label.push_str(chars.as_str());
            label
        }
        None => String::new(),
    }
}

fn group_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["label", "label_text", "group_label"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_string)
}

fn tab_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["text", "label", "value_text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_string)
}

fn segmented_background<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
) -> &'a str {
    if state.unavailable() {
        SEGMENTED_DISABLED
    } else if state.pressed() {
        color_attribute(metadata, "pressed_background_color").unwrap_or(SEGMENTED_PRESSED)
    } else if state.hot() {
        color_attribute(metadata, "hover_background_color").unwrap_or(SEGMENTED_HOVER)
    } else {
        color_attribute(metadata, "background_color").unwrap_or(SEGMENTED_BACKGROUND)
    }
}

fn segmented_border<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
) -> &'a str {
    if state.unavailable() {
        "#334852"
    } else if state.pressed() || state.hot() {
        color_attribute(metadata, "focus_border_color").unwrap_or(SEGMENTED_SELECTED_BORDER)
    } else {
        color_attribute(metadata, "border_color").unwrap_or(SEGMENTED_BORDER)
    }
}

fn divider_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
) -> &'a str {
    segmented_border(metadata, state)
}

fn selected_surface<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
) -> &'a str {
    if state.unavailable() {
        SEGMENTED_DISABLED
    } else {
        color_attribute(metadata, "selected_background_color").unwrap_or(SEGMENTED_SELECTED_SURFACE)
    }
}

fn selected_border<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
) -> &'a str {
    if state.unavailable() {
        "#334852"
    } else {
        color_attribute(metadata, "selected_border_color").unwrap_or(SEGMENTED_SELECTED_BORDER)
    }
}

fn selected_underline<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
) -> &'a str {
    if state.unavailable() {
        TEXT_DISABLED
    } else {
        color_attribute(metadata, "selected_underline_color")
            .or_else(|| color_attribute(metadata, "accent_color"))
            .unwrap_or(SEGMENTED_SELECTED_BORDER)
    }
}

fn option_text_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
    selected: bool,
) -> &'a str {
    if state.unavailable() {
        TEXT_DISABLED
    } else if selected {
        color_attribute(metadata, "selected_foreground_color")
            .or_else(|| color_attribute(metadata, "selected_text_color"))
            .unwrap_or(TEXT)
    } else {
        color_attribute(metadata, "foreground_color")
            .or_else(|| color_attribute(metadata, "idle_text_color"))
            .unwrap_or(TEXT_MUTED)
    }
}

fn group_label_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
) -> &'a str {
    if state.unavailable() {
        TEXT_DISABLED
    } else {
        color_attribute(metadata, "label_color").unwrap_or(GROUP_LABEL)
    }
}

fn tab_background<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
) -> Option<&'a str> {
    if state.unavailable() {
        Some(SEGMENTED_DISABLED)
    } else if state.pressed() {
        Some(color_attribute(metadata, "pressed_background_color").unwrap_or(SEGMENTED_PRESSED))
    } else if state.hot() {
        Some(color_attribute(metadata, "hover_background_color").unwrap_or(SEGMENTED_HOVER))
    } else {
        color_attribute(metadata, "background_color")
    }
}

fn tab_text_color<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
) -> &'a str {
    if state.unavailable() {
        TEXT_DISABLED
    } else if state.active {
        color_attribute(metadata, "selected_foreground_color")
            .or_else(|| color_attribute(metadata, "selected_text_color"))
            .unwrap_or(TEXT)
    } else {
        color_attribute(metadata, "foreground_color")
            .or_else(|| color_attribute(metadata, "idle_text_color"))
            .unwrap_or(TEXT_MUTED)
    }
}

fn border_width(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "border_width")
        .unwrap_or(1.0)
        .max(0.0)
}

fn selected_border_width(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "selected_border_width")
        .unwrap_or(0.0)
        .max(0.0)
}

fn selected_underline_height(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "selected_underline_height")
        .unwrap_or(TAB_UNDERLINE_HEIGHT)
        .max(0.0)
}

fn corner_radius(metadata: &UiTemplateNodeMetadata) -> f32 {
    number_attribute(metadata, "corner_radius")
        .or_else(|| number_attribute(metadata, "radius"))
        .unwrap_or(SEGMENT_RADIUS)
        .max(0.0)
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

fn number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata.attributes.get(key).and_then(value_as_f32)
}

fn string_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}

fn color_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(Value::as_str)
        .filter(|color| !color.trim().is_empty())
}

fn value_as_f32(value: &Value) -> Option<f32> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .map(|value| value as f32)
}

fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background: &str,
    border: Option<&str>,
    border_width: f32,
    corner_radius: f32,
    state: &SegmentedRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            background_color: Some(background.to_string()),
            border_color: border.map(str::to_string),
            border_width,
            corner_radius,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: None,
        image: None,
        opacity,
    }
}

fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: &str,
    font_size: f32,
    line_height: f32,
    state: &SegmentedRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(foreground.to_string()),
            font_size,
            line_height,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}
