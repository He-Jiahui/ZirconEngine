use toml::Value;
use zircon_runtime_interface::ui::{
    component::UiComponentState,
    event_ui::{UiNodeId, UiStateFlags},
    layout::UiFrame,
    style::{UiPainterFamily, UiPainterResolvedState},
    surface::{UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiVisualAssetRef},
    tree::UiTemplateNodeMetadata,
};

use super::super::painter_state::UiRenderPainterStateSource;

pub(super) const FONT_SIZE: f32 = 12.0;
pub(super) const TABLE_FONT_SIZE: f32 = 11.0;
pub(super) const TEXT: &str = "#c5d0d5";
pub(super) const TEXT_SELECTED: &str = "#cce8ea";
pub(super) const TEXT_MUTED: &str = "#828c93";
pub(super) const TEXT_DISABLED: &str = "#59656c";
pub(super) const ACCENT: &str = "#35c7d0";
pub(super) const SURFACE_SELECTED: &str = "#0d4149";
pub(super) const SURFACE_HOVER: &str = "#1a2429";
pub(super) const SURFACE_PRESSED: &str = "#12343d";
pub(super) const SURFACE_DISABLED: &str = "#252c31";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CollectionRowKind {
    List,
    Tree,
    Table,
}

#[derive(Clone, Copy)]
pub(super) struct RowRenderState {
    family: UiPainterFamily,
    visual_state: UiPainterResolvedState,
    selected: bool,
    checked: bool,
    expanded: bool,
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
        }
    }

    pub(super) fn marked(self) -> bool {
        self.selected || self.checked
    }

    pub(super) fn expanded(self) -> bool {
        self.expanded
    }

    pub(super) fn disabled(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Disabled)
    }

    pub(super) fn pressed(self) -> bool {
        matches!(self.visual_state, UiPainterResolvedState::Pressed)
    }

    pub(super) fn hot(self) -> bool {
        matches!(
            self.visual_state,
            UiPainterResolvedState::Hovered
                | UiPainterResolvedState::Focused
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

pub(super) fn collection_row_kind(metadata: &UiTemplateNodeMetadata) -> Option<CollectionRowKind> {
    match metadata.component.as_str() {
        "ListRow" => Some(CollectionRowKind::List),
        "TreeRow" => Some(CollectionRowKind::Tree),
        "Table" | "TableRow" => Some(CollectionRowKind::Table),
        _ => None,
    }
}

pub(super) fn row_label(metadata: &UiTemplateNodeMetadata) -> Option<String> {
    ["label", "text", "value_text", "title"]
        .iter()
        .find_map(|key| string_attribute(metadata, key))
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_string)
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

pub(super) fn color_attribute<'a>(
    metadata: &'a UiTemplateNodeMetadata,
    key: &str,
) -> Option<&'a str> {
    metadata
        .style_overrides
        .get(key)
        .or_else(|| metadata.attributes.get(key))
        .and_then(Value::as_str)
        .filter(|color| !color.trim().is_empty())
}

pub(super) fn line_height(font_size: f32) -> f32 {
    font_size * 1.2
}

pub(super) fn quad_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    background: &str,
    border: Option<&str>,
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

pub(super) fn text_command(
    node_id: UiNodeId,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
    text: String,
    foreground: &str,
    font_size: f32,
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
            foreground_color: Some(foreground.to_string()),
            font_size,
            line_height: line_height(font_size),
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
    foreground: &str,
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
            foreground_color: Some(foreground.to_string()),
            ..UiResolvedStyle::default()
        }
        .with_painter_state(state.family, state.visual_state),
        text_layout: None,
        text: None,
        image: Some(UiVisualAssetRef::Icon(icon.to_string())),
        opacity,
    }
}

fn value_as_f32(value: &Value) -> Option<f32> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|value| value as f64))
        .map(|value| value as f32)
}
