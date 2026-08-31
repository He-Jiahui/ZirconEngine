use std::sync::OnceLock;

use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    design_tokens::{EditorDesignTokens, EditorTypographyTokens},
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState, UiRgbaColor},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
    tree::UiTemplateNodeMetadata,
};

use super::super::painter_state::UiRenderPainterStateSource;

#[cfg(test)]
mod direct_hex_color_tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CollectionRowKind {
    List,
    Tree,
    Table,
}

#[derive(Clone, Copy)]
pub(super) struct RowRenderState {
    pub(super) family: UiPainterFamily,
    pub(super) visual_state: UiPainterResolvedState,
    selected: bool,
    checked: bool,
    expanded: bool,
    surface_hot: bool,
}

impl RowRenderState {
    pub(super) fn resolve(
        kind: CollectionRowKind,
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let component_flags = component_state.map(|state| &state.flags);
        let selected = component_flags.is_some_and(|flags| flags.selected)
            || bool_attribute(metadata, "selected").unwrap_or(false);
        let checked = component_flags.is_some_and(|flags| flags.checked)
            || state_flags.checked
            || bool_attribute(metadata, "checked")
                .or_else(|| bool_attribute(metadata, "value"))
                .unwrap_or(false);
        let expanded = component_flags.is_some_and(|flags| flags.expanded)
            || bool_attribute(metadata, "expanded").unwrap_or(false);
        let surface_hot = component_flags.is_some_and(|flags| {
            flags.hovered
                || flags.drop_hovered
                || flags.active_drag_target
                || flags.dragging
                || flags.popup_open
        }) || bool_attribute(metadata, "hovered")
            .or_else(|| bool_attribute(metadata, "hover"))
            .or_else(|| bool_attribute(metadata, "drop_hovered"))
            .or_else(|| bool_attribute(metadata, "active_drag_target"))
            .or_else(|| bool_attribute(metadata, "dragging"))
            .or_else(|| bool_attribute(metadata, "open"))
            .or_else(|| bool_attribute(metadata, "popup_open"))
            .unwrap_or(false);
        let mut painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state_with_value_checked();
        painter_state.checked = checked;
        painter_state.selected = selected;
        let family = match kind {
            CollectionRowKind::List => UiPainterFamily::ListRow,
            CollectionRowKind::Tree => UiPainterFamily::TreeRow,
            CollectionRowKind::Table => UiPainterFamily::TableRow,
        };
        Self {
            family,
            visual_state: painter_state.resolved_state_for_family(family),
            selected,
            checked,
            expanded,
            surface_hot,
        }
    }

    pub(super) fn marked(self) -> bool {
        self.selected || self.checked
    }

    pub(super) fn expanded(self) -> bool {
        self.expanded
    }

    pub(super) fn unavailable(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Disabled)
            || matches!(self.visual_state, UiPainterResolvedState::Loading)
    }

    pub(super) fn pressed(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Pressed)
    }

    pub(super) fn hot(self) -> bool {
        self.surface_hot
            || matches!(
                self.visual_state,
                UiPainterResolvedState::Hovered
                    | UiPainterResolvedState::Open
                    | UiPainterResolvedState::Dragging
                    | UiPainterResolvedState::DropHovered
            )
    }

    pub(super) fn focus_or_press(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Focused
                | UiPainterResolvedState::Pressed
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
        )
    }
}

/// Shared visual roles keep list, tree, and table rows in one token-driven state model.
#[derive(Clone, Copy)]
pub(super) struct CollectionRowVisual {
    pub(super) focus_surface: UiRgbaColor,
    pub(super) table_surface: UiRgbaColor,
    pub(super) table_header_surface: UiRgbaColor,
    pub(super) table_tail_surface: UiRgbaColor,
    pub(super) hover_surface: UiRgbaColor,
    pub(super) selected_surface: UiRgbaColor,
    pub(super) selected_hover_surface: UiRgbaColor,
    pub(super) pressed_surface: UiRgbaColor,
    pub(super) disabled_surface: UiRgbaColor,
    pub(super) separator: UiRgbaColor,
    pub(super) focus_border: UiRgbaColor,
    pub(super) text_primary: UiRgbaColor,
    pub(super) text_secondary: UiRgbaColor,
    pub(super) text_selected: UiRgbaColor,
    pub(super) text_disabled: UiRgbaColor,
    pub(super) icon_secondary: UiRgbaColor,
    pub(super) icon_selected: UiRgbaColor,
    pub(super) border_width: f32,
    pub(super) corner_radius: f32,
    pub(super) body_font_size: f32,
    pub(super) caption_font_size: f32,
    pub(super) line_height_ratio: f32,
    pub(super) inline_inset: f32,
    pub(super) compact_inset: f32,
    pub(super) action_size: f32,
    pub(super) action_gap: f32,
    pub(super) tree_indent: f32,
}

impl CollectionRowVisual {
    pub(super) fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_collection_row_visual();
        if let Some(color) = first_rgba_attribute(metadata, &["background_color"]) {
            visual.focus_surface = color;
            visual.table_surface = color;
            visual.selected_surface = color;
            visual.selected_hover_surface = color;
        }
        visual.hover_surface = first_rgba_attribute(metadata, &["hover_background_color"])
            .unwrap_or(visual.hover_surface);
        visual.selected_surface = first_rgba_attribute(metadata, &["selected_background_color"])
            .unwrap_or(visual.selected_surface);
        visual.selected_hover_surface =
            first_rgba_attribute(metadata, &["selected_hover_background_color"])
                .unwrap_or(visual.selected_hover_surface);
        visual.pressed_surface = first_rgba_attribute(metadata, &["pressed_background_color"])
            .unwrap_or(visual.pressed_surface);
        visual.disabled_surface = first_rgba_attribute(metadata, &["disabled_background_color"])
            .unwrap_or(visual.disabled_surface);
        visual.separator = first_rgba_attribute(metadata, &["separator_color", "border_color"])
            .unwrap_or(visual.separator);
        visual.focus_border =
            first_rgba_attribute(metadata, &["focus_border_color", "border_color"])
                .unwrap_or(visual.focus_border);

        if let Some(color) = first_rgba_attribute(metadata, &["foreground_color", "text_color"]) {
            visual.text_primary = color;
            visual.text_selected = color;
        }
        visual.text_secondary =
            first_rgba_attribute(metadata, &["secondary_foreground_color", "value_color"])
                .unwrap_or(visual.text_secondary);
        visual.text_selected = first_rgba_attribute(metadata, &["selected_foreground_color"])
            .unwrap_or(visual.text_selected);
        visual.text_disabled = first_rgba_attribute(metadata, &["disabled_foreground_color"])
            .unwrap_or(visual.text_disabled);
        visual.icon_secondary =
            first_rgba_attribute(metadata, &["icon_color"]).unwrap_or(visual.icon_secondary);
        visual.icon_selected = first_rgba_attribute(metadata, &["selected_icon_color"])
            .unwrap_or(visual.icon_selected);

        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.border_width);
        visual.corner_radius = metric_attribute(metadata, "corner_radius")
            .or_else(|| metric_attribute(metadata, "radius"))
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.corner_radius);
        if let Some(font_size) =
            metric_attribute(metadata, "font_size").filter(|value| *value > 0.0)
        {
            visual.body_font_size = font_size;
            visual.caption_font_size = font_size;
        }
        visual.caption_font_size = metric_attribute(metadata, "table_font_size")
            .or_else(|| metric_attribute(metadata, "caption_font_size"))
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.caption_font_size);
        visual.line_height_ratio = metric_attribute(metadata, "line_height_ratio")
            .or_else(|| metric_attribute(metadata, "line_height"))
            .filter(|value| *value > 0.0 && *value <= 4.0)
            .unwrap_or(visual.line_height_ratio);
        visual.inline_inset = metric_attribute(metadata, "layout_padding_left")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.inline_inset);
        visual.compact_inset = metric_attribute(metadata, "layout_spacing")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.compact_inset);
        visual.action_size = metric_attribute(metadata, "layout_icon_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.action_size);
        visual.action_gap = metric_attribute(metadata, "layout_action_gap")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.action_gap);
        visual.tree_indent = metric_attribute(metadata, "tree_indent_px")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.tree_indent);
        visual
    }

    pub(super) fn line_height(self, font_size: f32) -> f32 {
        font_size * self.line_height_ratio
    }
}

fn default_collection_row_visual() -> &'static CollectionRowVisual {
    static VISUAL: OnceLock<CollectionRowVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let palette = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        CollectionRowVisual {
            focus_surface: palette.surface[1],
            table_surface: palette.surface_recessed,
            table_header_surface: palette.surface[1],
            table_tail_surface: palette.surface[0],
            hover_surface: palette.surface_hover,
            selected_surface: palette.surface_selected,
            selected_hover_surface: palette.accent_soft,
            pressed_surface: palette.surface[3],
            disabled_surface: palette.surface_disabled,
            separator: palette.separator_soft,
            focus_border: palette.accent,
            text_primary: palette.text_primary,
            text_secondary: palette.text_secondary,
            text_selected: palette.text_primary,
            text_disabled: palette.text_disabled,
            icon_secondary: palette.text_secondary,
            icon_selected: palette.text_primary,
            border_width: controls.border_width,
            corner_radius: controls.small_radius,
            body_font_size: typography.body_size,
            caption_font_size: typography.caption_size,
            line_height_ratio: EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
            inline_inset: density.gap_medium,
            compact_inset: density.gap_small,
            action_size: density.gap_large,
            action_gap: density.gap_medium,
            tree_indent: density.gap_large + density.gap_medium,
        }
    })
}

pub(super) fn collection_row_kind(metadata: &UiTemplateNodeMetadata) -> Option<CollectionRowKind> {
    match metadata.component.as_str() {
        "ListRow" => Some(CollectionRowKind::List),
        "TreeRow" => Some(CollectionRowKind::Tree),
        "Table" | "TableRow" => Some(CollectionRowKind::Table),
        _ => None,
    }
}

pub(super) fn row_label(metadata: &UiTemplateNodeMetadata) -> Option<&str> {
    ["label", "text", "value_text", "title"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|label| !label.is_empty())
}

pub(super) fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

pub(super) fn number_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata.attributes.get(key).and_then(value_as_f32)
}

pub(super) fn string_attribute<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    key: &str,
) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}

pub(super) fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background: UiRgbaColor,
    border: Option<UiRgbaColor>,
    border_width: f32,
    corner_radius: f32,
    state: &RowRenderState,
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

pub(super) fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: UiRgbaColor,
    font_size: f32,
    line_height: f32,
    state: &RowRenderState,
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

pub(super) fn icon_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    icon: &str,
    foreground: UiRgbaColor,
    state: &RowRenderState,
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

fn metric_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<f32> {
    metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(value_as_f32)
}

fn value_as_f32(value: &Value) -> Option<f32> {
    let value = match value {
        Value::Integer(value) => *value as f64,
        Value::Float(value) if value.is_finite() => *value,
        _ => return None,
    } as f32;
    value.is_finite().then_some(value)
}

fn parse_css_color(value: &str) -> Option<UiRgbaColor> {
    let encoded = value.trim().strip_prefix('#')?;
    let encoded = encoded.as_bytes();
    let (red, green, blue, alpha) = match encoded.len() {
        6 => (
            decode_hex_byte(encoded, 0)?,
            decode_hex_byte(encoded, 2)?,
            decode_hex_byte(encoded, 4)?,
            u8::MAX,
        ),
        8 => (
            decode_hex_byte(encoded, 0)?,
            decode_hex_byte(encoded, 2)?,
            decode_hex_byte(encoded, 4)?,
            decode_hex_byte(encoded, 6)?,
        ),
        _ => return None,
    };
    Some(UiRgbaColor::from_u8(red, green, blue, alpha))
}

fn decode_hex_byte(encoded: &[u8], offset: usize) -> Option<u8> {
    let high = decode_hex_digit(*encoded.get(offset)?)?;
    let low = decode_hex_digit(*encoded.get(offset + 1)?)?;
    Some((high << 4) | low)
}

fn decode_hex_digit(encoded: u8) -> Option<u8> {
    match encoded {
        b'0'..=b'9' => Some(encoded - b'0'),
        b'a'..=b'f' => Some(encoded - b'a' + 10),
        b'A'..=b'F' => Some(encoded - b'A' + 10),
        _ => None,
    }
}

fn css_color(color: UiRgbaColor) -> String {
    let [red, green, blue, alpha] = color.to_u8();
    if alpha == u8::MAX {
        format!("#{red:02x}{green:02x}{blue:02x}")
    } else {
        format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
    }
}
