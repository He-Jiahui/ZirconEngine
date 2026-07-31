use std::sync::OnceLock;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    design_tokens::{EditorDesignTokens, EditorTypographyTokens},
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiPainterStyleSelector, UiRgbaColor},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
    tree::UiTemplateNodeMetadata,
};

use super::painter_state::UiRenderPainterStateSource;

#[derive(Clone, Copy, Debug)]
struct ButtonVisual {
    primary_surface: UiRgbaColor,
    primary_hover: UiRgbaColor,
    primary_pressed: UiRgbaColor,
    primary_border: UiRgbaColor,
    primary_text: UiRgbaColor,
    secondary_surface: UiRgbaColor,
    secondary_hover: UiRgbaColor,
    secondary_pressed: UiRgbaColor,
    secondary_border: UiRgbaColor,
    secondary_text: UiRgbaColor,
    tertiary_surface: UiRgbaColor,
    tertiary_hover: UiRgbaColor,
    tertiary_pressed: UiRgbaColor,
    tertiary_text: UiRgbaColor,
    danger_surface: UiRgbaColor,
    danger_border: UiRgbaColor,
    danger_text: UiRgbaColor,
    disabled_surface: UiRgbaColor,
    disabled_border: UiRgbaColor,
    disabled_text: UiRgbaColor,
    focus_border: UiRgbaColor,
    icon_normal: UiRgbaColor,
    icon_selected_surface: UiRgbaColor,
    icon_selected: UiRgbaColor,
    icon_panel_surface: UiRgbaColor,
    icon_panel_border: UiRgbaColor,
    selected_background: Option<UiRgbaColor>,
    padding_left: f32,
    padding_right: f32,
    icon_size: f32,
    icon_button_size: f32,
    spacing: f32,
    border_width: f32,
    button_radius: f32,
    icon_button_radius: f32,
    font_size: f32,
    line_height: f32,
    min_frame_extent: f32,
}

impl ButtonVisual {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_button_visual();
        if let Some(color) = first_rgba_attribute(metadata, &["background_color"]) {
            visual.primary_surface = color;
            visual.secondary_surface = color;
            visual.tertiary_surface = color;
            visual.danger_surface = color;
            visual.icon_panel_surface = color;
        }
        if let Some(color) = first_rgba_attribute(metadata, &["hover_background_color"]) {
            visual.primary_hover = color;
            visual.secondary_hover = color;
            visual.tertiary_hover = color;
        }
        if let Some(color) = first_rgba_attribute(metadata, &["pressed_background_color"]) {
            visual.primary_pressed = color;
            visual.secondary_pressed = color;
            visual.tertiary_pressed = color;
        }
        visual.selected_background = first_rgba_attribute(metadata, &["selected_background_color"]);
        visual.disabled_surface = first_rgba_attribute(metadata, &["disabled_background_color"])
            .unwrap_or(visual.disabled_surface);

        if let Some(color) = first_rgba_attribute(metadata, &["border_color"]) {
            visual.primary_border = color;
            visual.secondary_border = color;
            visual.danger_border = color;
            visual.icon_panel_border = color;
        }
        visual.focus_border =
            first_rgba_attribute(metadata, &["focus_border_color"]).unwrap_or(visual.focus_border);
        visual.disabled_border = first_rgba_attribute(metadata, &["disabled_border_color"])
            .unwrap_or(visual.disabled_border);

        if let Some(color) = first_rgba_attribute(metadata, &["foreground_color", "text_color"]) {
            visual.primary_text = color;
            visual.secondary_text = color;
            visual.tertiary_text = color;
            visual.danger_text = color;
            visual.icon_normal = color;
        }
        visual.icon_normal =
            first_rgba_attribute(metadata, &["icon_color"]).unwrap_or(visual.icon_normal);
        visual.icon_selected =
            first_rgba_attribute(metadata, &["selected_icon_color", "icon_color"])
                .unwrap_or(visual.icon_selected);
        visual.disabled_text = first_rgba_attribute(metadata, &["disabled_foreground_color"])
            .unwrap_or(visual.disabled_text);

        visual.padding_left = metric_attribute(metadata, "layout_padding_left")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.padding_left);
        visual.padding_right = metric_attribute(metadata, "layout_padding_right")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.padding_right);
        visual.icon_size = metric_attribute(metadata, "layout_icon_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.icon_size);
        visual.icon_button_size = metric_attribute(metadata, "layout_icon_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.icon_button_size);
        visual.spacing = metric_attribute(metadata, "layout_spacing")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.spacing);
        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.border_width);
        if let Some(radius) = metric_attribute(metadata, "corner_radius")
            .or_else(|| metric_attribute(metadata, "radius"))
            .filter(|value| *value >= 0.0)
        {
            visual.button_radius = radius;
            visual.icon_button_radius = radius;
        }
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
        visual.min_frame_extent = visual.border_width.max(f32::EPSILON);
        visual
    }
}

fn default_button_visual() -> &'static ButtonVisual {
    static VISUAL: OnceLock<ButtonVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        ButtonVisual {
            primary_surface: colors.accent_soft,
            primary_hover: colors.surface_selected,
            primary_pressed: colors.surface[3],
            primary_border: colors.accent,
            primary_text: colors.text_primary,
            secondary_surface: colors.surface[1],
            secondary_hover: colors.surface_hover,
            secondary_pressed: colors.surface[3],
            secondary_border: colors.border,
            secondary_text: colors.text_primary,
            tertiary_surface: colors.surface[0],
            tertiary_hover: colors.surface_hover,
            tertiary_pressed: colors.surface[3],
            tertiary_text: colors.text_secondary,
            danger_surface: colors.error_container,
            danger_border: colors.error,
            danger_text: colors.error,
            disabled_surface: colors.surface_disabled,
            disabled_border: colors.border_disabled,
            disabled_text: colors.text_disabled,
            focus_border: colors.accent,
            icon_normal: colors.text_secondary,
            icon_selected_surface: colors.surface_selected,
            icon_selected: colors.accent,
            icon_panel_surface: colors.surface[2],
            icon_panel_border: colors.border,
            selected_background: None,
            padding_left: density.gap_large,
            padding_right: density.gap_large,
            icon_size: controls.dense_height - density.gap_large,
            icon_button_size: controls.dense_height - density.gap_large
                + controls.border_width * 2.0,
            spacing: density.gap_medium,
            border_width: controls.border_width,
            button_radius: controls.small_radius,
            icon_button_radius: controls.control_radius,
            font_size: typography.body_size,
            line_height: typography.body_size * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            min_frame_extent: controls.border_width.max(f32::EPSILON),
        }
    })
}

pub(super) fn button_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_button_component)
}

pub(super) fn button_suppresses_owner_image(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_button_component)
}

pub(super) fn button_render_commands(
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
    if !is_button_component(metadata) {
        return Vec::new();
    }

    let visual = ButtonVisual::resolve(metadata);
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return Vec::new();
    }
    let state = ButtonRenderState::resolve(metadata, state_flags, component_state);
    if is_icon_button(metadata) {
        icon_button_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        )
    } else {
        button_commands(
            node_id, metadata, &state, &visual, frame, clip_frame, z_index, opacity,
        )
    }
}

#[derive(Clone, Copy)]
struct ButtonRenderState {
    family: UiPainterFamily,
    kind: ButtonKind,
    visual_state: UiPainterResolvedState,
    surface_hot: bool,
    marked: bool,
}

impl ButtonRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let component_flags = component_state.map(|state| &state.flags);
        let selected = component_flags.is_some_and(|flags| flags.selected)
            || bool_attribute(metadata, "selected").unwrap_or(false);
        let checked = component_flags.is_some_and(|flags| flags.checked)
            || state_flags.checked
            || bool_attribute(metadata, "checked").unwrap_or(false);
        let mut painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        painter_state.checked = checked;
        painter_state.selected = selected;
        let family = if is_icon_button(metadata) {
            UiPainterFamily::IconButton
        } else {
            UiPainterFamily::Button
        };
        let kind = button_kind(metadata);
        let surface_hot = painter_state.hovered
            || painter_state.open
            || painter_state.dragging
            || painter_state.drop_hovered;
        let marked = selected || checked;
        Self {
            family,
            kind,
            visual_state: UiPainterStyleSelector::resolved_state_for_family(painter_state, family),
            surface_hot,
            marked,
        }
    }

    fn unavailable(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
        )
    }

    fn selected(self) -> bool {
        self.marked
            || matches!(
                self.visual_state,
                UiPainterResolvedState::Selected | UiPainterResolvedState::Checked
            )
    }

    fn focused(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Focused)
    }

    fn pressed(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Pressed)
    }

    fn surface_hot(self) -> bool {
        self.surface_hot
            || matches!(
                self.visual_state,
                UiPainterResolvedState::Hovered
                    | UiPainterResolvedState::Open
                    | UiPainterResolvedState::Dragging
                    | UiPainterResolvedState::DropHovered
            )
    }
}

fn is_button_component(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "Button" | "ToggleButton" | "IconButton"
    )
}

fn is_icon_button(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.component == "IconButton"
}

fn button_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &ButtonRenderState,
    visual: &ButtonVisual,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mut commands = vec![surface_command(
        node_id, frame, clip_frame, z_index, state, visual, opacity,
    )];
    let icon = icon_name(metadata);
    let label = button_label(metadata);
    let text_width = (frame.width
        - visual.padding_left
        - visual.padding_right
        - icon
            .as_ref()
            .map(|_| visual.icon_size + visual.spacing)
            .unwrap_or(0.0))
    .max(visual.min_frame_extent);
    let mut cursor_x = frame.x + visual.padding_left;
    if let Some(icon) = icon {
        let icon_frame = UiFrame::new(
            cursor_x,
            frame.y + (frame.height - visual.icon_size).max(0.0) * 0.5,
            visual.icon_size,
            visual.icon_size,
        );
        commands.push(icon_command(
            node_id,
            icon_frame,
            clip_frame,
            z_index.saturating_add(3),
            icon,
            foreground_color(state, visual),
            state,
            opacity,
        ));
        cursor_x += visual.icon_size + visual.spacing;
    }
    if let Some(label) = label {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                cursor_x,
                frame.y + (frame.height - visual.line_height).max(0.0) * 0.5,
                text_width,
                visual.line_height.min(frame.height),
            ),
            clip_frame,
            z_index.saturating_add(4),
            label,
            foreground_color(state, visual),
            visual.font_size,
            visual.line_height,
            state,
            opacity,
        ));
    }
    commands
}

fn icon_button_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &ButtonRenderState,
    visual: &ButtonVisual,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let mut commands = vec![surface_command(
        node_id, frame, clip_frame, z_index, state, visual, opacity,
    )];
    let Some(icon) = icon_name(metadata) else {
        return commands;
    };
    commands.push(icon_command(
        node_id,
        UiFrame::new(
            frame.x + (frame.width - visual.icon_button_size).max(0.0) * 0.5,
            frame.y + (frame.height - visual.icon_button_size).max(0.0) * 0.5,
            visual.icon_button_size,
            visual.icon_button_size,
        ),
        clip_frame,
        z_index.saturating_add(3),
        icon,
        icon_button_foreground(state, visual),
        state,
        opacity,
    ));
    commands
}

fn surface_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    state: &ButtonRenderState,
    visual: &ButtonVisual,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index: z_index.saturating_add(1),
        style: UiResolvedStyle {
            background_color: Some(css_color(background_color(state, visual))),
            border_color: Some(css_color(border_color(state, visual))),
            border_width: visual.border_width,
            corner_radius: if state.family == UiPainterFamily::IconButton {
                visual.icon_button_radius
            } else {
                visual.button_radius
            },
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
    state: &ButtonRenderState,
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
    icon: String,
    foreground: UiRgbaColor,
    state: &ButtonRenderState,
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
        image: Some(UiVisualAssetRef::Icon(icon)),
        opacity,
    }
}

fn background_color(state: &ButtonRenderState, visual: &ButtonVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_surface
    } else if state.family == UiPainterFamily::IconButton {
        if state.selected() {
            visual
                .selected_background
                .unwrap_or(visual.icon_selected_surface)
        } else if state.pressed() {
            visual.secondary_pressed
        } else if state.surface_hot() {
            visual.secondary_hover
        } else {
            visual.icon_panel_surface
        }
    } else if state.pressed() {
        surface_for_kind(state.kind, visual, ButtonSurfaceState::Pressed)
    } else if state.selected() {
        visual
            .selected_background
            .unwrap_or_else(|| surface_for_kind(state.kind, visual, ButtonSurfaceState::Hover))
    } else if state.surface_hot() {
        surface_for_kind(state.kind, visual, ButtonSurfaceState::Hover)
    } else {
        surface_for_kind(state.kind, visual, ButtonSurfaceState::Normal)
    }
}

#[derive(Clone, Copy)]
enum ButtonSurfaceState {
    Normal,
    Hover,
    Pressed,
}

fn surface_for_kind(
    kind: ButtonKind,
    visual: &ButtonVisual,
    state: ButtonSurfaceState,
) -> UiRgbaColor {
    match (kind, state) {
        (ButtonKind::Primary, ButtonSurfaceState::Normal) => visual.primary_surface,
        (ButtonKind::Primary, ButtonSurfaceState::Hover) => visual.primary_hover,
        (ButtonKind::Primary, ButtonSurfaceState::Pressed) => visual.primary_pressed,
        (ButtonKind::Secondary, ButtonSurfaceState::Normal) => visual.secondary_surface,
        (ButtonKind::Secondary, ButtonSurfaceState::Hover) => visual.secondary_hover,
        (ButtonKind::Secondary, ButtonSurfaceState::Pressed) => visual.secondary_pressed,
        (ButtonKind::Tertiary, ButtonSurfaceState::Normal) => visual.tertiary_surface,
        (ButtonKind::Tertiary, ButtonSurfaceState::Hover) => visual.tertiary_hover,
        (ButtonKind::Tertiary, ButtonSurfaceState::Pressed) => visual.tertiary_pressed,
        (ButtonKind::Danger, _) => visual.danger_surface,
    }
}

fn border_color(state: &ButtonRenderState, visual: &ButtonVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_border
    } else if state.focused() || state.pressed() || state.selected() {
        visual.focus_border
    } else if state.family == UiPainterFamily::IconButton {
        visual.icon_panel_border
    } else {
        match state.kind {
            ButtonKind::Primary => visual.primary_border,
            ButtonKind::Danger => visual.danger_border,
            ButtonKind::Secondary | ButtonKind::Tertiary => visual.secondary_border,
        }
    }
}

fn foreground_color(state: &ButtonRenderState, visual: &ButtonVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_text
    } else {
        match state.kind {
            ButtonKind::Primary => visual.primary_text,
            ButtonKind::Danger => visual.danger_text,
            ButtonKind::Tertiary => visual.tertiary_text,
            ButtonKind::Secondary => visual.secondary_text,
        }
    }
}

fn icon_button_foreground(state: &ButtonRenderState, visual: &ButtonVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.disabled_text
    } else if state.selected() || state.pressed() {
        visual.icon_selected
    } else {
        visual.icon_normal
    }
}

#[derive(Clone, Copy)]
enum ButtonKind {
    Primary,
    Secondary,
    Tertiary,
    Danger,
}

fn button_kind(metadata: &UiTemplateNodeMetadata) -> ButtonKind {
    let values = [
        string_attribute(metadata, "button_color"),
        string_attribute(metadata, "button_variant"),
        string_attribute(metadata, "validation_level"),
    ];
    if values
        .iter()
        .flatten()
        .any(|value| contains_ascii_case(value, "danger") || contains_ascii_case(value, "error"))
    {
        ButtonKind::Danger
    } else if values
        .iter()
        .flatten()
        .any(|value| contains_ascii_case(value, "primary"))
    {
        ButtonKind::Primary
    } else if values
        .iter()
        .flatten()
        .any(|value| contains_ascii_case(value, "tertiary") || contains_ascii_case(value, "text"))
    {
        ButtonKind::Tertiary
    } else {
        ButtonKind::Secondary
    }
}

fn contains_ascii_case(value: &str, needle: &str) -> bool {
    value
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}

fn button_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["label", "text", "value_text"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_string)
}

fn icon_name(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["icon", "image", "source"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|icon| !icon.is_empty())
        .map(|icon| {
            icon.rsplit(['/', '\\'])
                .next()
                .unwrap_or(icon)
                .trim_end_matches(".svg")
                .to_string()
        })
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
