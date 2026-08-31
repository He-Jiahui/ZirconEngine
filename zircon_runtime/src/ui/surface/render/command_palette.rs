use std::{
    collections::{BTreeSet, HashMap},
    sync::OnceLock,
};

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
use super::popup_position::{PopupPlacement, resolve_anchored_popup_geometry};
use super::popup_rows::{PopupRowPaintState, push_popup_row_label, push_popup_row_surface};

const COMMANDS: &str = "commands";
const FILTERED_COMMANDS: &str = "filtered_commands";
const DISABLED_COMMANDS: &str = "disabled_commands";
const SELECTED_COMMAND_ID: &str = "selected_command_id";
const FOCUSED_INDEX: &str = "focused_index";
const QUERY: &str = "query";
const PLACEHOLDER: &str = "placeholder";
const RECENT_COMMANDS: &str = "recent_commands";
const COMMAND_SOURCE: &str = "command_source";

#[derive(Clone, Copy, Debug)]
struct CommandPaletteVisual {
    panel_surface: UiRgbaColor,
    panel_border: UiRgbaColor,
    field_surface: UiRgbaColor,
    field_border: UiRgbaColor,
    text: UiRgbaColor,
    muted_text: UiRgbaColor,
    border_width: f32,
    panel_radius: f32,
    search_radius: f32,
    panel_padding_x: f32,
    search_top: f32,
    search_height: f32,
    search_text_inset_x: f32,
    list_gap: f32,
    row_inset_x: f32,
    row_height: f32,
    font_size: f32,
    line_height: f32,
}

impl CommandPaletteVisual {
    fn resolve(metadata: &UiTemplateNodeMetadata) -> Self {
        let mut visual = *default_command_palette_visual();
        visual.panel_surface =
            first_rgba_attribute(metadata, &["background_color"]).unwrap_or(visual.panel_surface);
        visual.panel_border =
            first_rgba_attribute(metadata, &["border_color"]).unwrap_or(visual.panel_border);
        visual.field_surface = first_rgba_attribute(
            metadata,
            &["search_background_color", "field_background_color"],
        )
        .unwrap_or(visual.field_surface);
        visual.field_border =
            first_rgba_attribute(metadata, &["search_border_color", "focus_border_color"])
                .unwrap_or(visual.field_border);
        visual.text = first_rgba_attribute(metadata, &["foreground_color", "text_color"])
            .unwrap_or(visual.text);
        visual.muted_text =
            first_rgba_attribute(metadata, &["placeholder_color"]).unwrap_or(visual.muted_text);
        visual.border_width = metric_attribute(metadata, "border_width")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.border_width);
        visual.panel_radius = metric_attribute(metadata, "corner_radius")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.panel_radius);
        visual.search_radius = metric_attribute(metadata, "search_radius")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.search_radius);
        visual.panel_padding_x = metric_attribute(metadata, "panel_padding_x")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.panel_padding_x);
        visual.search_top = metric_attribute(metadata, "search_top")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.search_top);
        visual.search_height = metric_attribute(metadata, "search_height")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.search_height);
        visual.search_text_inset_x = metric_attribute(metadata, "search_text_inset_x")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.search_text_inset_x);
        visual.list_gap = metric_attribute(metadata, "list_gap")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.list_gap);
        visual.row_inset_x = metric_attribute(metadata, "row_inset_x")
            .filter(|value| *value >= 0.0)
            .unwrap_or(visual.row_inset_x);
        visual.row_height = metric_attribute(metadata, "row_height")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.row_height);
        visual.font_size = metric_attribute(metadata, "font_size")
            .filter(|value| *value > 0.0)
            .unwrap_or(visual.font_size);
        if let Some(line_height) =
            metric_attribute(metadata, "line_height").filter(|value| *value > 0.0)
        {
            visual.line_height = line_height;
        } else if let Some(line_height_ratio) =
            metric_attribute(metadata, "line_height_ratio").filter(|value| *value > 0.0)
        {
            visual.line_height = visual.font_size * line_height_ratio;
        }
        visual
    }

    fn list_top(self) -> f32 {
        self.search_top + self.search_height + self.list_gap
    }

    fn empty_text_top(self) -> f32 {
        self.list_top() + self.list_gap
    }
}

fn default_command_palette_visual() -> &'static CommandPaletteVisual {
    static VISUAL: OnceLock<CommandPaletteVisual> = OnceLock::new();
    VISUAL.get_or_init(|| {
        let tokens = EditorDesignTokens::workbench_dark();
        let colors = &tokens.palette;
        let controls = &tokens.controls;
        let density = &tokens.density;
        let typography = &tokens.typography;
        CommandPaletteVisual {
            panel_surface: colors.popup,
            panel_border: colors.border,
            field_surface: colors.surface_recessed,
            field_border: colors.accent,
            text: colors.text_primary,
            muted_text: colors.text_secondary,
            border_width: controls.border_width,
            panel_radius: controls.panel_radius,
            search_radius: controls.small_radius,
            panel_padding_x: density.gap_large,
            search_top: density.gap_medium,
            search_height: controls.compact_height,
            search_text_inset_x: density.gap_medium,
            list_gap: density.gap_medium,
            row_inset_x: density.gap_medium,
            row_height: density.row_height,
            font_size: typography.body_size,
            line_height: typography.body_size * EditorTypographyTokens::WORKBENCH_LINE_HEIGHT_RATIO,
        }
    })
}

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
    popup_anchor_frame: Option<UiFrame>,
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
    let visual = CommandPaletteVisual::resolve(metadata);
    let min_frame_extent = visual.border_width.max(f32::EPSILON);
    if frame.width <= min_frame_extent || frame.height <= min_frame_extent {
        return Vec::new();
    }
    let (frame, clip_frame) = resolve_anchored_popup_geometry(
        metadata,
        frame,
        popup_anchor_frame,
        clip_frame,
        PopupPlacement::Top,
        0.0,
    );

    let state = CommandPaletteRenderState::resolve(metadata, state_flags, component_state);
    let mut commands = vec![quad_command(
        node_id,
        frame,
        clip_frame,
        z_index.saturating_add(1),
        visual.panel_surface,
        Some(visual.panel_border),
        visual.border_width,
        visual.panel_radius,
        UiPainterFamily::Dropdown,
        state.visual_state,
        opacity,
    )];

    let search_frame = UiFrame::new(
        frame.x + visual.panel_padding_x,
        frame.y + visual.search_top,
        (frame.width - visual.panel_padding_x * 2.0).max(min_frame_extent),
        visual.search_height,
    );
    commands.push(quad_command(
        node_id,
        search_frame,
        clip_frame,
        z_index.saturating_add(2),
        visual.field_surface,
        Some(visual.field_border),
        visual.border_width,
        visual.search_radius,
        UiPainterFamily::TextField,
        UiPainterResolvedState::Focused,
        opacity,
    ));

    let query = string_attribute(metadata, QUERY).unwrap_or_default();
    let placeholder = string_attribute(metadata, PLACEHOLDER).unwrap_or("Search commands");
    let (search_text, search_color) = if query.trim().is_empty() {
        (placeholder.to_string(), visual.muted_text)
    } else {
        (query.to_string(), visual.text)
    };
    commands.push(text_command(
        node_id,
        UiFrame::new(
            search_frame.x + visual.search_text_inset_x,
            search_frame.y + (search_frame.height - visual.line_height).max(0.0) * 0.5,
            (search_frame.width - visual.search_text_inset_x * 2.0).max(min_frame_extent),
            visual.line_height.min(search_frame.height),
        ),
        clip_frame,
        z_index.saturating_add(3),
        search_text,
        search_color,
        visual.font_size,
        visual.line_height,
        UiPainterFamily::TextField,
        UiPainterResolvedState::Focused,
        opacity,
    ));

    let rows = command_rows(metadata);
    if rows.is_empty() {
        commands.push(text_command(
            node_id,
            UiFrame::new(
                frame.x + visual.panel_padding_x,
                frame.y + visual.empty_text_top(),
                (frame.width - visual.panel_padding_x * 2.0).max(min_frame_extent),
                visual.line_height,
            ),
            clip_frame,
            z_index.saturating_add(4),
            string_attribute(metadata, "empty_text")
                .unwrap_or("No commands found")
                .to_string(),
            visual.muted_text,
            visual.font_size,
            visual.line_height,
            UiPainterFamily::PopupRow,
            UiPainterResolvedState::Normal,
            opacity,
        ));
        return commands;
    }

    for (row, command) in rows.into_iter().enumerate() {
        let row_frame = UiFrame::new(
            frame.x + visual.row_inset_x,
            frame.y + visual.list_top() + row as f32 * visual.row_height,
            (frame.width - visual.row_inset_x * 2.0).max(min_frame_extent),
            visual.row_height,
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
            command.label,
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
            || contains_ascii_case(&self.id, query)
            || contains_ascii_case(&self.label, query)
            || contains_ascii_case(&self.source, query)
            || contains_ascii_case(&self.shortcut, query)
            || contains_ascii_case(&self.category, query)
            || self
                .keywords
                .iter()
                .any(|keyword| contains_ascii_case(keyword, query))
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
            let command_index = command_entry_index(&commands);
            command_id_values(filtered)
                .into_iter()
                .filter_map(|id| {
                    command_index
                        .get(id.as_str())
                        .map(|entry_index| commands[*entry_index].clone())
                        .or_else(|| (!id.is_empty()).then(|| CommandPaletteRow::new(id)))
                })
                .collect()
        } else {
            let source = string_attribute(metadata, COMMAND_SOURCE)
                .map(str::trim)
                .filter(|source| !source.is_empty());
            let query = string_attribute(metadata, QUERY)
                .map(str::trim)
                .filter(|query| !query.is_empty());
            commands
                .into_iter()
                .filter(|entry| entry.matches_source(source))
                .filter(|entry| entry.matches_query(query))
                .collect()
        };

    for (index, row) in rows.iter_mut().enumerate() {
        row.selected = !selected_id.is_empty() && row.matches_id(selected_id);
        row.disabled =
            row.disabled || disabled_ids.contains(&row.id) || disabled_ids.contains(&row.label);
        row.special = special_ids.contains(&row.id) || special_ids.contains(&row.label);
        row.focused = focused_index == Some(index);
    }

    rows
}

fn command_entry_index(commands: &[CommandPaletteRow]) -> HashMap<&str, usize> {
    let mut index = HashMap::with_capacity(commands.len().saturating_mul(2));
    for (entry_index, entry) in commands.iter().enumerate() {
        if !entry.id.is_empty() {
            index.entry(entry.id.as_str()).or_insert(entry_index);
        }
        if !entry.label.is_empty() {
            index.entry(entry.label.as_str()).or_insert(entry_index);
        }
    }
    index
}

fn contains_ascii_case(value: &str, needle: &str) -> bool {
    value
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
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

fn value_as_f32(value: &Value) -> Option<f32> {
    let value = match value {
        Value::Integer(value) => *value as f64,
        Value::Float(value) if value.is_finite() => *value,
        _ => return None,
    } as f32;
    value.is_finite().then_some(value)
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
            background_color: Some(css_color(background)),
            border_color: border.map(css_color),
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
    foreground: UiRgbaColor,
    font_size: f32,
    line_height: f32,
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
            foreground_color: Some(css_color(foreground)),
            font_size,
            line_height,
            ..UiResolvedStyle::default()
        }
        .with_painter_state(painter_family, painter_state),
        text_layout: None,
        text: Some(text),
        image: None,
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
