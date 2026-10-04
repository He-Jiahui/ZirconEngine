use std::collections::{HashMap, HashSet};

use zircon_runtime_interface::ui::component::UiValue;

#[derive(Clone, Debug)]
struct TreeEntry {
    id: String,
    label: String,
    parent: Option<usize>,
    children: Vec<usize>,
}

/// The visible-row change caused by one tree-model mutation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TreeRowDelta {
    pub changed: bool,
    pub generation: u64,
    pub first_changed_row: Option<usize>,
    pub removed_count: usize,
    pub added_count: usize,
}

/// A generation-owned, linearized tree projection.
#[derive(Clone, Debug)]
pub struct TreeIndex {
    generation: u64,
    entries: Vec<TreeEntry>,
    id_to_entry: HashMap<String, usize>,
    expanded: HashSet<String>,
    selected: HashSet<String>,
    disabled: HashSet<String>,
    visible: Vec<usize>,
    visible_row_by_entry: Vec<Option<usize>>,
    previous_enabled_visible: Vec<Option<usize>>,
    next_enabled_visible: Vec<Option<usize>>,
    first_enabled_visible: Option<usize>,
    last_enabled_visible: Option<usize>,
}

impl TreeIndex {
    pub fn compile(
        value: &UiValue,
        expanded: Option<&UiValue>,
        disabled: Option<&UiValue>,
        generation: u64,
    ) -> Self {
        let mut entries = Vec::new();
        let mut id_to_entry = HashMap::new();
        collect_entries(value, None, &mut entries, &mut id_to_entry);

        let mut index = Self {
            generation,
            visible_row_by_entry: vec![None; entries.len()],
            entries,
            id_to_entry,
            expanded: string_id_set(expanded),
            selected: HashSet::new(),
            disabled: string_id_set(disabled),
            visible: Vec::new(),
            previous_enabled_visible: Vec::new(),
            next_enabled_visible: Vec::new(),
            first_enabled_visible: None,
            last_enabled_visible: None,
        };
        index.rebuild_visible();
        index
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

    pub fn contains_id(&self, id: &str) -> bool {
        self.id_to_entry.contains_key(id)
    }

    pub fn visible_count(&self) -> usize {
        self.visible.len()
    }

    /// Returns the visible row for an id, if that node is currently visible.
    pub fn row_for_id(&self, id: &str) -> Option<usize> {
        let entry = *self.id_to_entry.get(id)?;
        self.visible_row_by_entry.get(entry).copied().flatten()
    }

    pub fn visible_id_at(&self, row: usize) -> Option<&str> {
        self.visible
            .get(row)
            .map(|entry| self.entries[*entry].id.as_str())
    }

    pub fn parent_of(&self, id: &str) -> Option<&str> {
        let entry = *self.id_to_entry.get(id)?;
        let parent = self.entries.get(entry)?.parent?;
        self.entries.get(parent).map(|entry| entry.id.as_str())
    }

    pub fn label_of(&self, id: &str) -> Option<&str> {
        self.id_to_entry
            .get(id)
            .and_then(|entry| self.entries.get(*entry))
            .map(|entry| entry.label.as_str())
    }

    pub fn is_disabled(&self, id: &str) -> bool {
        self.disabled.contains(id)
    }

    pub fn is_selected(&self, id: &str) -> bool {
        self.selected.contains(id)
    }

    pub(crate) fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn expanded_ids(&self) -> impl Iterator<Item = &str> {
        self.expanded.iter().map(String::as_str)
    }

    /// Returns expanded ids in source order so a serialized projection is
    /// deterministic even though membership is maintained as a hash set.
    pub fn expanded_ids_ordered(&self) -> impl Iterator<Item = &str> {
        self.entries
            .iter()
            .filter(|entry| self.expanded.contains(&entry.id))
            .map(|entry| entry.id.as_str())
    }

    pub fn selected_ids(&self) -> impl Iterator<Item = &str> {
        self.selected.iter().map(String::as_str)
    }

    pub fn selected_ids_ordered(&self) -> impl Iterator<Item = &str> {
        self.entries
            .iter()
            .filter(|entry| self.selected.contains(&entry.id))
            .map(|entry| entry.id.as_str())
    }

    pub fn visible_ids(&self) -> impl Iterator<Item = &str> {
        self.visible
            .iter()
            .map(|entry| self.entries[*entry].id.as_str())
    }

    pub fn set_selected(&mut self, id: &str, selected: bool) -> bool {
        if !self.id_to_entry.contains_key(id) {
            return false;
        }
        let changed = if selected {
            self.selected.insert(id.to_string())
        } else {
            self.selected.remove(id)
        };
        if changed {
            self.generation = self.generation.saturating_add(1);
        }
        changed
    }

    /// Seeds the selection set from the serializable control value without
    /// rebuilding the parsed tree. Unknown ids are ignored at the index
    /// boundary and remain available to the generic reducer if needed.
    pub(crate) fn sync_selected(&mut self, value: Option<&UiValue>) {
        let selected = string_id_set(value)
            .into_iter()
            .filter(|id| self.id_to_entry.contains_key(id))
            .collect();
        self.selected = selected;
    }

    pub fn first_enabled_visible_row(&self) -> Option<usize> {
        self.first_enabled_visible
    }

    pub fn last_enabled_visible_row(&self) -> Option<usize> {
        self.last_enabled_visible
    }

    pub fn next_enabled_visible_row(&self, row: usize, forward: bool) -> Option<usize> {
        if forward {
            self.next_enabled_visible.get(row).copied().flatten()
        } else {
            self.previous_enabled_visible.get(row).copied().flatten()
        }
    }

    pub fn set_disabled(&mut self, id: &str, disabled: bool) -> bool {
        if !self.id_to_entry.contains_key(id) {
            return false;
        }
        let changed = if disabled {
            self.disabled.insert(id.to_string())
        } else {
            self.disabled.remove(id)
        };
        if changed {
            self.generation = self.generation.saturating_add(1);
            self.rebuild_visible();
        }
        changed
    }

    pub fn set_expanded(&mut self, id: &str, expanded: bool) -> TreeRowDelta {
        if !self.id_to_entry.contains_key(id) {
            return TreeRowDelta {
                generation: self.generation,
                ..TreeRowDelta::default()
            };
        }
        let changed = if expanded {
            self.expanded.insert(id.to_string())
        } else {
            self.expanded.remove(id)
        };
        if !changed {
            return TreeRowDelta {
                generation: self.generation,
                ..TreeRowDelta::default()
            };
        }

        let previous = self.visible.clone();
        self.generation = self.generation.saturating_add(1);
        self.rebuild_visible();
        TreeRowDelta {
            changed: true,
            generation: self.generation,
            first_changed_row: first_difference(&previous, &self.visible),
            removed_count: previous.len().saturating_sub(self.visible.len()),
            added_count: self.visible.len().saturating_sub(previous.len()),
        }
    }

    pub fn next_visible_row(&self, row: usize) -> Option<usize> {
        (row + 1 < self.visible.len()).then_some(row + 1)
    }

    pub fn previous_visible_row(&self, row: usize) -> Option<usize> {
        row.checked_sub(1)
    }

    fn rebuild_visible(&mut self) {
        self.visible.clear();
        for entry in &mut self.visible_row_by_entry {
            *entry = None;
        }
        for root in 0..self.entries.len() {
            if self.entries[root].parent.is_none() {
                collect_visible(&self.entries, &self.expanded, root, &mut self.visible);
            }
        }
        for (row, entry) in self.visible.iter().copied().enumerate() {
            self.visible_row_by_entry[entry] = Some(row);
        }

        self.previous_enabled_visible.clear();
        self.previous_enabled_visible
            .resize(self.visible.len(), None);
        self.next_enabled_visible.clear();
        self.next_enabled_visible.resize(self.visible.len(), None);
        self.first_enabled_visible = None;
        self.last_enabled_visible = None;
        let mut previous = None;
        for row in 0..self.visible.len() {
            let entry = self.visible[row];
            self.previous_enabled_visible[row] = previous;
            if !self.disabled.contains(&self.entries[entry].id) {
                if self.first_enabled_visible.is_none() {
                    self.first_enabled_visible = Some(row);
                }
                previous = Some(row);
                self.last_enabled_visible = Some(row);
            }
        }
        let mut next = None;
        for row in (0..self.visible.len()).rev() {
            let entry = self.visible[row];
            self.next_enabled_visible[row] = next;
            if !self.disabled.contains(&self.entries[entry].id) {
                next = Some(row);
            }
        }
    }
}

fn collect_entries(
    value: &UiValue,
    parent: Option<usize>,
    entries: &mut Vec<TreeEntry>,
    id_to_entry: &mut HashMap<String, usize>,
) {
    match value {
        UiValue::Array(values) => {
            for value in values {
                collect_entries(value, parent, entries, id_to_entry);
            }
        }
        UiValue::String(value) | UiValue::Enum(value) => {
            insert_entry(value, value, parent, entries, id_to_entry);
        }
        UiValue::Map(values) => {
            let Some(id) = map_id(values) else {
                for field in ["children", "nodes", "items", "options"] {
                    if let Some(child) = values.get(field) {
                        collect_entries(child, parent, entries, id_to_entry);
                    }
                }
                return;
            };
            let label = map_label(values).unwrap_or(id);
            let entry = insert_entry(id, label, parent, entries, id_to_entry);
            for field in ["children", "nodes", "items", "options"] {
                if let Some(child) = values.get(field) {
                    collect_entries(child, entry, entries, id_to_entry);
                }
            }
        }
        _ => {}
    }
}

fn insert_entry(
    id: &str,
    label: &str,
    parent: Option<usize>,
    entries: &mut Vec<TreeEntry>,
    id_to_entry: &mut HashMap<String, usize>,
) -> Option<usize> {
    if id.is_empty() {
        return parent;
    }
    if let Some(existing) = id_to_entry.get(id).copied() {
        return Some(existing);
    }
    let entry = entries.len();
    entries.push(TreeEntry {
        id: id.to_string(),
        label: label.to_string(),
        parent,
        children: Vec::new(),
    });
    id_to_entry.insert(id.to_string(), entry);
    if let Some(parent) = parent {
        entries[parent].children.push(entry);
    }
    Some(entry)
}

fn collect_visible(
    entries: &[TreeEntry],
    expanded: &HashSet<String>,
    entry: usize,
    visible: &mut Vec<usize>,
) {
    visible.push(entry);
    if !expanded.contains(&entries[entry].id) {
        return;
    }
    for child in &entries[entry].children {
        collect_visible(entries, expanded, *child, visible);
    }
}

fn map_id(values: &std::collections::BTreeMap<String, UiValue>) -> Option<&str> {
    ["id", "value", "row_id", "rowId", "node_id", "nodeId", "key"]
        .into_iter()
        .find_map(|field| values.get(field).and_then(string_value))
        .filter(|value| !value.is_empty())
}

fn map_label(values: &std::collections::BTreeMap<String, UiValue>) -> Option<&str> {
    ["label", "text", "title", "name", "value_text"]
        .into_iter()
        .find_map(|field| values.get(field).and_then(string_value))
        .filter(|value| !value.is_empty())
}

fn string_value(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value),
        _ => None,
    }
}

fn string_id_set(value: Option<&UiValue>) -> HashSet<String> {
    let mut ids = HashSet::new();
    if let Some(value) = value {
        collect_string_ids(value, &mut ids);
    }
    ids
}

fn collect_string_ids(value: &UiValue, ids: &mut HashSet<String>) {
    match value {
        UiValue::Array(values) => {
            for value in values {
                collect_string_ids(value, ids);
            }
        }
        UiValue::String(value) | UiValue::Enum(value) => {
            if !value.is_empty() {
                ids.insert(value.to_string());
            }
        }
        UiValue::Flags(values) => {
            ids.extend(values.iter().filter(|value| !value.is_empty()).cloned());
        }
        UiValue::Map(values) => {
            if let Some(id) = map_id(values) {
                ids.insert(id.to_string());
            }
        }
        _ => {}
    }
}

fn first_difference(left: &[usize], right: &[usize]) -> Option<usize> {
    let common = left.len().min(right.len());
    left.iter()
        .zip(right)
        .position(|(left, right)| left != right)
        .or_else(|| (left.len() != right.len()).then_some(common))
}
