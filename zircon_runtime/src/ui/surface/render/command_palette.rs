use std::collections::BTreeSet;

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
use super::popup_rows::{push_popup_row_label, push_popup_row_surface, PopupRowPaintState};

const COMMANDS: &str = "commands";
const FILTERED_COMMANDS: &str = "filtered_commands";
const DISABLED_COMMANDS: &str = "disabled_commands";
const SELECTED_COMMAND_ID: &str = "selected_command_id";
const FOCUSED_INDEX: &str = "focused_index";
const QUERY: &str = "query";
const PLACEHOLDER: &str = "placeholder";
const RECENT_COMMANDS: &str = "recent_commands";
const COMMAND_SOURCE: &str = "command_source";

const PANEL_SURFACE: &str = "#151b1f";
const PANEL_BORDER: &str = "#303840";
const FIELD_SURFACE: &str = "#10161a";
const FIELD_BORDER: &str = "#35c7d0";
const TEXT: &str = "#e8ecee";
const MUTED_TEXT: &str = "#59656c";

const PANEL_RADIUS: f32 = 6.0;
const PANEL_PADDING_X: f32 = 12.0;
const SEARCH_TOP: f32 = 10.0;
const SEARCH_HEIGHT: f32 = 30.0;
const SEARCH_TEXT_X: f32 = 10.0;
const SEARCH_TEXT_Y: f32 = 7.0;
const LIST_TOP: f32 = 48.0;
const ROW_INSET_X: f32 = 8.0;
const ROW_HEIGHT: f32 = 26.0;
const EMPTY_TEXT_Y: f32 = 58.0;
const TEXT_FONT_SIZE: f32 = 12.0;
const TEXT_LINE_HEIGHT: f32 = 14.4;

pub(super) fn command_palette_suppresses_owner_text(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_command_palette)
}

pub(super) fn command_palette_suppresses_owner_image(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_command_palette)
}

pub(super) fn command_palette_suppresses_owner_surface(
    metadata: Option<&UiTemplateNodeMetadata>,
) -> bool {
    metadata.is_some_and(is_command_palette)
}

pub(super) fn command_palette_render_commands(
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
    if !is_command_palette(metadata) || !command_palette_open(metadata) {
        return Vec::new();
    }
    if frame.width <= 1.0 || frame.height <= 1.0 {
        return Vec::new();
    }

    let state = CommandPaletteRenderState::resolve(metadata, state_flags, component_state);
    let mut commands = vec![quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        PANEL_SURFACE,
        Some(PANEL_BORDER),
        1.0,
        PANEL_RADIUS,
        UiPainterFamily::Dropdown,
        state.visual_state,
        opacity,
    )];

    let search_frame = UiFrame::new(
        frame.x + PANEL_PADDING_X,
        frame.y + SEARCH_TOP,
        (frame.width - PANEL_PADDING_X * 2.0).max(1.0),
        SEARCH_HEIGHT,
    );
    commands.push(quad_command(
        node_id,
        search_frame,
        clip_frame,
        z_index.saturating_add(2),
        FIELD_SURFACE,
        Some(FIELD_BORDER),
        1.0,
        4.0,
        UiPainterFamily::TextField,
        UiPainterResolvedState::Focused,
        opacity,
    ));

    let query = string_attribute(metadata, QUERY).unwrap_or_default();
    let placeholder = string_attribute(metadata, PLACEHOLDER).unwrap_or("Search commands");
    let (search_text, search_color) = if query.trim().is_empty() {
        (placeholder.to_string(), MUTED_TEXT)
    } else {
        (query.to_string(), TEXT)
    };
    commands.push(text_command(
        node_id,
        UiFrame::new(
            search_frame.x + SEARCH_TEXT_X,
            search_frame.y + SEARCH_TEXT_Y,
            (search_frame.width - SEARCH_TEXT_X * 2.0).max(1.0),
            TEXT_LINE_HEIGHT,
        ),
        clip_frame,
        z_index.saturating_add(3),
        search_text,
        search_color,
        UiPainterFamily::TextField,
        UiPainterResolvedState::Focused,
        opacity,
    ));

    let rows = command_rows(metadata);
    if rows.is_empty() {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                frame.x + PANEL_PADDING_X,
                frame.y + EMPTY_TEXT_Y,
                (frame.width - PANEL_PADDING_X * 2.0).max(1.0),
                TEXT_LINE_HEIGHT,
            ),
            clip_frame,
            z_index.saturating_add(4),
            string_attribute(metadata, "empty_text")
                .unwrap_or("No commands found")
                .to_string(),
            MUTED_TEXT,
            UiPainterFamily::PopupRow,
            UiPainterResolvedState::Normal,
            opacity,
        ));
        return commands;
    }

    for (row, command) in rows.iter().enumerate() {
        let row_frame = UiFrame::new(
            frame.x + ROW_INSET_X,
            frame.y + LIST_TOP + row as f32 * ROW_HEIGHT,
            (frame.width - ROW_INSET_X * 2.0).max(1.0),
            ROW_HEIGHT,
        );
        let row_z = z_index.saturating_add(4 + row as i32 * 3);
        let row_state = command.paint_state();
        push_popup_row_surface(
            &mut commands,
            node_id,
            row_frame,
            clip_frame,
            row_z,
            row_state,
            opacity,
        );
        push_popup_row_label(
            &mut commands,
            node_id,
            row_frame,
            clip_frame,
            row_z.saturating_add(2),
            command.label.clone(),
            row_state.text_color(false),
            row_state,
            opacity,
        );
    }

    commands
}

fn is_command_palette(metadata: &UiTemplateNodeMetadata) -> bool {
    metadata.component == "CommandPalette"
}

fn command_palette_open(metadata: &UiTemplateNodeMetadata) -> bool {
    bool_attribute(metadata, "popup_open")
        .or_else(|| bool_attribute(metadata, "open"))
        .unwrap_or(false)
}

#[derive(Clone, Copy)]
struct CommandPaletteRenderState {
    visual_state: UiPainterResolvedState,
}

impl CommandPaletteRenderState {
    fn resolve(
        metadata: &UiTemplateNodeMetadata,
        state_flags: &UiStateFlags,
        component_state: Option<&UiComponentState>,
    ) -> Self {
        let painter_state =
            UiRenderPainterStateSource::new(Some(metadata), state_flags, component_state)
                .painter_state();
        Self {
            visual_state: painter_state.resolved_state_for_family(UiPainterFamily::Dropdown),
        }
    }
}

#[derive(Clone, Debug)]
struct CommandPaletteRow {
    id: String,
    label: String,
    source: String,
    shortcut: String,
    category: String,
    keywords: Vec<String>,
    selected: bool,
    disabled: bool,
    special: bool,
    focused: bool,
}

impl CommandPaletteRow {
    fn new(id: String) -> Self {
        Self {
            label: id.clone(),
            id,
            source: String::new(),
            shortcut: String::new(),
            category: String::new(),
            keywords: Vec::new(),
            selected: false,
            disabled: false,
            special: false,
            focused: false,
        }
    }

    fn matches_query(&self, query: Option<&str>) -> bool {
        let Some(query) = query else {
            return true;
        };
        let query = query.trim();
        query.is_empty()
            || self.id.to_ascii_lowercase().contains(query)
            || self.label.to_ascii_lowercase().contains(query)
            || self.source.to_ascii_lowercase().contains(query)
            || self.shortcut.to_ascii_lowercase().contains(query)
            || self.category.to_ascii_lowercase().contains(query)
            || self
                .keywords
                .iter()
                .any(|keyword| keyword.to_ascii_lowercase().contains(query))
    }

    fn matches_source(&self, source: Option<&str>) -> bool {
        match source {
            Some(source) => self.source.is_empty() || self.source.eq_ignore_ascii_case(source),
            None => true,
        }
    }

    fn matches_id(&self, id: &str) -> bool {
        !id.is_empty() && (self.id == id || self.label == id)
    }

    fn paint_state(&self) -> PopupRowPaintState {
        PopupRowPaintState::resolve(
            self.selected || self.special,
            false,
            self.focused,
            false,
            self.disabled,
            false,
        )
    }
}

fn command_rows(metadata: &UiTemplateNodeMetadata) -> Vec<CommandPaletteRow> {
    let commands = metadata
        .attributes
        .get(COMMANDS)
        .map(command_entry_list)
        .unwrap_or_default();
    let selected_id = string_attribute(metadata, SELECTED_COMMAND_ID).unwrap_or_default();
    let disabled_ids = command_id_set(metadata.attributes.get(DISABLED_COMMANDS));
    let special_ids = command_id_set(metadata.attributes.get(RECENT_COMMANDS));
    let focused_index = usize_attribute(metadata, FOCUSED_INDEX);

    let mut rows: Vec<CommandPaletteRow> =
        if let Some(filtered) = metadata.attributes.get(FILTERED_COMMANDS) {
            command_id_values(filtered)
                .into_iter()
                .filter_map(|id| {
                    commands
                        .iter()
                        .find(|entry| entry.matches_id(&id))
                        .cloned()
                        .or_else(|| (!id.is_empty()).then(|| CommandPaletteRow::new(id)))
                })
                .collect()
        } else {
            let source = string_attribute(metadata, COMMAND_SOURCE)
                .map(|source| source.trim().to_ascii_lowercase())
                .filter(|source| !source.is_empty());
            let query = string_attribute(metadata, QUERY)
                .map(|query| query.trim().to_ascii_lowercase())
                .filter(|query| !query.is_empty());
            commands
                .into_iter()
                .filter(|entry| entry.matches_source(source.as_deref()))
                .filter(|entry| entry.matches_query(query.as_deref()))
                .collect()
        };

    for (index, row) in rows.iter_mut().enumerate() {
        row.selected = !selected_id.is_empty() && row.matches_id(selected_id);
        row.disabled = row.disabled || disabled_ids.iter().any(|id| row.matches_id(id));
        row.special = special_ids.iter().any(|id| row.matches_id(id));
        row.focused = focused_index == Some(index);
    }

    rows
}

fn command_entry_list(value: &Value) -> Vec<CommandPaletteRow> {
    match value {
        Value::Array(values) => values.iter().flat_map(command_entry_list).collect(),
        Value::String(value) => command_entry_from_string(value).into_iter().collect(),
        Value::Table(values) => command_entry_from_table(values).into_iter().collect(),
        _ => Vec::new(),
    }
}

fn command_entry_from_string(value: &str) -> Option<CommandPaletteRow> {
    let mut parts = value.split('|');
    let id = parts.next()?.trim().to_string();
    if id.is_empty() {
        return None;
    }

    let mut entry = CommandPaletteRow::new(id);
    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "label" | "text" | "title" | "name" => entry.label = value.to_string(),
            "source" | "command_source" | "commandSource" => entry.source = value.to_string(),
            "shortcut" | "accelerator" | "keybinding" => entry.shortcut = value.to_string(),
            "category" | "group" => entry.category = value.to_string(),
            "keywords" => entry.keywords = split_keywords(value),
            "disabled" => entry.disabled = matches!(value, "true" | "1" | "yes"),
            "enabled" => entry.disabled = matches!(value, "false" | "0" | "no"),
            _ => {}
        }
    }
    Some(entry)
}

fn command_entry_from_table(values: &toml::map::Map<String, Value>) -> Option<CommandPaletteRow> {
    let id = first_string_value(values, &["id", "command_id", "commandId", "value", "key"])?;
    if id.is_empty() {
        return None;
    }

    Some(CommandPaletteRow {
        label: first_string_value(values, &["label", "text", "title", "name", "value_text"])
            .unwrap_or_else(|| id.clone()),
        source: first_string_value(values, &["source", "command_source", "commandSource"])
            .unwrap_or_default(),
        shortcut: first_string_value(values, &["shortcut", "accelerator", "keybinding"])
            .unwrap_or_default(),
        category: first_string_value(values, &["category", "group"]).unwrap_or_default(),
        keywords: first_string_value(values, &["keywords"])
            .map(|value| split_keywords(&value))
            .unwrap_or_default(),
        disabled: values.get("disabled").and_then(bool_value).unwrap_or(false)
            || values.get("enabled").and_then(bool_value) == Some(false),
        selected: false,
        special: false,
        focused: false,
        id,
    })
}

fn command_id_set(value: Option<&Value>) -> BTreeSet<String> {
    value
        .map(command_id_values)
        .unwrap_or_default()
        .into_iter()
        .collect()
}

fn command_id_values(value: &Value) -> Vec<String> {
    match value {
        Value::Array(values) => values
            .iter()
            .flat_map(command_id_values)
            .filter(|value| !value.is_empty())
            .collect(),
        Value::String(value) => vec![value.split('|').next().unwrap_or(value).trim().to_string()],
        Value::Table(values) => {
            first_string_value(values, &["id", "command_id", "commandId", "value", "key"])
                .into_iter()
                .collect()
        }
        _ => Vec::new(),
    }
}

fn first_string_value(values: &toml::map::Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| values.get(*key).and_then(string_value))
        .find(|value| !value.is_empty())
}

fn split_keywords(value: &str) -> Vec<String> {
    value
        .split([',', ';'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn string_attribute<'a>(metadata: &'a UiTemplateNodeMetadata, key: &str) -> Option<&'a str> {
    metadata.attributes.get(key).and_then(Value::as_str)
}

fn string_value(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn bool_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<bool> {
    metadata.attributes.get(key).and_then(Value::as_bool)
}

fn bool_value(value: &Value) -> Option<bool> {
    match value {
        Value::Boolean(value) => Some(*value),
        Value::String(value) => match value.trim() {
            "true" | "1" | "yes" => Some(true),
            "false" | "0" | "no" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn usize_attribute(metadata: &UiTemplateNodeMetadata, key: &str) -> Option<usize> {
    match metadata.attributes.get(key)? {
        Value::Integer(value) => (*value >= 0).then_some(*value as usize),
        Value::Float(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        _ => None,
    }
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
    painter_family: UiPainterFamily,
    painter_state: UiPainterResolvedState,
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
        .with_painter_state(painter_family, painter_state),
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
    painter_family: UiPainterFamily,
    painter_state: UiPainterResolvedState,
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
            font_size: TEXT_FONT_SIZE,
            line_height: TEXT_LINE_HEIGHT,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(painter_family, painter_state),
        text_layout: None,
        text: Some(text),
        image: None,
        opacity,
    }
}
