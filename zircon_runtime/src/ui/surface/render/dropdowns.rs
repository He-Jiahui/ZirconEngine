use std::sync::OnceLock;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::{UiComponentState, UiValue},
    design_tokens::{EditorDesignTokens, EditorTypographyTokens},
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiRgbaColor},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
    tree::UiTemplateNodeMetadata,
};

use super::painter_state::UiRenderPainterStateSource;

#[derive(Clone, Copy, Debug)]
struct DropdownVisual {
    surface_idle: UiRgbaColor,
    surface_hover: UiRgbaColor,
    surface_pressed: UiRgbaColor,
    surface_open: UiRgbaColor,
    surface_disabled: UiRgbaColor,
    border_idle: UiRgbaColor,
    border_focus: UiRgbaColor,
    border_disabled: UiRgbaColor,
    text: UiRgbaColor,
    label_text: UiRgbaColor,
    icon: UiRgbaColor,
    text_disabled: UiRgbaColor,
    horizontal_inset: f32,
    caret_size: f32,
    caret_right_inset: f32,
    caret_gap: f32,
    open_mark_width: f32,
    open_mark_inset: f32,
    border_width: f32,
    corner_radius: f32,
    label_top: f32,
    label_font_size: f32,
    label_line_height: f32,
    value_bottom_inset: f32,
    value_font_size: f32,
    value_line_height: f32,
    two_line_min_height: f32,
    min_frame_extent: f32,
}

impl DropdownVisual {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_dropdown_visual();
        visual.surface_idle =
            first_rgba_attribute(metadata, &["background_color"]).unwrap_or(visual.surface_idle);
        visual.surface_hover = first_rgba_attribute(metadata, &["hover_background_color"])
            .unwrap_or(visual.surface_hover);
        visual.surface_pressed = first_rgba_attribute(metadata, &["pressed_background_color"])
            .unwrap_or(visual.surface_pressed);
        visual.surface_open = first_rgba_attribute(metadata, &["open_background_color"])
            .unwrap_or(visual.surface_open);
        visual.surface_disabled = first_rgba_attribute(metadata, &["disabled_background_color"])
            .unwrap_or(visual.surface_disabled);
        visual.border_idle =
            first_rgba_attribute(metadata, &["border_color"]).unwrap_or(visual.border_idle);
        visual.border_focus =
            first_rgba_attribute(metadata, &["focus_border_color"]).unwrap_or(visual.border_focus);
        visual.border_disabled = first_rgba_attribute(metadata, &["disabled_border_color"])
            .unwrap_or(visual.border_disabled);
        visual.text = first_rgba_attribute(metadata, &["foreground_color", "text_color"])
            .unwrap_or(visual.text);
        visual.label_text =
            first_rgba_attribute(metadata, &["label_color"]).unwrap_or(visual.label_text);
        visual.icon = first_rgba_attribute(metadata, &["icon_color"]).unwrap_or(visual.icon);
        visual.text_disabled = first_rgba_attribute(metadata, &["disabled_foreground_color"])
            .unwrap_or(visual.text_disabled);
        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.border_width);
        visual.corner_radius = metric_attribute(metadata, "corner_radius")
            .or_else(|| metric_attribute(metadata, "radius"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.corner_radius);
        visual.horizontal_inset = metric_attribute(metadata, "horizontal_inset")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.horizontal_inset);
        visual.caret_size = metric_attribute(metadata, "caret_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.caret_size);
        visual.caret_right_inset = metric_attribute(metadata, "caret_right_inset")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.caret_right_inset);
        visual.caret_gap = metric_attribute(metadata, "caret_gap")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.caret_gap);
        visual.open_mark_width = metric_attribute(metadata, "open_mark_width")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.open_mark_width);
        visual.open_mark_inset = metric_attribute(metadata, "open_mark_inset")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.open_mark_inset);
        visual.label_top = metric_attribute(metadata, "label_top")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.label_top);
        visual.value_bottom_inset = metric_attribute(metadata, "value_bottom_inset")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.value_bottom_inset);
        visual.label_font_size = metric_attribute(metadata, "label_font_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.label_font_size);
        visual.value_font_size = metric_attribute(metadata, "font_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.value_font_size);
        visual.label_line_height = line_height(
            metadata,
            "label_line_height",
            "label_line_height_ratio",
            visual.label_font_size,
            visual.label_line_height,
        );
        visual.value_line_height = line_height(
            metadata,
            "line_height",
            "line_height_ratio",
            visual.value_font_size,
            visual.value_line_height,
        );
        visual.min_frame_extent = visual.border_width.max(f32::EPSILON);
        visual
    }
}

fn default_dropdown_visual() -> &'static DropdownVisual {
    static VISUAL: OnceLock<DropdownVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        DropdownVisual {
            surface_idle: colors.surface_recessed,
            surface_hover: colors.surface_hover,
            surface_pressed: colors.surface[3],
            surface_open: colors.accent_soft,
            surface_disabled: colors.surface_disabled,
            border_idle: colors.border,
            border_focus: colors.accent,
            border_disabled: colors.border_disabled,
            text: colors.text_primary,
            label_text: colors.text_secondary,
            icon: colors.text_secondary,
            text_disabled: colors.text_disabled,
            horizontal_inset: density.gap_medium,
            caret_size: typography.overlay_size,
            caret_right_inset: density.gap_large,
            caret_gap: density.gap_small,
            open_mark_width: controls.border_width * 2.0,
            open_mark_inset: density.gap_small + controls.border_width,
            border_width: controls.border_width,
            corner_radius: controls.small_radius,
            label_top: controls.border_width,
            label_font_size: typography.caption_size,
            label_line_height: typography.caption_size
                * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            value_bottom_inset: controls.border_width,
            value_font_size: typography.overlay_size,
            value_line_height: typography.overlay_size
                * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            two_line_min_height: controls.compact_height,
            min_frame_extent: controls.border_width.max(f32::EPSILON),
        }
    })
}

pub(super) fn dropdown_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_dropdown)
}

pub(super) fn dropdown_render_commands(
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
    if !is_dropdown(metadata) {
        return Vec::new();
    }

    let visual = DropdownVisual::resolve(metadata);
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return Vec::new();
    }
    let state = DropdownRenderState::resolve(metadata, state_flags, component_state);
    let mut commands = Vec::new();
    commands.push(quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        surface_color(&state, &visual),
        Some(border_color(&state, &visual)),
        visual.border_width,
        visual.corner_radius,
        &state,
        opacity,
    ));

    let caret = caret_rect(frame, &visual);
    let text_right = (caret.x - visual.caret_gap)
        .max(frame.x + visual.horizontal_inset + visual.min_frame_extent);
    let label = dropdown_label(metadata);
    let has_label = label.is_some();
    if let Some(label) = label {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                frame.x + visual.horizontal_inset,
                frame.y + visual.label_top,
                (text_right - frame.x - visual.horizontal_inset).max(visual.min_frame_extent),
                visual.label_line_height.min(frame.height),
            ),
            clip_frame,
            z_index.saturating_add(3),
            label,
            label_color(&state, &visual),
            visual.label_font_size,
            visual.label_line_height,
            &state,
            opacity,
        ));
    }
    if let Some(value) = selected_value_text(metadata) {
        commands.push(text_command(
            node_id,
            value_rect(frame, text_right, has_label, &visual),
            clip_frame,
            z_index.saturating_add(3),
            value,
            text_color(&state, &visual),
            visual.value_font_size,
            visual.value_line_height,
            &state,
            opacity,
        ));
    }

    commands.push(icon_command(
        node_id,
        caret,
        clip_frame,
        z_index.saturating_add(4),
        if state.open() {
            "chevron-up"
        } else {
            "chevron-down"
        },
        icon_color(metadata, &state, &visual),
        &state,
        opacity,
    ));
    if state.open() {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                (frame.right() - visual.open_mark_width - visual.border_width).max(frame.x),
                frame.y + visual.open_mark_inset,
                visual.open_mark_width,
                (frame.height - visual.open_mark_inset * 2.0).max(visual.min_frame_extent),
            ),
            clip_frame,
            z_index.saturating_add(5),
            border_color(&state, &visual),
            None,
            0.0,
            visual.border_width,
            &state,
            opacity,
        ));
    }
    commands
}

#[derive(Clone, Copy)]
struct DropdownRenderState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
    surface_hot: bool,
}

impl DropdownRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        let family = UiPainterFamily::Dropdown;
        let surface_hot =
            painter_state.hovered || painter_state.dragging || painter_state.drop_hovered;
        Self {
            family,
            visual_state: painter_state.resolved_state_for_family(family),
            surface_hot,
        }
    }

    fn unavailable(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
        )
    }

    fn open(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Open)
    }

    fn pressed(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Pressed)
    }

    fn focused(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Focused)
    }

    fn surface_hot(self) -> bool {
        self.surface_hot
            || matches!(
                self.visual_state,
                UiPainterResolvedState::Hovered
                    | UiPainterResolvedState::Dragging
                    | UiPainterResolvedState::DropHovered
            )
    }

    fn active_border(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Hovered
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
        )
    }
}

fn is_dropdown(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "ComboBox" | "Dropdown" | "Select"
    )
}

fn value_rect(
    frame: UiFrame,
    text_right: f32,
    has_label: bool,
    visual: &DropdownVisual,
) -> UiFrame {
    if has_label && frame.height >= visual.two_line_min_height {
        UiFrame::new(
            frame.x + visual.horizontal_inset,
            (frame.y + frame.height - visual.value_line_height - visual.value_bottom_inset).round(),
            (text_right - frame.x - visual.horizontal_inset).max(visual.min_frame_extent),
            visual.value_line_height.min(frame.height),
        )
    } else {
        UiFrame::new(
            frame.x + visual.horizontal_inset,
            (frame.y + (frame.height - visual.value_line_height).max(0.0) * 0.5).round(),
            (text_right - frame.x - visual.horizontal_inset).max(visual.min_frame_extent),
            visual.value_line_height.min(frame.height),
        )
    }
}

fn caret_rect(frame: UiFrame, visual: &DropdownVisual) -> UiFrame {
    UiFrame::new(
        frame.x + frame.width - visual.caret_right_inset - visual.caret_size,
        frame.y + (frame.height - visual.caret_size).max(0.0) * 0.5,
        visual.caret_size,
        visual.caret_size,
    )
}

fn selected_value_text(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    string_attribute(metadata, "value_text")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| option_label_for_value(metadata))
        .or_else(|| {
            metadata
                .attributes
                .get("value")
                .map(UiValue::from_toml)
                .map(|value| value.display_text())
                .filter(|value| !value.is_empty())
        })
        .or_else(|| {
            string_attribute(metadata, "placeholder")
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
}

fn option_label_for_value(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    let selected = metadata.attributes.get("value")?;
    let selected = UiValue::from_toml(selected).display_text();
    if selected.is_empty() {
        return None;
    }
    metadata
        .attributes
        .get("options")
        .and_then(Value::as_array)
        .and_then(|options| {
            options.iter().find_map(|option| {
                let (id, label) = option_id_and_label(option)?;
                (id == selected || label == selected).then_some(label)
            })
        })
}

fn option_id_and_label(value: &Value) -> Option<(String, String)> {
    match value {
        Value::String(raw) => {
            let mut parts = raw.splitn(3, '|');
            let id = parts.next().unwrap_or_default().trim().to_string();
            let flags = parts.next().unwrap_or_default();
            let label = flag_value(flags, "label")
                .or_else(|| flag_value(flags, "text"))
                .unwrap_or_else(|| id.clone());
            Some((id, label))
        }
        Value::Table(table) => {
            let id = table
                .get("id")
                .or_else(|| table.get("value"))
                .or_else(|| table.get("label"))
                .or_else(|| table.get("text"))
                .and_then(Value::as_str)?
                .trim()
                .to_string();
            let label = table
                .get("label")
                .or_else(|| table.get("text"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|label| !label.is_empty())
                .unwrap_or(id.as_str())
                .to_string();
            Some((id, label))
        }
        _ => None,
    }
}

fn flag_value(flags: &str, expected_key: &str) -> Option<String> {
    flags.split(',').find_map(|flag| {
        let (key, value) = flag.split_once('=')?;
        key.trim()
            .eq_ignore_ascii_case(expected_key)
            .then(|| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn dropdown_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["label", "label_text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_string)
}

fn surface_color(state: &DropdownRenderState, visual: &DropdownVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.surface_disabled
    } else if state.open() {
        visual.surface_open
    } else if state.pressed() {
        visual.surface_pressed
    } else if state.surface_hot() {
        visual.surface_hover
    } else {
        visual.surface_idle
    }
}

fn border_color(state: &DropdownRenderState, visual: &DropdownVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.border_disabled
    } else if state.open() || state.pressed() || state.focused() || state.active_border() {
        visual.border_focus
    } else {
        visual.border_idle
    }
}

fn label_color(state: &DropdownRenderState, visual: &DropdownVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else {
        visual.label_text
    }
}

fn text_color(state: &DropdownRenderState, visual: &DropdownVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else {
        visual.text
    }
}

fn icon_color(
    metadata: &UiTemplateNodeMetadata,
    state: &DropdownRenderState,
    visual: &DropdownVisual,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else {
        first_rgba_attribute(metadata, &["icon_color", "foreground_color"]).unwrap_or(visual.icon)
    }
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
    let (red, green, blue, alpha) = match encoded.len() {
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
    Some(UiRgbaColor::from_u8(red, green, blue, alpha))
}

fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background: UiRgbaColor,
    border: Option<UiRgbaColor>,
    border_width: f32,
    corner_radius: f32,
    state: &DropdownRenderState,
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

fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: UiRgbaColor,
    font_size: f32,
    line_height: f32,
    state: &DropdownRenderState,
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

fn icon_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    icon: &str,
    foreground: UiRgbaColor,
    state: &DropdownRenderState,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Image,
        frame,
        clip_frame,
        z_index,
        style: UiResolvedStyle {
            foreground_color: Some(css_color(foreground)),
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon(icon.to_string())),
        opacity,
    }
}

fn css_color(color: UiRgbaColor) -> String {
    let [red, green, blue, alpha] = color.to_u8();
    let mut value = if alpha == u8::MAX {
        format!("{red:02x}{green:02x}{blue:02x}")
    } else {
        format!("{red:02x}{green:02x}{blue:02x}{alpha:02x}")
    };
    value.insert(0, '#');
    value
}
