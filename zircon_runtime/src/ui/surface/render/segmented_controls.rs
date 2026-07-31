use std::sync::OnceLock;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    design_tokens::{EditorDesignTokens, EditorTypographyTokens},
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiRgbaColor},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle},
    tree::UiTemplateNodeMetadata,
};

use super::painter_state::UiRenderPainterStateSource;

#[derive(Clone, Copy, Debug)]
struct SegmentedVisual {
    background: UiRgbaColor,
    border: UiRgbaColor,
    selected_surface: UiRgbaColor,
    focus_border: UiRgbaColor,
    selected_border: UiRgbaColor,
    selected_underline: UiRgbaColor,
    hover: UiRgbaColor,
    pressed: UiRgbaColor,
    disabled_surface: UiRgbaColor,
    disabled_border: UiRgbaColor,
    text: UiRgbaColor,
    text_muted: UiRgbaColor,
    text_disabled: UiRgbaColor,
    group_label: UiRgbaColor,
    font_size: f32,
    line_height: f32,
    group_label_font_size: f32,
    group_label_line_height: f32,
    group_label_height: f32,
    group_label_gap: f32,
    segment_text_inset_x: f32,
    segment_text_inset_y: f32,
    selected_inset: f32,
    corner_radius: f32,
    tab_font_size: f32,
    tab_line_height: f32,
    tab_text_inset_x: f32,
    tab_underline_height: f32,
    border_width: f32,
    selected_border_width: f32,
    min_frame_extent: f32,
}

impl SegmentedVisual {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_segmented_visual();
        visual.background =
            first_rgba_attribute(metadata, &["background_color"]).unwrap_or(visual.background);
        visual.border = first_rgba_attribute(metadata, &["border_color"]).unwrap_or(visual.border);
        visual.selected_surface = first_rgba_attribute(metadata, &["selected_background_color"])
            .unwrap_or(visual.selected_surface);
        visual.focus_border =
            first_rgba_attribute(metadata, &["focus_border_color"]).unwrap_or(visual.focus_border);
        visual.selected_border = first_rgba_attribute(metadata, &["selected_border_color"])
            .unwrap_or(visual.selected_border);
        visual.selected_underline =
            first_rgba_attribute(metadata, &["selected_underline_color", "accent_color"])
                .unwrap_or(visual.selected_underline);
        visual.hover =
            first_rgba_attribute(metadata, &["hover_background_color"]).unwrap_or(visual.hover);
        visual.pressed =
            first_rgba_attribute(metadata, &["pressed_background_color"]).unwrap_or(visual.pressed);
        visual.disabled_surface = first_rgba_attribute(metadata, &["disabled_background_color"])
            .unwrap_or(visual.disabled_surface);
        visual.disabled_border = first_rgba_attribute(metadata, &["disabled_border_color"])
            .unwrap_or(visual.disabled_border);
        visual.text = first_rgba_attribute(
            metadata,
            &["selected_foreground_color", "selected_text_color"],
        )
        .unwrap_or(visual.text);
        visual.text_muted =
            first_rgba_attribute(metadata, &["foreground_color", "idle_text_color"])
                .unwrap_or(visual.text_muted);
        visual.text_disabled = first_rgba_attribute(metadata, &["disabled_foreground_color"])
            .unwrap_or(visual.text_disabled);
        visual.group_label =
            first_rgba_attribute(metadata, &["label_color"]).unwrap_or(visual.group_label);
        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.border_width);
        visual.selected_border_width = metric_attribute(metadata, "selected_border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.selected_border_width);
        visual.corner_radius = metric_attribute(metadata, "corner_radius")
            .or_else(|| metric_attribute(metadata, "radius"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.corner_radius);
        visual.font_size = metric_attribute(metadata, "font_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.font_size);
        visual.line_height = line_height(
            metadata,
            "line_height",
            "line_height_ratio",
            visual.font_size,
            visual.line_height,
        );
        visual.tab_font_size = metric_attribute(metadata, "tab_font_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.tab_font_size);
        visual.tab_line_height = line_height(
            metadata,
            "tab_line_height",
            "tab_line_height_ratio",
            visual.tab_font_size,
            visual.tab_line_height,
        );
        visual.group_label_height = metric_attribute(metadata, "group_label_height")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.group_label_height);
        visual.group_label_gap = metric_attribute(metadata, "group_label_gap")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.group_label_gap);
        visual.segment_text_inset_x = metric_attribute(metadata, "segment_text_inset_x")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.segment_text_inset_x);
        visual.segment_text_inset_y = metric_attribute(metadata, "segment_text_inset_y")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.segment_text_inset_y);
        visual.selected_inset = metric_attribute(metadata, "selected_inset")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.selected_inset);
        visual.tab_text_inset_x = metric_attribute(metadata, "tab_text_inset_x")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.tab_text_inset_x);
        visual.tab_underline_height = metric_attribute(metadata, "selected_underline_height")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.tab_underline_height);
        visual.min_frame_extent = visual.border_width.max(f32::EPSILON);
        visual
    }
}

fn default_segmented_visual() -> &'static SegmentedVisual {
    static VISUAL: OnceLock<SegmentedVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        SegmentedVisual {
            background: colors.surface[2],
            border: colors.border,
            selected_surface: colors.surface_selected,
            focus_border: colors.accent,
            selected_border: colors.accent,
            selected_underline: colors.accent,
            hover: colors.surface_hover,
            pressed: colors.surface[3],
            disabled_surface: colors.surface_disabled,
            disabled_border: colors.border_disabled,
            text: colors.text_primary,
            text_muted: colors.text_secondary,
            text_disabled: colors.text_disabled,
            group_label: colors.text_secondary,
            font_size: typography.body_size,
            line_height: typography.body_size * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            group_label_font_size: typography.caption_size,
            group_label_line_height: typography.caption_size
                * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            group_label_height: typography.overlay_size + controls.border_width * 2.0,
            group_label_gap: density.gap_small,
            segment_text_inset_x: density.gap_medium,
            segment_text_inset_y: density.gap_small + controls.border_width,
            selected_inset: controls.border_width * 2.0,
            corner_radius: controls.control_radius,
            tab_font_size: typography.overlay_size,
            tab_line_height: typography.overlay_size
                * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            tab_text_inset_x: density.gap_large,
            tab_underline_height: controls.border_width * 2.0,
            border_width: controls.border_width,
            selected_border_width: 0.0,
            min_frame_extent: controls.border_width.max(f32::EPSILON),
        }
    })
}

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
    let Some(kind) = control_kind(metadata) else {
        return Vec::new();
    };
    let visual = SegmentedVisual::resolve(metadata);
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return Vec::new();
    }
    let state = SegmentedRenderState::resolve(metadata, state_flags, component_state);
    match kind {
        SegmentedControlKind::SegmentedControl => segmented_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        ),
        SegmentedControlKind::Tab => tab_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        ),
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
    surface_hot: bool,
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
            surface_hot: painter_state.hovered
                || painter_state.open
                || painter_state.dragging
                || painter_state.drop_hovered,
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
    fn focused(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Focused)
    }
    fn surface_hot(self) -> bool {
        self.surface_hot
    }
}

#[allow(clippy::too_many_arguments)]
fn segmented_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
    frame: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let options = segmented_options(metadata);
    if options.is_empty() {
        return Vec::new();
    }
    let mut commands = Vec::new();
    let label = group_label(metadata);
    let has_label = label.is_some();
    if let Some(label) = label {
        commands.push(text_command(
            node_id,
            UiFrame::new(frame.x, frame.y, frame.width, visual.group_label_height),
            clip,
            z.saturating_add(3),
            label,
            group_label_color(state, visual),
            visual.group_label_font_size,
            visual.group_label_line_height,
            state,
            opacity,
        ));
    }
    let body = segmented_body_frame(metadata, frame, has_label, visual);
    commands.push(quad_command(
        node_id,
        body,
        clip,
        z.saturating_add(1),
        segmented_background(state, visual),
        Some(segmented_border(state, visual)),
        visual.border_width,
        visual.corner_radius,
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
                    segment.y + visual.segment_text_inset_y - visual.border_width,
                    visual.border_width,
                    (segment.height - (visual.segment_text_inset_y - visual.border_width) * 2.0)
                        .max(visual.min_frame_extent),
                ),
                clip,
                z.saturating_add(2),
                divider_color(state, visual),
                None,
                0.0,
                0.0,
                state,
                opacity,
            ));
        }
        let option_selected = option_is_selected(option, selected);
        if option_selected {
            push_selected_segment(
                &mut commands,
                node_id,
                state,
                visual,
                segment,
                clip,
                z.saturating_add(3),
                opacity,
            );
        }
        commands.push(text_command(
            node_id,
            UiFrame::new(
                segment.x + visual.segment_text_inset_x,
                segment.y + visual.segment_text_inset_y,
                (segment.width - visual.segment_text_inset_x * 2.0).max(visual.min_frame_extent),
                (segment.height - visual.segment_text_inset_y * 2.0).max(visual.line_height),
            ),
            clip,
            z.saturating_add(5),
            option_label(option),
            option_text_color(state, visual, option_selected),
            visual.font_size,
            visual.line_height,
            state,
            opacity,
        ));
    }
    commands
}

#[allow(clippy::too_many_arguments)]
fn tab_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
    frame: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mut commands = Vec::new();
    if let Some(background) = tab_background(metadata, state, visual) {
        commands.push(quad_command(
            node_id,
            frame,
            clip,
            z.saturating_add(1),
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
                frame.y + (frame.height - visual.tab_underline_height).max(0.0),
                frame.width,
                visual.tab_underline_height,
            ),
            clip,
            z.saturating_add(3),
            selected_underline(state, visual),
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
                frame.x + visual.tab_text_inset_x,
                frame.y + (frame.height - visual.tab_line_height).max(0.0) * 0.5,
                (frame.width - visual.tab_text_inset_x * 2.0).max(visual.min_frame_extent),
                visual.tab_line_height,
            ),
            clip,
            z.saturating_add(4),
            label,
            tab_text_color(state, visual),
            visual.tab_font_size,
            visual.tab_line_height,
            state,
            opacity,
        ));
    }
    commands
}

fn push_selected_segment(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
    segment: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    opacity: f32,
) {
    let selected = inset_frame(segment, visual.selected_inset, visual.min_frame_extent);
    commands.push(quad_command(
        node_id,
        selected,
        clip,
        z,
        selected_surface(state, visual),
        (visual.selected_border_width > 0.0).then_some(visual.selected_border),
        visual.selected_border_width,
        (visual.corner_radius - visual.border_width).max(0.0),
        state,
        opacity,
    ));
    if visual.tab_underline_height > 0.0 {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                selected.x,
                selected.y + (selected.height - visual.tab_underline_height).max(0.0),
                selected.width,
                visual
                    .tab_underline_height
                    .min(selected.height)
                    .max(visual.min_frame_extent),
            ),
            clip,
            z.saturating_add(1),
            selected_underline(state, visual),
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
fn segmented_body_frame(
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    has_label: bool,
    visual: &SegmentedVisual,
) -> UiFrame {
    let label_block = if has_label {
        visual.group_label_height + visual.group_label_gap
    } else {
        0.0
    };
    UiFrame::new(
        frame.x + metric_attribute(metadata, "layout_offset_x").unwrap_or(0.0),
        frame.y + label_block + metric_attribute(metadata, "layout_offset_y").unwrap_or(0.0),
        frame.width,
        (frame.height - label_block).max(visual.min_frame_extent),
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
        .max(f32::EPSILON),
        frame.height,
    )
}
fn inset_frame(frame: UiFrame, inset: f32, min: f32) -> UiFrame {
    UiFrame::new(
        frame.x + inset,
        frame.y + inset,
        (frame.width - inset * 2.0).max(min),
        (frame.height - inset * 2.0).max(min),
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
fn selected_segment_value(metadata: &UiTemplateNodeMetadata) -> Option<&str> {
    ["value", "value_text", "selected", "text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|value| !value.is_empty())
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
fn segmented_background(state: &SegmentedRenderState, visual: &SegmentedVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_surface
    } else if state.pressed() {
        visual.pressed
    } else if state.surface_hot() {
        visual.hover
    } else {
        visual.background
    }
}
fn segmented_border(state: &SegmentedRenderState, visual: &SegmentedVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_border
    } else if state.pressed() || state.focused() || state.surface_hot() {
        visual.focus_border
    } else {
        visual.border
    }
}
fn divider_color(state: &SegmentedRenderState, visual: &SegmentedVisual) -> UiRgbaColor {
    segmented_border(state, visual)
}
fn selected_surface(state: &SegmentedRenderState, visual: &SegmentedVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_surface
    } else {
        visual.selected_surface
    }
}
fn selected_underline(state: &SegmentedRenderState, visual: &SegmentedVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else {
        visual.selected_underline
    }
}
fn option_text_color(
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
    selected: bool,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else if selected {
        visual.text
    } else {
        visual.text_muted
    }
}
fn group_label_color(state: &SegmentedRenderState, visual: &SegmentedVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else {
        visual.group_label
    }
}
fn tab_background(
    metadata: &UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
) -> Option<UiRgbaColor> {
    if state.unavailable() {
        Some(visual.disabled_surface)
    } else if state.pressed() {
        Some(visual.pressed)
    } else if state.surface_hot() {
        Some(visual.hover)
    } else {
        first_rgba_attribute(metadata, &["background_color"])
    }
}
fn tab_text_color(state: &SegmentedRenderState, visual: &SegmentedVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else if state.active {
        visual.text
    } else {
        visual.text_muted
    }
}
fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}
fn metric_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(value_as_f32)
}
fn string_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}
fn first_rgba_attribute(metadata: &UiTemplateNodeMetadata, keys: &[&str]) -> Option<UiRgbaColor> {
    keys.iter().find_map(|key| {
        metadata
            .style_overrides
            .get(*key)
            .or_else(|| metadata.attributes.get(*key))
            .and_then(Value::as_str)
            .and_then(parse_css_color)
    })
}
fn value_as_f32(value: &Value) -> Option<f32> {
    let value = match value {
        Value::Integer(value) => *value as f64,
        Value::Float(value) if value.is_finite() => *value,
        _ => return None,
    } as f32;
    value.is_finite().then_some(value)
}
fn line_height(
    metadata: &UiTemplateNodeMetadata,
    absolute_key: &str,
    ratio_key: &str,
    font_size: f32,
    default: f32,
) -> f32 {
    metric_attribute(metadata, absolute_key)
        .filter(|value| *value > 0.0)
        .or_else(|| {
            metric_attribute(metadata, ratio_key)
                .filter(|value| *value > 0.0)
                .map(|ratio| font_size * ratio)
        })
        .unwrap_or(default)
}
fn parse_css_color(value: &str) -> Option<UiRgbaColor> {
    let encoded = value.trim().strip_prefix('#')?;
    if !encoded.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let (r, g, b, a) = match encoded.len() {
        6 => (
            u8::from_str_radix(&encoded[0..2], 16).ok()?,
            u8::from_str_radix(&encoded[2..4], 16).ok()?,
            u8::from_str_radix(&encoded[4..6], 16).ok()?,
            u8::MAX,
        ),
        8 => (
            u8::from_str_radix(&encoded[0..2], 16).ok()?,
            u8::from_str_radix(&encoded[2..4], 16).ok()?,
            u8::from_str_radix(&encoded[4..6], 16).ok()?,
            u8::from_str_radix(&encoded[6..8], 16).ok()?,
        ),
        _ => return None,
    };
    Some(UiRgbaColor::from_u8(r, g, b, a))
}
fn css_color(color: UiRgbaColor) -> String {
    let [r, g, b, a] = color.to_u8();
    let mut value = if a == u8::MAX {
        format!("{r:02x}{g:02x}{b:02x}")
    } else {
        format!("{r:02x}{g:02x}{b:02x}{a:02x}")
    };
    value.insert(0, '#');
    value
}
#[allow(clippy::too_many_arguments)]
fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background: UiRgbaColor,
    border: Option<UiRgbaColor>,
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
            background_color: Some(css_color(background)),
            border_color: border.map(css_color),
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
#[allow(clippy::too_many_arguments)]
fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: UiRgbaColor,
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
            foreground_color: Some(css_color(foreground)),
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
