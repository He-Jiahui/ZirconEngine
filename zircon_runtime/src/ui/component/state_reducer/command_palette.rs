use std::collections::{BTreeMap, HashSet};

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEventError, UiComponentKeyboardAction, UiComponentState,
    UiValidationState, UiValue,
};

use super::text_search::contains_lowercase_query;

const COMMANDS: &str = "commands";
const FILTERED_COMMANDS: &str = "filtered_commands";
const DISABLED_COMMANDS: &str = "disabled_commands";
const SELECTED_COMMAND_ID: &str = "selected_command_id";
const FOCUSED_INDEX: &str = "focused_index";
const QUERY: &str = "query";
const COMMAND_SOURCE: &str = "command_source";
const COMMITTED_COMMAND_ID: &str = "committed_command_id";
const CATALOG_GENERATION: &str = "catalog_generation";
const MATCH_COUNT: &str = "match_count";
const WINDOW_COUNT: &str = "window_count";
const WINDOW_OFFSET: &str = "window_offset";
const WINDOW_REQUEST_CURRENT_OFFSET: &str = "window_request_current_offset";
const WINDOW_REQUEST_OFFSET: &str = "window_request_offset";
const WINDOW_REQUEST_FOCUS: &str = "window_request_focus";
const WINDOW_REQUEST_GENERATION: &str = "window_request_generation";

const WINDOW_FOCUS_FIRST: &str = "first";
const WINDOW_FOCUS_LAST: &str = "last";

pub(super) fn sync_after_value_change(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    changed_property: &str,
) -> Result<bool, UiComponentEventError> {
    if !is_command_palette(descriptor) {
        return Ok(false);
    }
    if command_palette_sync_property(changed_property) {
        sync_filter_state(state, descriptor);
    }
    Ok(true)
}

pub(super) fn apply_keyboard_text(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    text: &str,
) -> Result<bool, UiComponentEventError> {
    if !is_command_palette(descriptor) {
        return Ok(false);
    }

    let text = text
        .chars()
        .filter(|character| !character.is_control())
        .collect::<String>();
    if text.is_empty() {
        return Ok(true);
    }

    let mut query = string_setting(state, descriptor, QUERY).unwrap_or_default();
    query.push_str(&text);
    super::set_value(state, QUERY.to_string(), UiValue::String(query));
    sync_filter_state(state, descriptor);
    Ok(true)
}

pub(super) fn apply_keyboard_action(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    action: UiComponentKeyboardAction,
) -> Result<bool, UiComponentEventError> {
    if !is_command_palette(descriptor) {
        return Ok(false);
    }
    if !bool_setting(state, descriptor, "keyboard_navigation", true) {
        return Ok(true);
    }

    match action {
        UiComponentKeyboardAction::Next
        | UiComponentKeyboardAction::Previous
        | UiComponentKeyboardAction::First
        | UiComponentKeyboardAction::Last
        | UiComponentKeyboardAction::LargeIncrement
        | UiComponentKeyboardAction::LargeDecrement => {
            navigate_filtered_commands(state, descriptor, action);
            Ok(true)
        }
        UiComponentKeyboardAction::Activate => {
            let command_id = focused_command_id(state, descriptor).unwrap_or_default();
            if !command_id.is_empty() {
                commit_command(state, descriptor, &command_id)?;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(super) fn apply_selection(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    _property: &str,
    option_id: &str,
    selected: bool,
) -> Result<bool, UiComponentEventError> {
    if !is_command_palette(descriptor) {
        return Ok(false);
    }

    if !selected {
        write_selected_command(state, "", -1);
        state.flags.selected = false;
        return Ok(true);
    }

    select_command(state, descriptor, option_id)?;
    Ok(true)
}

pub(super) fn apply_commit(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    value: &UiValue,
) -> Result<bool, UiComponentEventError> {
    if !is_command_palette(descriptor) {
        return Ok(false);
    }

    let command_id = command_id_from_commit(state, descriptor, property, value);
    if !command_id.is_empty() {
        commit_command(state, descriptor, &command_id)?;
    }
    Ok(true)
}

struct CommandFilterProjection {
    filtered: Vec<String>,
    disabled: HashSet<String>,
}

fn sync_filter_state(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> CommandFilterProjection {
    let entries = command_entries(state, descriptor);
    let query = string_setting(state, descriptor, QUERY)
        .map(|query| query.trim().to_lowercase())
        .filter(|query| !query.is_empty());
    let source = string_setting(state, descriptor, COMMAND_SOURCE)
        .map(|source| source.trim().to_lowercase())
        .filter(|source| !source.is_empty());

    let filtered = entries
        .iter()
        .filter(|entry| command_matches_source(entry, source.as_deref()))
        .filter(|entry| {
            query
                .as_deref()
                .map(|query| entry.matches_query(query))
                .unwrap_or(true)
        })
        .map(|entry| entry.id.clone())
        .collect::<Vec<_>>();
    let disabled = disabled_command_ids(state, &entries);
    let focus_index = next_focus_index(state, &filtered, &disabled);

    super::set_value(
        state,
        FILTERED_COMMANDS.to_string(),
        UiValue::Array(filtered.iter().cloned().map(UiValue::String).collect()),
    );
    write_selected_command(
        state,
        filtered
            .get(focus_index.max(0) as usize)
            .filter(|_| focus_index >= 0)
            .map(String::as_str)
            .unwrap_or_default(),
        focus_index,
    );
    state.flags.focused = focus_index >= 0;
    CommandFilterProjection { filtered, disabled }
}

fn navigate_filtered_commands(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    action: UiComponentKeyboardAction,
) {
    let projection = sync_filter_state(state, descriptor);
    let filtered = projection.filtered;
    if filtered.is_empty() {
        write_selected_command(state, "", -1);
        return;
    }

    let disabled = projection.disabled;
    let max_index = (filtered.len() - 1) as i64;
    let current = int_setting(state, descriptor, FOCUSED_INDEX)
        .unwrap_or(0)
        .clamp(0, max_index);
    let focusable = |index: i64| !disabled.contains(&filtered[index as usize]);
    let window = CommandPaletteWindow::read(state, descriptor, filtered.len());
    let next = match action {
        UiComponentKeyboardAction::First if window.offset == 0 => {
            (0..=max_index).find(|index| focusable(*index))
        }
        UiComponentKeyboardAction::Last if window.offset == window.last_offset() => {
            (0..=max_index).rev().find(|index| focusable(*index))
        }
        UiComponentKeyboardAction::Previous => (0..current).rev().find(|index| focusable(*index)),
        UiComponentKeyboardAction::Next => {
            ((current + 1)..=max_index).find(|index| focusable(*index))
        }
        _ => None,
    };

    if let Some(index) = next {
        write_selected_command(state, &filtered[index as usize], index);
        state.flags.focused = true;
    } else if let Some((offset, focus)) = window.request_for(action, current, max_index) {
        request_window(state, descriptor, offset, focus);
    } else if !focusable(current) {
        write_selected_command(state, "", -1);
        state.flags.focused = false;
    }
}

#[derive(Clone, Copy)]
struct CommandPaletteWindow {
    offset: usize,
    count: usize,
    total_count: usize,
}

impl CommandPaletteWindow {
    fn read(
        state: &UiComponentState,
        descriptor: &UiComponentDescriptor,
        filtered_count: usize,
    ) -> Self {
        let offset = usize_setting(state, descriptor, WINDOW_OFFSET).unwrap_or(0);
        let count = usize_setting(state, descriptor, WINDOW_COUNT)
            .filter(|count| *count > 0)
            .unwrap_or(filtered_count.max(1));
        let total_count = usize_setting(state, descriptor, MATCH_COUNT)
            .unwrap_or(offset.saturating_add(filtered_count))
            .max(offset.saturating_add(filtered_count));
        Self {
            offset,
            count,
            total_count,
        }
    }

    fn request_for(
        self,
        action: UiComponentKeyboardAction,
        current: i64,
        max_index: i64,
    ) -> Option<(usize, &'static str)> {
        let last_offset = self.last_offset();
        match action {
            UiComponentKeyboardAction::Next
                if current == max_index && self.offset < last_offset =>
            {
                Some((self.next_offset(), WINDOW_FOCUS_FIRST))
            }
            UiComponentKeyboardAction::Previous if current == 0 && self.offset > 0 => {
                Some((self.previous_offset(), WINDOW_FOCUS_LAST))
            }
            UiComponentKeyboardAction::First if self.offset > 0 => Some((0, WINDOW_FOCUS_FIRST)),
            UiComponentKeyboardAction::Last if self.offset < last_offset => {
                Some((last_offset, WINDOW_FOCUS_LAST))
            }
            UiComponentKeyboardAction::LargeDecrement if self.offset < last_offset => {
                Some((self.next_offset(), WINDOW_FOCUS_FIRST))
            }
            UiComponentKeyboardAction::LargeIncrement if self.offset > 0 => {
                Some((self.previous_offset(), WINDOW_FOCUS_LAST))
            }
            _ => None,
        }
    }

    fn last_offset(self) -> usize {
        self.total_count
            .saturating_sub(1)
            .checked_div(self.count)
            .unwrap_or(0)
            .saturating_mul(self.count)
    }

    fn next_offset(self) -> usize {
        self.offset
            .saturating_add(self.count)
            .min(self.last_offset())
    }

    fn previous_offset(self) -> usize {
        self.offset.saturating_sub(self.count)
    }
}

fn request_window(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    offset: usize,
    focus: &str,
) {
    let generation = int_setting(state, descriptor, CATALOG_GENERATION).unwrap_or(0);
    let current_offset = int_setting(state, descriptor, WINDOW_OFFSET).unwrap_or(0);
    super::set_value(
        state,
        WINDOW_REQUEST_CURRENT_OFFSET.to_string(),
        UiValue::Int(current_offset),
    );
    super::set_value(
        state,
        WINDOW_REQUEST_OFFSET.to_string(),
        UiValue::Int(i64::try_from(offset).unwrap_or(i64::MAX)),
    );
    super::set_value(
        state,
        WINDOW_REQUEST_FOCUS.to_string(),
        UiValue::String(focus.to_string()),
    );
    super::set_value(
        state,
        WINDOW_REQUEST_GENERATION.to_string(),
        UiValue::Int(generation),
    );
}

fn select_command(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    command_id: &str,
) -> Result<(), UiComponentEventError> {
    ensure_command_enabled(state, descriptor, command_id)?;
    let index = filtered_command_ids(state, descriptor)
        .iter()
        .position(|id| id == command_id)
        .or_else(|| {
            command_entries(state, descriptor)
                .iter()
                .position(|entry| entry.id == command_id)
        })
        .map(|index| index as i64)
        .unwrap_or(0);
    write_selected_command(state, command_id, index);
    state.flags.focused = true;
    state.flags.selected = true;
    Ok(())
}

fn commit_command(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    command_id: &str,
) -> Result<(), UiComponentEventError> {
    select_command(state, descriptor, command_id)?;
    super::set_value(
        state,
        COMMITTED_COMMAND_ID.to_string(),
        UiValue::String(command_id.to_string()),
    );
    super::overlay::close_popup(state, descriptor)?;
    Ok(())
}

fn ensure_command_enabled(
    state: &mut UiComponentState,
    descriptor: &UiComponentDescriptor,
    command_id: &str,
) -> Result<(), UiComponentEventError> {
    let entries = command_entries(state, descriptor);
    if disabled_command_ids(state, &entries).contains(command_id) {
        state.validation = UiValidationState::error(format!(
            "disabled command `{command_id}` cannot be selected"
        ));
        return Err(UiComponentEventError::DisabledOption {
            component_id: descriptor.id.clone(),
            option_id: command_id.to_string(),
        });
    }
    Ok(())
}

fn next_focus_index(
    state: &UiComponentState,
    filtered: &[String],
    disabled: &HashSet<String>,
) -> i64 {
    if filtered.is_empty() {
        return -1;
    }

    if let Some(selected) = state
        .values
        .get(SELECTED_COMMAND_ID)
        .and_then(string_value)
        .filter(|selected| !selected.is_empty())
    {
        if let Some(index) = filtered
            .iter()
            .position(|id| id == &selected && !disabled.contains(id))
        {
            return index as i64;
        }
    }

    if let Some(index) = int_value(state.values.get(FOCUSED_INDEX)).and_then(|index| {
        let index = usize::try_from(index).ok()?;
        filtered
            .get(index)
            .filter(|id| !disabled.contains(*id))
            .map(|_| index)
    }) {
        return index as i64;
    }

    filtered
        .iter()
        .position(|id| !disabled.contains(id))
        .map(|index| index as i64)
        .unwrap_or(-1)
}

fn write_selected_command(state: &mut UiComponentState, command_id: &str, focus_index: i64) {
    super::set_value(
        state,
        SELECTED_COMMAND_ID.to_string(),
        UiValue::String(command_id.to_string()),
    );
    super::set_value(state, FOCUSED_INDEX.to_string(), UiValue::Int(focus_index));
}

fn focused_command_id(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Option<String> {
    let filtered = filtered_command_ids(state, descriptor);
    let index = int_setting(state, descriptor, FOCUSED_INDEX)?;
    filtered.get(usize::try_from(index).ok()?).cloned()
}

fn command_id_from_commit(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    value: &UiValue,
) -> String {
    string_value(value)
        .or_else(|| string_setting(state, descriptor, SELECTED_COMMAND_ID))
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| property.to_string())
}

fn command_palette_sync_property(property: &str) -> bool {
    matches!(
        property,
        QUERY
            | COMMANDS
            | FILTERED_COMMANDS
            | DISABLED_COMMANDS
            | COMMAND_SOURCE
            | SELECTED_COMMAND_ID
            | FOCUSED_INDEX
    )
}

fn is_command_palette(descriptor: &UiComponentDescriptor) -> bool {
    descriptor.role == "command-palette" || descriptor.id == "CommandPalette"
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CommandEntry {
    id: String,
    label: String,
    source: String,
    shortcut: String,
    category: String,
    keywords: Vec<String>,
    disabled: bool,
}

impl CommandEntry {
    fn matches_query(&self, query: &str) -> bool {
        [
            self.id.as_str(),
            self.label.as_str(),
            self.source.as_str(),
            self.shortcut.as_str(),
            self.category.as_str(),
        ]
        .into_iter()
        .chain(self.keywords.iter().map(String::as_str))
        .any(|value| contains_lowercase_query(value, query))
    }
}

fn command_matches_source(entry: &CommandEntry, source: Option<&str>) -> bool {
    match source {
        Some(source) => entry.source.is_empty() || entry.source.eq_ignore_ascii_case(source),
        None => true,
    }
}

fn command_entries(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Vec<CommandEntry> {
    state
        .values
        .get(COMMANDS)
        .or_else(|| {
            descriptor
                .prop(COMMANDS)
                .and_then(|schema| schema.default_value.as_ref())
        })
        .map(command_entry_list)
        .unwrap_or_default()
}

fn command_entry_list(value: &UiValue) -> Vec<CommandEntry> {
    match value {
        UiValue::Array(values) => values.iter().flat_map(command_entry_list).collect(),
        UiValue::String(value) | UiValue::Enum(value) => {
            command_entry_from_string(value).into_iter().collect()
        }
        UiValue::Map(values) => command_entry_from_map(values).into_iter().collect(),
        _ => Vec::new(),
    }
}

fn command_entry_from_string(value: &str) -> Option<CommandEntry> {
    let mut parts = value.split('|');
    let id = parts.next()?.trim().to_string();
    if id.is_empty() {
        return None;
    }

    let mut entry = CommandEntry {
        label: id.clone(),
        id,
        source: String::new(),
        shortcut: String::new(),
        category: String::new(),
        keywords: Vec::new(),
        disabled: false,
    };

    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "label" | "text" | "title" | "name" => entry.label = value.to_string(),
            "source" | "command_source" | "commandSource" => entry.source = value.to_string(),
            "shortcut" | "accelerator" | "keybinding" => entry.shortcut = value.to_string(),
            "category" | "group" => entry.category = value.to_string(),
            "keywords" | "keyword" => entry.keywords = split_keywords(value),
            "disabled" => entry.disabled = matches!(value, "true" | "1" | "yes"),
            _ => {}
        }
    }
    Some(entry)
}

fn command_entry_from_map(values: &BTreeMap<String, UiValue>) -> Option<CommandEntry> {
    let id = first_string_value(values, &["id", "command_id", "commandId", "value", "key"])?;
    if id.is_empty() {
        return None;
    }

    Some(CommandEntry {
        label: first_string_value(values, &["label", "text", "title", "name", "value_text"])
            .unwrap_or_else(|| id.clone()),
        source: first_string_value(values, &["source", "command_source", "commandSource"])
            .unwrap_or_default(),
        shortcut: first_string_value(values, &["shortcut", "accelerator", "keybinding"])
            .unwrap_or_default(),
        category: first_string_value(values, &["category", "group"]).unwrap_or_default(),
        keywords: values
            .get("keywords")
            .or_else(|| values.get("keyword"))
            .map(keyword_values)
            .unwrap_or_default(),
        disabled: values.get("disabled").and_then(bool_value).unwrap_or(false)
            || values.get("enabled").and_then(bool_value) == Some(false),
        id,
    })
}

fn filtered_command_ids(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> Vec<String> {
    if let Some(value) = state.values.get(FILTERED_COMMANDS) {
        return command_id_values(value);
    }

    command_entries(state, descriptor)
        .into_iter()
        .map(|entry| entry.id)
        .collect()
}

fn disabled_command_ids(state: &UiComponentState, entries: &[CommandEntry]) -> HashSet<String> {
    let mut disabled = state
        .values
        .get(DISABLED_COMMANDS)
        .map(command_id_values)
        .unwrap_or_default()
        .into_iter()
        .collect::<HashSet<_>>();
    for entry in entries {
        if entry.disabled {
            disabled.insert(entry.id.clone());
        }
    }
    disabled
}

fn command_id_values(value: &UiValue) -> Vec<String> {
    match value {
        UiValue::Array(values) => values
            .iter()
            .flat_map(command_id_values)
            .filter(|value| !value.is_empty())
            .collect(),
        UiValue::String(value) | UiValue::Enum(value) => {
            vec![value.split('|').next().unwrap_or(value).trim().to_string()]
        }
        UiValue::Map(values) => {
            first_string_value(values, &["id", "command_id", "commandId", "value", "key"])
                .into_iter()
                .collect()
        }
        UiValue::Flags(values) => values.clone(),
        _ => Vec::new(),
    }
}

fn keyword_values(value: &UiValue) -> Vec<String> {
    match value {
        UiValue::Array(values) => values.iter().flat_map(keyword_values).collect(),
        UiValue::String(value) | UiValue::Enum(value) => split_keywords(value),
        _ => Vec::new(),
    }
}

fn split_keywords(value: &str) -> Vec<String> {
    value
        .split([',', ';'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn first_string_value(values: &BTreeMap<String, UiValue>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .filter_map(|key| values.get(*key).and_then(string_value))
        .find(|value| !value.is_empty())
}

fn string_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<String> {
    state
        .values
        .get(property)
        .and_then(string_value)
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(string_value)
        })
}

fn bool_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
    default_value: bool,
) -> bool {
    state
        .values
        .get(property)
        .and_then(bool_value)
        .or_else(|| {
            descriptor
                .prop(property)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(bool_value)
        })
        .unwrap_or(default_value)
}

fn int_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<i64> {
    int_value(state.values.get(property)).or_else(|| {
        descriptor
            .prop(property)
            .and_then(|schema| schema.default_value.as_ref())
            .and_then(|value| int_value(Some(value)))
    })
}

fn usize_setting(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    property: &str,
) -> Option<usize> {
    usize::try_from(int_setting(state, descriptor, property)?).ok()
}

fn string_value(value: &UiValue) -> Option<String> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value.clone()),
        _ => None,
    }
}

fn bool_value(value: &UiValue) -> Option<bool> {
    match value {
        UiValue::Bool(value) => Some(*value),
        _ => None,
    }
}

fn int_value(value: Option<&UiValue>) -> Option<i64> {
    match value {
        Some(UiValue::Int(value)) => Some(*value),
        Some(UiValue::Float(value)) => Some(value.round() as i64),
        _ => None,
    }
}
