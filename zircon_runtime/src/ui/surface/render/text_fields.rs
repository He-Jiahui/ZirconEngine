use std::sync::OnceLock;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    design_tokens::{EditorDesignTokens, EditorTypographyTokens},
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiRgbaColor},
    surface::{
        UiEditableTextState, UiRenderCommand, UiRenderCommandKind, UiResolvedStyle,
        UiResolvedTextLayout, UiRichTextFormat, UiTextDirection,
    },
    tree::UiTemplateNodeMetadata,
    widget::UiWidgetBehavior,
};

use super::extract::resolve_text_layout_with_cache;
use super::painter_state::UiRenderPainterStateSource;
use crate::ui::secure_text_policy::secure_text_policy;
use crate::ui::text::{
    UiPreeditSpan, UiSecureTextPresentation, UiTextLayoutRequest, UiTextMeasureCache,
    apply_secure_text_presentation,
};

#[derive(Clone, Copy, Debug)]
struct TextFieldVisual {
    surface_idle: UiRgbaColor,
    surface_hover: UiRgbaColor,
    surface_pressed: UiRgbaColor,
    surface_focused: UiRgbaColor,
    surface_disabled: UiRgbaColor,
    border_idle: UiRgbaColor,
    border_hover: UiRgbaColor,
    border_focus: UiRgbaColor,
    border_disabled: UiRgbaColor,
    text: UiRgbaColor,
    placeholder_text: UiRgbaColor,
    text_disabled: UiRgbaColor,
    padding_left: f32,
    padding_right: f32,
    padding_top: f32,
    padding_bottom: f32,
    border_width: f32,
    corner_radius: f32,
    font_size: f32,
    line_height: f32,
    min_frame_extent: f32,
}

impl TextFieldVisual {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_text_field_visual();
        visual.surface_idle =
            first_rgba_attribute(metadata, &["background_color"]).unwrap_or(visual.surface_idle);
        visual.surface_hover = first_rgba_attribute(metadata, &["hover_background_color"])
            .unwrap_or(visual.surface_hover);
        visual.surface_pressed = first_rgba_attribute(metadata, &["pressed_background_color"])
            .unwrap_or(visual.surface_pressed);
        visual.surface_focused = first_rgba_attribute(
            metadata,
            &[
                "focused_background_color",
                "focus_background_color",
                "background_color",
            ],
        )
        .unwrap_or(visual.surface_focused);
        visual.surface_disabled = first_rgba_attribute(metadata, &["disabled_background_color"])
            .unwrap_or(visual.surface_disabled);
        visual.border_idle =
            first_rgba_attribute(metadata, &["border_color"]).unwrap_or(visual.border_idle);
        visual.border_hover =
            first_rgba_attribute(metadata, &["hover_border_color"]).unwrap_or(visual.border_hover);
        visual.border_focus =
            first_rgba_attribute(metadata, &["focus_border_color"]).unwrap_or(visual.border_focus);
        visual.border_disabled = first_rgba_attribute(metadata, &["disabled_border_color"])
            .unwrap_or(visual.border_disabled);
        visual.text = first_rgba_attribute(metadata, &["foreground_color", "text_color"])
            .unwrap_or(visual.text);
        visual.placeholder_text = first_rgba_attribute(metadata, &["placeholder_color"])
            .unwrap_or(visual.placeholder_text);
        visual.text_disabled = first_rgba_attribute(metadata, &["disabled_foreground_color"])
            .unwrap_or(visual.text_disabled);
        visual.padding_left = metric_attribute(metadata, "layout_padding_left")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.padding_left);
        visual.padding_right = metric_attribute(metadata, "layout_padding_right")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.padding_right);
        visual.padding_top = metric_attribute(metadata, "layout_padding_top")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.padding_top);
        visual.padding_bottom = metric_attribute(metadata, "layout_padding_bottom")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.padding_bottom);
        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.border_width);
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
        visual.min_frame_extent = visual.border_width.max(f32::EPSILON);
        visual
    }
}

fn default_text_field_visual() -> &'static TextFieldVisual {
    static VISUAL: OnceLock<TextFieldVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        TextFieldVisual {
            surface_idle: colors.surface_recessed,
            surface_hover: colors.surface_hover,
            surface_pressed: colors.surface[3],
            surface_focused: colors.surface_recessed,
            surface_disabled: colors.surface_disabled,
            border_idle: colors.separator_soft,
            border_hover: colors.border,
            border_focus: colors.accent,
            border_disabled: colors.border_disabled,
            text: colors.text_primary,
            placeholder_text: colors.text_secondary,
            text_disabled: colors.text_disabled,
            padding_left: density.gap_medium,
            padding_right: density.gap_medium,
            padding_top: density.gap_small,
            padding_bottom: density.gap_small,
            border_width: controls.border_width,
            corner_radius: controls.control_radius,
            font_size: typography.body_size,
            line_height: typography.body_size * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            min_frame_extent: controls.border_width.max(f32::EPSILON),
        }
    })
}

pub(super) fn text_field_suppresses_owner_text(metadata: Option<&UiTemplateNodeMetadata>) -> bool {
    metadata.is_some_and(is_text_field)
}

pub(super) fn text_field_render_commands(
    node_id: UiNodeId,
    metadata: Option<&UiTemplateNodeMetadata>,
    state_flags: &UiStateFlags,
    component_state: Option<&UiComponentState>,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
    visible_text: Option<&str>,
    editable: Option<&UiEditableTextState>,
    text_measure_cache: &mut UiTextMeasureCache,
) -> Vec<UiRenderCommand> {
    let Some(metadata) = metadata else {
        return Vec::new();
    };
    if !is_text_field(metadata) {
        return Vec::new();
    }

    let visual = TextFieldVisual::resolve(metadata);
    if frame.width <= visual.min_frame_extent || frame.height <= visual.min_frame_extent {
        return Vec::new();
    }
    let state = TextFieldRenderState::resolve(metadata, state_flags, component_state);
    let mut commands = vec![surface_command(
        node_id, &state, &visual, frame, clip_frame, z_index, opacity,
    )];
    if visible_text.is_some() || editable.is_some_and(|editable| !editable.text.is_empty()) {
        commands.push(text_command(
            node_id,
            metadata,
            &state,
            &visual,
            frame,
            clip_frame,
            z_index.saturating_add(2),
            opacity,
            base_style,
            visible_text.unwrap_or_default(),
            editable,
            text_measure_cache,
        ));
    }
    commands
}

#[derive(Clone, Copy)]
struct TextFieldRenderState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
    surface_hot: bool,
}

impl TextFieldRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        let family = UiPainterFamily::TextField;
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

    fn focused(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Focused)
    }

    fn pressed(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Pressed)
    }

    fn hot(self) -> bool {
        self.surface_hot
    }
}

fn is_text_field(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.widget.resolved_behavior(&metadata.component) == UiWidgetBehavior::TextInput
}

fn surface_command(
    node_id: UiNodeId,
    state: &TextFieldRenderState,
    visual: &TextFieldVisual,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
) -> UiRenderCommand {
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Quad,
        frame,
        clip_frame,
        z_index: z_index.saturating_add(1),
        style: UiResolvedStyle {
            background_color: Some(css_color(surface_color(state, visual))),
            border_color: Some(css_color(border_color(state, visual))),
            border_width: visual.border_width,
            corner_radius: visual.corner_radius,
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
    metadata: &UiTemplateNodeMetadata,
    state: &TextFieldRenderState,
    visual: &TextFieldVisual,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    opacity: f32,
    base_style: &UiResolvedStyle,
    visible_text: &str,
    editable: Option<&UiEditableTextState>,
    text_measure_cache: &mut UiTextMeasureCache,
) -> UiRenderCommand {
    let text_frame = text_frame(frame, visual);
    let text_clip = clip_frame
        .and_then(|clip| clip.intersection(text_frame))
        .unwrap_or(text_frame);
    let source_is_placeholder = is_placeholder_text(metadata, visible_text);
    let secure_requested = !source_is_placeholder && secure_text_policy(metadata).is_secure();
    let secure_source = secure_requested
        .then(|| secure_text_source(visible_text, editable))
        .flatten()
        .filter(|text| !text.is_empty());
    let secure_presentation = (secure_requested && supports_secure_text_field_mvp(metadata))
        .then_some(secure_source)
        .flatten()
        .map(|source| UiSecureTextPresentation::new(source, base_style.text_direction.into()));
    let rendered_text = if secure_requested {
        secure_presentation
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .map_or_else(
                || UiSecureTextPresentation::mask_display_text(secure_source.unwrap_or_default()),
                |presentation| presentation.display_text().to_string(),
            )
    } else {
        visible_text.to_string()
    };
    let mut style = text_style(metadata, state, visual, base_style, &rendered_text);
    // Password text is a plain-text presentation owner. Rich parsing can only reinterpret the
    // display mask, but it would bypass the presentation glyph-artifact route and its explicit
    // source-range map.
    if secure_requested {
        style.rich_text_format = UiRichTextFormat::Plain;
    }
    let (mut layout, render_editable) = if !secure_requested {
        (
            resolve_text_field_layout(
                visible_text,
                &style,
                text_frame,
                text_clip,
                editable,
                text_measure_cache,
            ),
            editable.cloned(),
        )
    } else {
        match secure_presentation {
            Some(Ok(presentation)) => {
                let layout = resolve_secure_text_field_layout(
                    &presentation,
                    &style,
                    text_frame,
                    text_clip,
                    text_measure_cache,
                );
                let editable = editable.map(|state| presentation.render_editable_state(state));
                (layout, editable)
            }
            Some(Err(_)) | None => (
                secure_text_layout_failure(&style),
                editable.map(|state| secure_render_editable_state(state, rendered_text.as_str())),
            ),
        }
    };
    if state.focused() && !state.unavailable() {
        layout.editable = render_editable;
    }
    style = style.with_painter_state(state.family, state.visual_state);
    UiRenderCommand {
        node_id,
        kind: UiRenderCommandKind::Text,
        frame: text_frame,
        clip_frame: Some(text_clip),
        z_index,
        style,
        text_layout: Some(layout),
        text: Some(rendered_text),
        image: None,
        opacity,
    }
}

fn resolve_secure_text_field_layout(
    presentation: &UiSecureTextPresentation,
    style: &UiResolvedStyle,
    text_frame: UiFrame,
    text_clip: UiFrame,
    text_measure_cache: &mut UiTextMeasureCache,
) -> UiResolvedTextLayout {
    let mut layout_style = style.clone();
    if matches!(layout_style.text_direction, UiTextDirection::Auto) {
        if let Some(line) = presentation.lines().first() {
            layout_style.text_direction = line.bidi.resolved_base_direction.into();
        }
    }
    let request = UiTextLayoutRequest::new(
        presentation.display_text(),
        &layout_style,
        text_frame,
        Some(text_clip),
    );
    let mut layout = resolve_text_layout_with_cache(&request, text_measure_cache).layout;
    if apply_secure_text_presentation(&mut layout, presentation).is_ok() {
        return layout;
    }
    secure_text_layout_failure(&layout_style)
}

fn secure_text_layout_failure(style: &UiResolvedStyle) -> UiResolvedTextLayout {
    UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        direction: style.text_direction,
        writing_mode: style.text_writing_mode,
        overflow: style.text_overflow,
        font_size: style.font_size,
        line_height: style.line_height,
        measured_width: 0.0,
        measured_height: 0.0,
        source_range: Default::default(),
        lines: Vec::new(),
        boxes: Vec::new(),
        overflow_clipped: true,
        editable: None,
        rich_text_artifact: None,
    }
}

/// Secure IME normally leaves `visible_text` and committed state identical. If an upstream
/// caller temporarily exposes a non-empty visible preedit while the committed state is empty,
/// mask that text too rather than allowing the render path to fall back to raw presentation.
fn secure_text_source<'a>(
    visible_text: &'a str,
    editable: Option<&'a UiEditableTextState>,
) -> Option<&'a str> {
    editable
        .map(|state| state.text.as_str())
        .filter(|text| !text.is_empty())
        .or_else(|| (!visible_text.is_empty()).then_some(visible_text))
}

fn secure_render_editable_state(
    source: &UiEditableTextState,
    display_text: &str,
) -> UiEditableTextState {
    let source_len = source.text.len();
    let mut caret = source.caret.clone();
    caret.offset = caret.offset.min(source_len);
    let selection = source.selection.as_ref().map(|selection| {
        let mut selection = selection.clone();
        selection.anchor = selection.anchor.min(source_len);
        selection.focus = selection.focus.min(source_len);
        selection
    });
    UiEditableTextState {
        text: display_text.to_string(),
        caret,
        selection,
        composition: None,
        read_only: source.read_only,
    }
}

fn resolve_text_field_layout(
    visible_text: &str,
    style: &UiResolvedStyle,
    text_frame: UiFrame,
    text_clip: UiFrame,
    editable: Option<&UiEditableTextState>,
    text_measure_cache: &mut UiTextMeasureCache,
) -> UiResolvedTextLayout {
    let request = UiTextLayoutRequest::new(visible_text, style, text_frame, Some(text_clip));
    let Some(composition) = editable.and_then(|editable| editable.composition.as_ref()) else {
        return resolve_text_layout_with_cache(&request, text_measure_cache).layout;
    };
    let preedit = UiPreeditSpan {
        range: composition.range,
        text: composition.text.clone(),
    };
    resolve_text_layout_with_cache(&request.with_preedit(&preedit), text_measure_cache).layout
}

fn text_frame(frame: UiFrame, visual: &TextFieldVisual) -> UiFrame {
    UiFrame::new(
        frame.x + visual.padding_left,
        frame.y + visual.padding_top,
        (frame.width - visual.padding_left - visual.padding_right).max(visual.min_frame_extent),
        (frame.height - visual.padding_top - visual.padding_bottom).max(visual.min_frame_extent),
    )
}

fn text_style(
    metadata: &UiTemplateNodeMetadata,
    state: &TextFieldRenderState,
    visual: &TextFieldVisual,
    base_style: &UiResolvedStyle,
    visible_text: &str,
) -> UiResolvedStyle {
    let mut style = base_style.clone();
    style.background_color = None;
    style.border_color = None;
    style.border_width = 0.0;
    style.corner_radius = 0.0;
    style.font_size = visual.font_size;
    style.line_height = visual.line_height;
    style.foreground_color = Some(css_color(text_color(metadata, state, visual, visible_text)));
    style
}

fn surface_color(state: &TextFieldRenderState, visual: &TextFieldVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.surface_disabled
    } else if state.pressed() {
        visual.surface_pressed
    } else if state.focused() {
        visual.surface_focused
    } else if state.hot() {
        visual.surface_hover
    } else {
        visual.surface_idle
    }
}

fn border_color(state: &TextFieldRenderState, visual: &TextFieldVisual) -> UiRgbaColor {
    if state.unavailable() {
        visual.border_disabled
    } else if state.focused() || state.pressed() {
        visual.border_focus
    } else if state.hot() {
        visual.border_hover
    } else {
        visual.border_idle
    }
}

fn text_color(
    metadata: &UiTemplateNodeMetadata,
    state: &TextFieldRenderState,
    visual: &TextFieldVisual,
    visible_text: &str,
) -> UiRgbaColor {
    if state.unavailable() {
        visual.text_disabled
    } else if is_placeholder_text(metadata, visible_text) {
        visual.placeholder_text
    } else {
        visual.text
    }
}

fn is_placeholder_text(metadata: &UiTemplateNodeMetadata, visible_text: &str) -> bool {
    string_attribute(metadata, "placeholder").is_some_and(|placeholder| {
        !placeholder.is_empty()
            && placeholder == visible_text
            && string_attribute(
                metadata,
                metadata.widget.value_property.as_deref().unwrap_or("value"),
            )
            .unwrap_or_default()
            .is_empty()
    })
}

/// The first secure presentation cut intentionally owns only the established single-line input
/// family. Unsupported secure text controls still publish a masked command and empty layout;
/// they must never fall through to the ordinary multi-line text route with raw content.
fn supports_secure_text_field_mvp(metadata: &UiTemplateNodeMetadata) -> bool {
    matches!(
        metadata.component.as_str(),
        "InputField" | "TextField" | "LineEdit" | "NumberField" | "SearchField"
    ) && !metadata
        .attributes
        .get("multiline")
        .and_then(Value::as_bool)
        .unwrap_or(false)
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
