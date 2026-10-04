use std::collections::{HashMap, HashSet};

use zircon_runtime_interface::ui::component::UiValue;

/// A stable movement request over a filtered command view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogNavigation {
    Next,
    Previous,
    First,
    Last,
}

#[derive(Clone, Debug)]
struct CommandEntry {
    id: String,
    source: String,
    corpus: String,
    disabled: bool,
}

/// A generation-owned command catalog.
///
/// Authoring `UiValue` payloads are decoded exactly once per catalog source
/// generation. Query changes rebuild only the filtered mapping; arrow keys use
/// precomputed neighboring enabled rows.
#[derive(Clone, Debug)]
pub struct CommandCatalog {
    generation: u64,
    entries: Vec<CommandEntry>,
    id_to_entry: HashMap<String, usize>,
    externally_disabled: HashSet<String>,
    filtered: Vec<usize>,
    previous_enabled: Vec<Option<usize>>,
    next_enabled: Vec<Option<usize>>,
    first_enabled: Option<usize>,
    last_enabled: Option<usize>,
    query: Option<String>,
    source: Option<String>,
    parse_count: u64,
    filter_rebuild_count: u64,
}

impl CommandCatalog {
    pub fn compile(value: &UiValue, generation: u64) -> Self {
        let mut entries = Vec::new();
        collect_entries(value, &mut entries);

        let mut seen_ids = HashSet::with_capacity(entries.len());
        entries.retain(|entry| seen_ids.insert(entry.id.clone()));
        let mut id_to_entry = HashMap::with_capacity(entries.len());
        for (index, entry) in entries.iter().enumerate() {
            id_to_entry.insert(entry.id.clone(), index);
        }

        let mut catalog = Self {
            generation,
            entries,
            id_to_entry,
            externally_disabled: HashSet::new(),
            filtered: Vec::new(),
            previous_enabled: Vec::new(),
            next_enabled: Vec::new(),
            first_enabled: None,
            last_enabled: None,
            query: Some("\0".to_string()),
            source: Some("\0".to_string()),
            parse_count: 1,
            filter_rebuild_count: 0,
        };
        let _ = catalog.filter(None, None);
        catalog
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn set_generation(&mut self, generation: u64) {
        self.generation = generation;
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub const fn parse_count(&self) -> u64 {
        self.parse_count
    }

    pub const fn filter_rebuild_count(&self) -> u64 {
        self.filter_rebuild_count
    }

    pub fn entry_index(&self, id: &str) -> Option<usize> {
        self.id_to_entry.get(id).copied()
    }

    pub fn filtered_ids(&self) -> impl Iterator<Item = &str> {
        self.filtered
            .iter()
            .map(|index| self.entries[*index].id.as_str())
    }

    pub fn filter(&mut self, query: Option<&str>, source: Option<&str>) -> bool {
        let query = normalize_optional(query);
        let source = normalize_optional(source);
        if self.query == query && self.source == source {
            return false;
        }

        self.filtered.clear();
        self.filtered.reserve(self.entries.len());
        for (index, entry) in self.entries.iter().enumerate() {
            if !matches_source(entry, source.as_deref()) {
                continue;
            }
            if query
                .as_deref()
                .is_some_and(|query| !entry.corpus.contains(query))
            {
                continue;
            }
            self.filtered.push(index);
        }
        self.query = query;
        self.source = source;
        self.filter_rebuild_count = self.filter_rebuild_count.saturating_add(1);
        self.rebuild_navigation();
        true
    }

    pub fn set_externally_disabled(&mut self, disabled: impl IntoIterator<Item = String>) -> bool {
        let disabled = disabled.into_iter().collect::<HashSet<_>>();
        if self.externally_disabled == disabled {
            return false;
        }
        self.externally_disabled = disabled;
        self.rebuild_navigation();
        true
    }

    pub fn move_focus(
        &self,
        current: Option<usize>,
        navigation: CatalogNavigation,
    ) -> Option<usize> {
        match navigation {
            CatalogNavigation::First => self.first_enabled,
            CatalogNavigation::Last => self.last_enabled,
            CatalogNavigation::Next => current
                .and_then(|index| self.next_enabled.get(index).copied().flatten())
                .or_else(|| current.filter(|index| self.is_enabled_filtered(*index)))
                .or(self.first_enabled),
            CatalogNavigation::Previous => current
                .and_then(|index| self.previous_enabled.get(index).copied().flatten())
                .or_else(|| current.filter(|index| self.is_enabled_filtered(*index)))
                .or(self.last_enabled),
        }
    }

    pub fn id_at_filtered_index(&self, index: usize) -> Option<&str> {
        self.filtered
            .get(index)
            .map(|entry| self.entries[*entry].id.as_str())
    }

    pub fn filtered_index_for_id(&self, id: &str) -> Option<usize> {
        let entry = self.entry_index(id)?;
        self.filtered
            .iter()
            .position(|candidate| *candidate == entry)
    }

    pub fn is_enabled_id(&self, id: &str) -> bool {
        self.id_to_entry
            .get(id)
            .and_then(|entry| self.entries.get(*entry))
            .is_some_and(|entry| !entry.disabled && !self.externally_disabled.contains(&entry.id))
    }

    pub fn is_enabled_filtered_index(&self, index: usize) -> bool {
        self.is_enabled_filtered(index)
    }

    pub fn first_enabled_index(&self) -> Option<usize> {
        self.first_enabled
    }

    fn rebuild_navigation(&mut self) {
        self.previous_enabled.clear();
        self.next_enabled.clear();
        self.previous_enabled.resize(self.filtered.len(), None);
        self.next_enabled.resize(self.filtered.len(), None);
        self.first_enabled = None;
        self.last_enabled = None;

        let mut previous = None;
        for index in 0..self.filtered.len() {
            if !self.is_enabled_filtered(index) {
                continue;
            }
            if self.first_enabled.is_none() {
                self.first_enabled = Some(index);
            }
            self.previous_enabled[index] = previous;
            if let Some(previous) = previous {
                self.next_enabled[previous] = Some(index);
            }
            previous = Some(index);
            self.last_enabled = Some(index);
        }
    }

    fn is_enabled_filtered(&self, index: usize) -> bool {
        let Some(entry) = self.filtered.get(index).map(|index| &self.entries[*index]) else {
            return false;
        };
        !entry.disabled && !self.externally_disabled.contains(&entry.id)
    }
}

fn collect_entries(value: &UiValue, entries: &mut Vec<CommandEntry>) {
    match value {
        UiValue::Array(values) => {
            for value in values {
                collect_entries(value, entries);
            }
        }
        UiValue::String(value) | UiValue::Enum(value) => {
            if let Some(entry) = parse_string_entry(value) {
                entries.push(entry);
            }
        }
        UiValue::Map(values) => {
            if let Some(entry) = parse_map_entry(values) {
                entries.push(entry);
            }
        }
        _ => {}
    }
}

fn parse_string_entry(value: &str) -> Option<CommandEntry> {
    let mut parts = value.split('|');
    let id = parts.next()?.trim();
    if id.is_empty() {
        return None;
    }

    let mut label = id.to_string();
    let mut source = String::new();
    let mut shortcut = String::new();
    let mut category = String::new();
    let mut keywords = String::new();
    let mut disabled = false;
    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "label" | "text" | "title" | "name" => label = value.to_string(),
            "source" | "command_source" | "commandSource" => source = value.to_string(),
            "shortcut" | "accelerator" | "keybinding" => shortcut = value.to_string(),
            "category" | "group" => category = value.to_string(),
            "keywords" | "keyword" => keywords = value.to_string(),
            "disabled" => disabled = matches!(value, "true" | "1" | "yes"),
            _ => {}
        }
    }

    Some(CommandEntry {
        id: id.to_string(),
        source: normalize(&source),
        corpus: normalized_corpus([id, &label, &source, &shortcut, &category, &keywords]),
        disabled,
    })
}

fn parse_map_entry(values: &std::collections::BTreeMap<String, UiValue>) -> Option<CommandEntry> {
    let id = first_string(values, &["id", "command_id", "commandId", "value", "key"])?;
    if id.is_empty() {
        return None;
    }
    let label =
        first_string(values, &["label", "text", "title", "name", "value_text"]).unwrap_or(id);
    let source =
        first_string(values, &["source", "command_source", "commandSource"]).unwrap_or_default();
    let shortcut =
        first_string(values, &["shortcut", "accelerator", "keybinding"]).unwrap_or_default();
    let category = first_string(values, &["category", "group"]).unwrap_or_default();
    let keywords = values
        .get("keywords")
        .or_else(|| values.get("keyword"))
        .map(normalized_value_text)
        .unwrap_or_default();
    let disabled = values.get("disabled").and_then(bool_value).unwrap_or(false)
        || values.get("enabled").and_then(bool_value) == Some(false);

    Some(CommandEntry {
        id: id.to_string(),
        source: normalize(source),
        corpus: normalized_corpus([id, label, source, shortcut, category, &keywords]),
        disabled,
    })
}

fn first_string<'a>(
    values: &'a std::collections::BTreeMap<String, UiValue>,
    keys: &[&str],
) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| values.get(*key).and_then(string_value))
        .filter(|value| !value.is_empty())
}

fn string_value(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value),
        _ => None,
    }
}

fn bool_value(value: &UiValue) -> Option<bool> {
    match value {
        UiValue::Bool(value) => Some(*value),
        _ => None,
    }
}

fn normalized_value_text(value: &UiValue) -> String {
    match value {
        UiValue::Array(values) => values
            .iter()
            .map(normalized_value_text)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        UiValue::Flags(values) => values.join(" "),
        UiValue::String(value) | UiValue::Enum(value) => value.clone(),
        _ => String::new(),
    }
}

fn normalize_optional(value: Option<&str>) -> Option<String> {
    value.map(normalize).filter(|value| !value.is_empty())
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn normalized_corpus<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .map(normalize)
        .collect::<Vec<_>>()
        .join("\u{1f}")
}

fn matches_source(entry: &CommandEntry, source: Option<&str>) -> bool {
    match source {
        Some(source) => entry.source.is_empty() || entry.source == source,
        None => true,
    }
}
