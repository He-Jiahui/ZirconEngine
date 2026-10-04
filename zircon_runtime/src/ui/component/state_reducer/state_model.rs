//! Persistent, runtime-owned indexes for component state.
//!
//! `UiComponentState` remains the serializable authoring/projection DTO.  This
//! module owns the compiled command, tree, and table views used by repeated
//! reducer actions and exposes one transaction boundary for state writes.

use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::component::{
    UiComponentDescriptor, UiComponentEvent, UiComponentEventError, UiComponentFlags,
    UiComponentKeyboardAction, UiComponentState, UiDragSourceMetadata, UiValidationState, UiValue,
};

mod command_catalog;
mod table_index;
mod tree_index;

pub use command_catalog::{CatalogNavigation, CommandCatalog};
pub use table_index::{TableIndex, TableSortResult};
pub use tree_index::{TreeIndex, TreeRowDelta};

/// A compact receipt for one component-state transaction.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiComponentStateChange {
    pub generation: u64,
    pub changed_fields: Vec<String>,
    pub flags_changed: bool,
    pub reference_sources_changed: bool,
}

/// One atomic mutation of a serializable component state.
///
/// The runtime keeps aliases at the projection boundary. Callers use canonical
/// field identities here so a logical action cannot create competing state
/// authorities for the same property.
#[derive(Clone, Debug, Default)]
pub struct UiComponentStatePatch {
    values: BTreeMap<String, UiValue>,
    reference_sources: BTreeMap<String, Option<UiDragSourceMetadata>>,
    flags: Option<UiComponentFlags>,
}

impl UiComponentStatePatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_value(&mut self, field: impl AsRef<str>, value: UiValue) -> &mut Self {
        self.values
            .insert(canonical_field(field.as_ref()).to_string(), value);
        self
    }

    pub fn set_reference_source(
        &mut self,
        field: impl AsRef<str>,
        source: UiDragSourceMetadata,
    ) -> &mut Self {
        self.reference_sources
            .insert(canonical_field(field.as_ref()).to_string(), Some(source));
        self
    }

    pub fn clear_reference_source(&mut self, field: impl AsRef<str>) -> &mut Self {
        self.reference_sources
            .insert(canonical_field(field.as_ref()).to_string(), None);
        self
    }

    pub fn set_flags(&mut self, flags: UiComponentFlags) -> &mut Self {
        self.flags = Some(flags);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty() && self.reference_sources.is_empty() && self.flags.is_none()
    }

    pub fn commit(
        self,
        state: &mut UiComponentState,
        generation: &mut u64,
    ) -> UiComponentStateChange {
        let mut changed_fields = BTreeSet::new();
        let mut reference_sources_changed = false;

        for (field, value) in self.values {
            if state.values.get(&field) == Some(&value) {
                continue;
            }
            state.values.insert(field.clone(), value);
            if state.reference_sources.remove(&field).is_some() {
                reference_sources_changed = true;
            }
            changed_fields.insert(field);
        }

        for (field, source) in self.reference_sources {
            let changed = match source {
                Some(source) => {
                    if state.reference_sources.get(&field) == Some(&source) {
                        false
                    } else {
                        state.reference_sources.insert(field.clone(), source);
                        true
                    }
                }
                None => state.reference_sources.remove(&field).is_some(),
            };
            if changed {
                reference_sources_changed = true;
                changed_fields.insert(field);
            }
        }

        let flags_changed = self
            .flags
            .as_ref()
            .is_some_and(|flags| flags != &state.flags);
        if let Some(flags) = self.flags.filter(|flags| *flags != state.flags) {
            state.flags = flags;
        }

        if !changed_fields.is_empty() || flags_changed {
            *generation = generation.saturating_add(1);
        }

        UiComponentStateChange {
            generation: *generation,
            changed_fields: changed_fields.into_iter().collect(),
            flags_changed,
            reference_sources_changed,
        }
    }
}

/// Runtime-owned component state and persistent derived indexes.
///
/// `UiComponentState` intentionally remains a serde-compatible projection
/// type. This model owns transient compiled state for repeat interactions and
/// invalidates only the affected derived view after an atomic patch.
#[derive(Clone, Debug)]
pub struct UiComponentStateModel {
    state: UiComponentState,
    generation: u64,
    command_catalog: Option<CommandCatalog>,
    tree_index: Option<TreeIndex>,
    table_index: Option<TableIndex>,
    command_catalog_dirty: bool,
    command_projection_dirty: bool,
    tree_index_dirty: bool,
    table_index_dirty: bool,
}

impl Default for UiComponentStateModel {
    fn default() -> Self {
        Self::new(UiComponentState::new())
    }
}

impl UiComponentStateModel {
    pub fn new(state: UiComponentState) -> Self {
        Self {
            state,
            generation: 0,
            command_catalog: None,
            tree_index: None,
            table_index: None,
            command_catalog_dirty: true,
            command_projection_dirty: true,
            tree_index_dirty: true,
            table_index_dirty: true,
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn state(&self) -> &UiComponentState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut UiComponentState {
        &mut self.state
    }

    pub fn into_state(self) -> UiComponentState {
        self.state
    }

    pub fn replace_state(&mut self, state: UiComponentState) -> bool {
        if self.state == state {
            return false;
        }
        self.state = state;
        self.generation = self.generation.saturating_add(1);
        self.invalidate_all_indexes();
        true
    }

    pub fn apply_patch(&mut self, patch: UiComponentStatePatch) -> UiComponentStateChange {
        let change = patch.commit(&mut self.state, &mut self.generation);
        self.invalidate_indexes_for_fields(&change.changed_fields);
        self.retag_live_indexes();
        change
    }

    pub fn refresh_command_projection(
        &mut self,
        query: Option<&str>,
        source: Option<&str>,
        disabled: impl IntoIterator<Item = String>,
    ) -> bool {
        let catalog = self.command_catalog_mut();
        let filter_changed = catalog.filter(query, source);
        let disabled_changed = catalog.set_externally_disabled(disabled);
        filter_changed || disabled_changed
    }

    pub fn apply_tree_expansion(&mut self, id: &str, expanded: bool) -> TreeRowDelta {
        let delta = self.tree_index_mut().set_expanded(id, expanded);
        if delta.changed {
            self.generation = delta.generation;
        }
        delta
    }

    /// Compatibility path for callers that still provide the legacy event DTO.
    /// New high-frequency consumers should use the typed derived indexes below.
    pub fn apply_event(
        &mut self,
        descriptor: &UiComponentDescriptor,
        event: UiComponentEvent,
    ) -> Result<UiComponentStateChange, UiComponentEventError> {
        super::ensure_event_supported(descriptor, event.kind())?;
        if let Some(change) = self.apply_indexed_event(descriptor, &event)? {
            return Ok(change);
        }
        let invalidation = EventInvalidation::from_event(&event);
        let changed_fields = event_fields(&event);
        super::apply_component_event(&mut self.state, descriptor, event)?;
        if !changed_fields.is_empty() {
            self.generation = self.generation.saturating_add(1);
        }
        if invalidation.command_catalog {
            self.command_catalog_dirty = true;
        }
        if invalidation.command_projection {
            self.command_projection_dirty = true;
        }
        if invalidation.tree_index {
            self.tree_index_dirty = true;
        }
        if invalidation.table_index {
            self.table_index_dirty = true;
        }
        Ok(UiComponentStateChange {
            generation: self.generation,
            changed_fields,
            flags_changed: false,
            reference_sources_changed: false,
        })
    }

    fn apply_indexed_event(
        &mut self,
        descriptor: &UiComponentDescriptor,
        event: &UiComponentEvent,
    ) -> Result<Option<UiComponentStateChange>, UiComponentEventError> {
        if is_command_palette(descriptor) {
            if let Some(change) = self.apply_command_indexed_event(descriptor, event)? {
                return Ok(Some(change));
            }
        }
        if is_tree_view(descriptor) {
            if let Some(change) = self.apply_tree_indexed_event(descriptor, event)? {
                return Ok(Some(change));
            }
        }
        if is_table(descriptor) {
            if let Some(change) = self.apply_table_indexed_event(descriptor, event)? {
                return Ok(Some(change));
            }
        }
        Ok(None)
    }

    fn apply_tree_indexed_event(
        &mut self,
        descriptor: &UiComponentDescriptor,
        event: &UiComponentEvent,
    ) -> Result<Option<UiComponentStateChange>, UiComponentEventError> {
        match event {
            UiComponentEvent::ToggleExpanded { expanded } => {
                self.apply_tree_expanded_event(descriptor, *expanded)
            }
            UiComponentEvent::KeyboardAction { action }
                if matches!(
                    action,
                    UiComponentKeyboardAction::Increment | UiComponentKeyboardAction::Decrement
                ) =>
            {
                self.apply_tree_expanded_event(
                    descriptor,
                    matches!(action, UiComponentKeyboardAction::Increment),
                )
            }
            UiComponentEvent::KeyboardAction { action }
                if matches!(
                    action,
                    UiComponentKeyboardAction::Next
                        | UiComponentKeyboardAction::Previous
                        | UiComponentKeyboardAction::First
                        | UiComponentKeyboardAction::Last
                ) =>
            {
                self.apply_tree_navigation_event(descriptor, *action)
            }
            UiComponentEvent::SelectOption {
                property,
                option_id,
                selected,
            } => self.apply_tree_selection_event(descriptor, property, option_id, *selected),
            _ => Ok(None),
        }
    }

    fn apply_tree_expanded_event(
        &mut self,
        descriptor: &UiComponentDescriptor,
        expanded: bool,
    ) -> Result<Option<UiComponentStateChange>, UiComponentEventError> {
        let Some(node_id) = self.tree_focus_id(descriptor) else {
            return Ok(None);
        };
        let delta = self.tree_index_mut().set_expanded(&node_id, expanded);
        if !delta.changed {
            return Ok(Some(noop_change(self.generation)));
        }

        let expanded_ids = self
            .tree_index()
            .expanded_ids_ordered()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let has_expanded_nodes = !expanded_ids.is_empty();
        let mut patch = UiComponentStatePatch::new();
        patch.set_value(
            tree_expanded_property(&self.state, descriptor),
            UiValue::Array(expanded_ids.into_iter().map(UiValue::String).collect()),
        );
        patch.set_value("expanded", UiValue::Bool(has_expanded_nodes));
        let mut flags = self.state.flags.clone();
        flags.expanded = has_expanded_nodes;
        patch.set_flags(flags);
        Ok(Some(self.apply_patch(patch)))
    }

    fn apply_tree_navigation_event(
        &mut self,
        descriptor: &UiComponentDescriptor,
        action: UiComponentKeyboardAction,
    ) -> Result<Option<UiComponentStateChange>, UiComponentEventError> {
        if !bool_setting_model(&self.state, descriptor, "keyboard_navigation", true) {
            return Ok(None);
        }

        let current = self.tree_current_row(descriptor);
        let wrap = !bool_setting_model(&self.state, descriptor, "disableListWrap", false);
        let target = {
            let tree = self.tree_index();
            let target = match action {
                UiComponentKeyboardAction::First => tree.first_enabled_visible_row(),
                UiComponentKeyboardAction::Last => tree.last_enabled_visible_row(),
                UiComponentKeyboardAction::Next => current
                    .and_then(|row| tree.next_enabled_visible_row(row, true))
                    .or_else(|| {
                        current
                            .is_none()
                            .then(|| tree.first_enabled_visible_row())
                            .flatten()
                    }),
                UiComponentKeyboardAction::Previous => current
                    .and_then(|row| tree.next_enabled_visible_row(row, false))
                    .or_else(|| {
                        current
                            .is_none()
                            .then(|| tree.last_enabled_visible_row())
                            .flatten()
                    }),
                _ => None,
            };
            match (target, action, wrap) {
                (Some(target), _, _) => Some(target),
                (None, UiComponentKeyboardAction::Next, true) => tree.first_enabled_visible_row(),
                (None, UiComponentKeyboardAction::Previous, true) => {
                    tree.last_enabled_visible_row()
                }
                _ => None,
            }
        };
        let Some(target) = target else {
            return Ok(Some(noop_change(self.generation)));
        };
        let Some(node_id) = self.tree_index().visible_id_at(target).map(str::to_owned) else {
            return Ok(Some(noop_change(self.generation)));
        };

        let follows_selection =
            bool_setting_model(&self.state, descriptor, "selection_follows_focus", false)
                || bool_setting_model(&self.state, descriptor, "selectionFollowsFocus", false);
        let mut patch = UiComponentStatePatch::new();
        patch.set_value(
            "focused_index",
            UiValue::Int(i64::try_from(target).unwrap_or(i64::MAX)),
        );
        if follows_selection {
            patch.set_value(
                "selected_index",
                UiValue::Int(i64::try_from(target).unwrap_or(i64::MAX)),
            );
            patch.set_value("value", UiValue::String(node_id.clone()));
        }
        let mut flags = self.state.flags.clone();
        flags.focused = true;
        if follows_selection {
            flags.selected = true;
        }
        patch.set_flags(flags);
        let change = self.apply_patch(patch);
        if follows_selection {
            self.tree_index_mut().set_selected(&node_id, true);
            self.retag_live_indexes();
        }
        Ok(Some(change))
    }

    fn apply_tree_selection_event(
        &mut self,
        descriptor: &UiComponentDescriptor,
        property: &str,
        option_id: &str,
        selected: bool,
    ) -> Result<Option<UiComponentStateChange>, UiComponentEventError> {
        let (exists, disabled, row) = {
            let tree = self.tree_index();
            (
                tree.contains_id(option_id),
                tree.is_disabled(option_id),
                tree.row_for_id(option_id),
            )
        };
        if !exists || row.is_none() {
            return Ok(None);
        }
        if disabled && selected {
            self.state.validation = UiValidationState::error(format!(
                "disabled tree node `{option_id}` cannot be selected"
            ));
            return Err(UiComponentEventError::DisabledOption {
                component_id: descriptor.id.clone(),
                option_id: option_id.to_string(),
            });
        }
        if tree_range_selecting_model(&self.state, descriptor) {
            return Ok(None);
        }

        let multi = is_selected_control_property_model(property)
            || bool_setting_model(&self.state, descriptor, "multi_select", false)
            || bool_setting_model(&self.state, descriptor, "multiSelect", false)
            || bool_setting_model(&self.state, descriptor, "checkboxSelection", false);
        let mut patch = UiComponentStatePatch::new();
        let mut next_selected = Vec::new();
        if multi {
            let selected_value = ["selected_items", "selectedItems"]
                .into_iter()
                .find_map(|field| self.state.values.get(field).cloned());
            self.tree_index_mut().sync_selected(selected_value.as_ref());
            self.tree_index_mut().set_selected(option_id, selected);
            next_selected = Vec::with_capacity(self.tree_index().selected_count());
            next_selected.extend(self.tree_index().selected_ids_ordered().map(str::to_owned));
            patch.set_value(
                if is_selected_control_property_model(property) {
                    property
                } else {
                    tree_selected_property(&self.state, descriptor)
                },
                UiValue::Array(next_selected.iter().cloned().map(UiValue::String).collect()),
            );
        } else {
            let value_property = if property.is_empty() {
                "value"
            } else {
                property
            };
            patch.set_value(
                value_property,
                if selected {
                    UiValue::String(option_id.to_string())
                } else {
                    UiValue::Null
                },
            );
            self.tree_index_mut().sync_selected(None);
            self.tree_index_mut().set_selected(option_id, selected);
        }

        let row = row.expect("tree row was checked above");
        patch.set_value(
            "focused_index",
            UiValue::Int(i64::try_from(row).unwrap_or(i64::MAX)),
        );
        patch.set_value(
            "selected_index",
            UiValue::Int(i64::try_from(row).unwrap_or(i64::MAX)),
        );
        patch.set_value(
            "selection_anchor_index",
            UiValue::Int(i64::try_from(row).unwrap_or(i64::MAX)),
        );
        let mut flags = self.state.flags.clone();
        flags.focused = true;
        flags.selected = if multi {
            !next_selected.is_empty()
        } else {
            selected
        };
        patch.set_flags(flags);
        Ok(Some(self.apply_patch(patch)))
    }

    fn tree_focus_id(&mut self, descriptor: &UiComponentDescriptor) -> Option<String> {
        let current = int_setting_model(&self.state, descriptor, "focused_index")
            .or_else(|| int_setting_model(&self.state, descriptor, "selected_index"))
            .and_then(|index| usize::try_from(index).ok());
        let value_id = ["value", "value_text", "group_value"]
            .into_iter()
            .find_map(|field| self.state.values.get(field).and_then(string_value_ref))
            .map(str::to_owned);
        let tree = self.tree_index();
        current
            .and_then(|row| tree.visible_id_at(row))
            .map(str::to_owned)
            .or_else(|| value_id.and_then(|id| tree.row_for_id(&id).map(|_| id)))
    }

    fn tree_current_row(&mut self, descriptor: &UiComponentDescriptor) -> Option<usize> {
        let current = int_setting_model(&self.state, descriptor, "focused_index")
            .or_else(|| int_setting_model(&self.state, descriptor, "selected_index"))
            .and_then(|index| usize::try_from(index).ok());
        let tree = self.tree_index();
        current.filter(|row| *row < tree.visible_count())
    }

    fn apply_table_indexed_event(
        &mut self,
        descriptor: &UiComponentDescriptor,
        event: &UiComponentEvent,
    ) -> Result<Option<UiComponentStateChange>, UiComponentEventError> {
        let (property, value) = match event {
            UiComponentEvent::ValueChanged { property, value }
                if matches!(canonical_field(property), "sort_column" | "sort_direction") =>
            {
                (property.as_str(), value)
            }
            _ => return Ok(None),
        };
        let canonical = canonical_field(property);
        let Some(value) = string_value_ref(value) else {
            return Ok(None);
        };
        let (column, direction) = if canonical == "sort_column" {
            let current_column = state_string_setting(&self.state, "sort_column");
            let current_direction = state_string_setting(&self.state, "sort_direction")
                .as_deref()
                .and_then(normalize_sort_direction_model)
                .unwrap_or("none");
            let direction = if current_column.as_deref() == Some(value) {
                if current_direction == "desc" {
                    "asc"
                } else {
                    "desc"
                }
            } else if value.is_empty() {
                "none"
            } else {
                "asc"
            };
            (value.to_string(), direction)
        } else {
            let Some(column) = state_string_setting(&self.state, "sort_column") else {
                return Ok(None);
            };
            let Some(direction) = normalize_sort_direction_model(value) else {
                return Ok(None);
            };
            (column, direction)
        };
        Ok(Some(
            self.apply_table_sort_event(descriptor, &column, direction),
        ))
    }

    fn apply_table_sort_event(
        &mut self,
        descriptor: &UiComponentDescriptor,
        column: &str,
        direction: &str,
    ) -> UiComponentStateChange {
        let table_was_compiled = self.table_index.is_some() && !self.table_index_dirty;
        let mut patch = UiComponentStatePatch::new();
        let (column, direction) = if column.is_empty() || direction == "none" {
            ("", "none")
        } else {
            (column, direction)
        };
        patch.set_value("sort_column", UiValue::String(column.to_string()));
        patch.set_value("sort_direction", UiValue::String(direction.to_string()));
        if descriptor.prop("sortModel").is_some() || self.state.values.contains_key("sortModel") {
            let sort_model = if column.is_empty() {
                UiValue::Array(Vec::new())
            } else {
                let mut entry = BTreeMap::new();
                entry.insert("field".to_string(), UiValue::String(column.to_string()));
                entry.insert("sort".to_string(), UiValue::String(direction.to_string()));
                UiValue::Array(vec![UiValue::Map(entry)])
            };
            patch.set_value("sortModel", sort_model);
        }

        let rows = self.state.values.get("rows").cloned();
        if !column.is_empty()
            && table_uses_client_sorting_model(&self.state, descriptor)
            && rows.is_some()
        {
            let result = {
                let index = self.table_index();
                index.sort_permutation(
                    rows.as_ref().expect("rows checked above"),
                    column,
                    direction,
                )
            };
            let sort_generation = self.generation;
            let sort_accepted = self
                .table_index()
                .accept_sort_result(result.clone(), sort_generation)
                .is_some();
            if sort_accepted {
                if let Some(reordered) = reorder_table_rows(
                    rows.as_ref().expect("rows checked above"),
                    &result.permutation,
                ) {
                    patch.set_value("rows", reordered);
                }
            }
        }
        let change = self.apply_patch(patch);
        if table_was_compiled {
            self.table_index_dirty = false;
            self.retag_live_indexes();
        }
        change
    }

    fn apply_command_indexed_event(
        &mut self,
        descriptor: &UiComponentDescriptor,
        event: &UiComponentEvent,
    ) -> Result<Option<UiComponentStateChange>, UiComponentEventError> {
        match event {
            UiComponentEvent::ValueChanged { property, value }
                if matches!(
                    property.as_str(),
                    "commands" | "query" | "command_source" | "disabled_commands"
                ) =>
            {
                let mut patch = UiComponentStatePatch::new();
                patch.set_value(property, value.clone());
                let mut change = self.apply_patch(patch);
                change = merge_changes(change, self.sync_command_projection());
                Ok(Some(change))
            }
            UiComponentEvent::KeyboardText { text } => {
                let filtered = text
                    .chars()
                    .filter(|character| !character.is_control())
                    .collect::<String>();
                if filtered.is_empty() {
                    return Ok(Some(UiComponentStateChange {
                        generation: self.generation,
                        ..UiComponentStateChange::default()
                    }));
                }
                let mut query = self
                    .state
                    .values
                    .get("query")
                    .and_then(string_value_ref)
                    .unwrap_or_default()
                    .to_string();
                query.push_str(&filtered);
                let mut patch = UiComponentStatePatch::new();
                patch.set_value("query", UiValue::String(query));
                let mut change = self.apply_patch(patch);
                change = merge_changes(change, self.sync_command_projection());
                Ok(Some(change))
            }
            UiComponentEvent::KeyboardAction { action }
                if matches!(
                    action,
                    UiComponentKeyboardAction::Next
                        | UiComponentKeyboardAction::Previous
                        | UiComponentKeyboardAction::First
                        | UiComponentKeyboardAction::Last
                ) =>
            {
                let mut change = self.sync_command_projection_if_dirty();
                let current = int_value(self.state.values.get("focused_index"))
                    .and_then(|value| usize::try_from(value).ok());
                let navigation = match action {
                    UiComponentKeyboardAction::Next => CatalogNavigation::Next,
                    UiComponentKeyboardAction::Previous => CatalogNavigation::Previous,
                    UiComponentKeyboardAction::First => CatalogNavigation::First,
                    UiComponentKeyboardAction::Last => CatalogNavigation::Last,
                    _ => unreachable!("guarded command navigation action"),
                };
                let target = self.command_catalog().move_focus(current, navigation);
                let mut patch = UiComponentStatePatch::new();
                if let Some(index) = target {
                    if let Some(id) = self.command_catalog().id_at_filtered_index(index) {
                        patch.set_value("selected_command_id", UiValue::String(id.to_string()));
                        patch.set_value(
                            "focused_index",
                            UiValue::Int(i64::try_from(index).unwrap_or(i64::MAX)),
                        );
                    }
                } else {
                    patch.set_value("selected_command_id", UiValue::String(String::new()));
                    patch.set_value("focused_index", UiValue::Int(-1));
                }
                let mut flags = self.state.flags.clone();
                flags.focused = target.is_some();
                flags.selected = target.is_some();
                patch.set_flags(flags);
                change = merge_changes(change, self.apply_patch(patch));
                Ok(Some(change))
            }
            UiComponentEvent::SelectOption {
                option_id,
                selected,
                ..
            } => {
                let _ = self.sync_command_projection_if_dirty();
                if *selected && !self.command_catalog().is_enabled_id(option_id) {
                    self.state.validation = UiValidationState::error(format!(
                        "disabled command `{option_id}` cannot be selected"
                    ));
                    return Err(UiComponentEventError::DisabledOption {
                        component_id: descriptor.id.clone(),
                        option_id: option_id.clone(),
                    });
                }
                let index = if *selected {
                    self.command_catalog()
                        .filtered_index_for_id(option_id)
                        .or_else(|| self.command_catalog().entry_index(option_id))
                        .map(|index| i64::try_from(index).unwrap_or(i64::MAX))
                        .unwrap_or(-1)
                } else {
                    -1
                };
                let mut patch = UiComponentStatePatch::new();
                patch.set_value(
                    "selected_command_id",
                    UiValue::String(if *selected {
                        option_id.clone()
                    } else {
                        String::new()
                    }),
                );
                patch.set_value("focused_index", UiValue::Int(index));
                let mut flags = self.state.flags.clone();
                flags.focused = *selected;
                flags.selected = *selected;
                patch.set_flags(flags);
                Ok(Some(self.apply_patch(patch)))
            }
            _ => Ok(None),
        }
    }

    fn sync_command_projection(&mut self) -> UiComponentStateChange {
        let query = self
            .state
            .values
            .get("query")
            .and_then(string_value_ref)
            .map(str::to_string);
        let source = self
            .state
            .values
            .get("command_source")
            .and_then(string_value_ref)
            .map(str::to_string);
        let disabled = self
            .state
            .values
            .get("disabled_commands")
            .map(string_ids)
            .unwrap_or_default();
        let selected = self
            .state
            .values
            .get("selected_command_id")
            .and_then(string_value_ref)
            .map(str::to_string);
        let current = int_value(self.state.values.get("focused_index"))
            .and_then(|value| usize::try_from(value).ok());

        let (filtered, focus) = {
            let catalog = self.command_catalog_mut();
            catalog.filter(query.as_deref(), source.as_deref());
            catalog.set_externally_disabled(disabled);
            let selected_index = selected
                .as_deref()
                .and_then(|id| catalog.filtered_index_for_id(id))
                .filter(|index| catalog.is_enabled_filtered_index(*index));
            let current_index = current.filter(|index| {
                catalog
                    .id_at_filtered_index(*index)
                    .is_some_and(|_| catalog.is_enabled_filtered_index(*index))
            });
            let focus = selected_index
                .or(current_index)
                .or_else(|| catalog.first_enabled_index());
            let filtered = catalog
                .filtered_ids()
                .map(str::to_string)
                .collect::<Vec<_>>();
            (filtered, focus)
        };

        let selected = focus
            .and_then(|index| filtered.get(index))
            .cloned()
            .unwrap_or_default();
        let focused_index = focus
            .map(|index| i64::try_from(index).unwrap_or(i64::MAX))
            .unwrap_or(-1);
        let mut patch = UiComponentStatePatch::new();
        patch.set_value(
            "filtered_commands",
            UiValue::Array(filtered.into_iter().map(UiValue::String).collect()),
        );
        patch.set_value("selected_command_id", UiValue::String(selected));
        patch.set_value("focused_index", UiValue::Int(focused_index));
        let mut flags = self.state.flags.clone();
        flags.focused = focus.is_some();
        patch.set_flags(flags);
        let change = self.apply_patch(patch);
        self.command_projection_dirty = false;
        change
    }

    fn sync_command_projection_if_dirty(&mut self) -> UiComponentStateChange {
        if self.command_projection_dirty {
            self.sync_command_projection()
        } else {
            noop_change(self.generation)
        }
    }

    pub fn command_catalog(&mut self) -> &CommandCatalog {
        if self.command_catalog_dirty || self.command_catalog.is_none() {
            let catalog = {
                let source = self.state.values.get("commands").unwrap_or(&UiValue::Null);
                CommandCatalog::compile(source, self.generation)
            };
            self.command_catalog = Some(catalog);
            self.command_catalog_dirty = false;
        }
        self.command_catalog
            .as_ref()
            .expect("command catalog was rebuilt before access")
    }

    pub fn command_catalog_mut(&mut self) -> &mut CommandCatalog {
        if self.command_catalog_dirty || self.command_catalog.is_none() {
            let catalog = {
                let source = self.state.values.get("commands").unwrap_or(&UiValue::Null);
                CommandCatalog::compile(source, self.generation)
            };
            self.command_catalog = Some(catalog);
            self.command_catalog_dirty = false;
        }
        self.command_catalog
            .as_mut()
            .expect("command catalog was rebuilt before mutable access")
    }

    pub fn tree_index(&mut self) -> &TreeIndex {
        if self.tree_index_dirty || self.tree_index.is_none() {
            let tree = {
                let source = ["nodes", "items", "options"]
                    .into_iter()
                    .find_map(|field| self.state.values.get(field))
                    .unwrap_or(&UiValue::Null);
                let expanded = ["expanded_items", "expandedItems"]
                    .into_iter()
                    .find_map(|field| self.state.values.get(field));
                let disabled = self.state.values.get("disabled_options");
                let mut tree = TreeIndex::compile(source, expanded, disabled, self.generation);
                let selected = ["selected_items", "selectedItems"]
                    .into_iter()
                    .find_map(|field| self.state.values.get(field));
                tree.sync_selected(selected);
                tree
            };
            self.tree_index = Some(tree);
            self.tree_index_dirty = false;
        }
        self.tree_index
            .as_ref()
            .expect("tree index was rebuilt before access")
    }

    pub fn tree_index_mut(&mut self) -> &mut TreeIndex {
        if self.tree_index_dirty || self.tree_index.is_none() {
            let tree = {
                let source = ["nodes", "items", "options"]
                    .into_iter()
                    .find_map(|field| self.state.values.get(field))
                    .unwrap_or(&UiValue::Null);
                let expanded = ["expanded_items", "expandedItems"]
                    .into_iter()
                    .find_map(|field| self.state.values.get(field));
                let disabled = self.state.values.get("disabled_options");
                let mut tree = TreeIndex::compile(source, expanded, disabled, self.generation);
                let selected = ["selected_items", "selectedItems"]
                    .into_iter()
                    .find_map(|field| self.state.values.get(field));
                tree.sync_selected(selected);
                tree
            };
            self.tree_index = Some(tree);
            self.tree_index_dirty = false;
        }
        self.tree_index
            .as_mut()
            .expect("tree index was rebuilt before mutable access")
    }

    pub fn table_index(&mut self) -> &TableIndex {
        if self.table_index_dirty || self.table_index.is_none() {
            let index = TableIndex::compile(
                self.state.values.get("columns"),
                self.state.values.get("rows"),
                self.generation,
            );
            self.table_index = Some(index);
            self.table_index_dirty = false;
        }
        self.table_index
            .as_ref()
            .expect("table index was rebuilt before access")
    }

    pub fn table_index_mut(&mut self) -> &mut TableIndex {
        if self.table_index_dirty || self.table_index.is_none() {
            let index = TableIndex::compile(
                self.state.values.get("columns"),
                self.state.values.get("rows"),
                self.generation,
            );
            self.table_index = Some(index);
            self.table_index_dirty = false;
        }
        self.table_index
            .as_mut()
            .expect("table index was rebuilt before mutable access")
    }

    fn invalidate_all_indexes(&mut self) {
        self.command_catalog_dirty = true;
        self.command_projection_dirty = true;
        self.tree_index_dirty = true;
        self.table_index_dirty = true;
    }

    fn invalidate_indexes_for_fields(&mut self, fields: &[String]) {
        for field in fields {
            match field.as_str() {
                "commands" => {
                    self.command_catalog_dirty = true;
                    self.command_projection_dirty = true;
                }
                "query" | "command_source" | "disabled_commands" => {
                    self.command_projection_dirty = true;
                }
                "nodes" | "items" | "options" => {
                    self.tree_index_dirty = true;
                }
                "columns" | "rows" => {
                    self.table_index_dirty = true;
                }
                _ => {}
            }
        }
    }

    fn retag_live_indexes(&mut self) {
        if let Some(catalog) = self.command_catalog.as_mut() {
            catalog.set_generation(self.generation);
        }
        if let Some(tree) = self.tree_index.as_mut() {
            tree.set_generation(self.generation);
        }
        if let Some(table) = self.table_index.as_mut() {
            table.set_generation(self.generation);
        }
    }
}

#[cfg(test)]
#[path = "state_model/tests/selected_output_capacity_tests.rs"]
mod selected_output_capacity_tests;

#[derive(Clone, Copy, Default)]
struct EventInvalidation {
    command_catalog: bool,
    command_projection: bool,
    tree_index: bool,
    table_index: bool,
}

impl EventInvalidation {
    fn from_event(event: &UiComponentEvent) -> Self {
        let field = match event {
            UiComponentEvent::ValueChanged { property, .. }
            | UiComponentEvent::Commit { property, .. }
            | UiComponentEvent::AddElement { property, .. }
            | UiComponentEvent::SetElement { property, .. }
            | UiComponentEvent::RemoveElement { property, .. }
            | UiComponentEvent::MoveElement { property, .. }
            | UiComponentEvent::AddMapEntry { property, .. }
            | UiComponentEvent::SetMapEntry { property, .. }
            | UiComponentEvent::RenameMapKey { property, .. }
            | UiComponentEvent::RemoveMapEntry { property, .. } => Some(property.as_str()),
            _ => None,
        };
        match field {
            Some("commands") => Self {
                command_catalog: true,
                command_projection: true,
                ..Self::default()
            },
            Some("query" | "command_source" | "disabled_commands") => Self {
                command_projection: true,
                ..Self::default()
            },
            Some("nodes" | "items" | "options") => Self {
                tree_index: true,
                ..Self::default()
            },
            Some("columns" | "rows") => Self {
                table_index: true,
                ..Self::default()
            },
            _ => Self::default(),
        }
    }
}

fn canonical_field(field: &str) -> &str {
    match field {
        "focusedIndex" => "focused_index",
        "selectedIndex" => "selected_index",
        "selectionAnchorIndex" => "selection_anchor_index",
        "sortField" | "sort_field" => "sort_column",
        "sortDirection" => "sort_direction",
        "columnWidth" => "column_width",
        _ => field,
    }
}

fn event_fields(event: &UiComponentEvent) -> Vec<String> {
    let field = match event {
        UiComponentEvent::ValueChanged { property, .. }
        | UiComponentEvent::Commit { property, .. }
        | UiComponentEvent::AddElement { property, .. }
        | UiComponentEvent::SetElement { property, .. }
        | UiComponentEvent::RemoveElement { property, .. }
        | UiComponentEvent::MoveElement { property, .. }
        | UiComponentEvent::AddMapEntry { property, .. }
        | UiComponentEvent::SetMapEntry { property, .. }
        | UiComponentEvent::RenameMapKey { property, .. }
        | UiComponentEvent::RemoveMapEntry { property, .. } => Some(property.as_str()),
        _ => None,
    };
    field
        .into_iter()
        .map(|field| canonical_field(field).to_string())
        .collect()
}

fn merge_changes(
    mut first: UiComponentStateChange,
    second: UiComponentStateChange,
) -> UiComponentStateChange {
    for field in second.changed_fields {
        if !first
            .changed_fields
            .iter()
            .any(|existing| existing == &field)
        {
            first.changed_fields.push(field);
        }
    }
    first.generation = second.generation.max(first.generation);
    first.flags_changed |= second.flags_changed;
    first.reference_sources_changed |= second.reference_sources_changed;
    first
}

fn string_value_ref(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value) | UiValue::Enum(value) => Some(value),
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

fn string_ids(value: &UiValue) -> Vec<String> {
    match value {
        UiValue::Array(values) => values.iter().flat_map(string_ids).collect(),
        UiValue::String(value) | UiValue::Enum(value) => vec![value.clone()],
        UiValue::Flags(values) => values.clone(),
        UiValue::Map(values) => ["id", "command_id", "commandId", "value", "key"]
            .into_iter()
            .find_map(|field| values.get(field).and_then(string_value_ref))
            .map(|value| vec![value.to_string()])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn is_command_palette(descriptor: &UiComponentDescriptor) -> bool {
    descriptor.role == "command-palette" || descriptor.id == "CommandPalette"
}

fn is_tree_view(descriptor: &UiComponentDescriptor) -> bool {
    matches!(
        descriptor.role.as_str(),
        "tree-view" | "folder-tree" | "mui-x-tree-view"
    ) || matches!(
        descriptor.id.as_str(),
        "TreeView" | "MaterialTreeView" | "FolderTree"
    )
}

fn is_table(descriptor: &UiComponentDescriptor) -> bool {
    matches!(
        descriptor.role.as_str(),
        "table" | "data-grid" | "mui-x-data-grid"
    ) || matches!(descriptor.id.as_str(), "Table" | "DataGrid")
}

fn noop_change(generation: u64) -> UiComponentStateChange {
    UiComponentStateChange {
        generation,
        ..UiComponentStateChange::default()
    }
}

fn int_setting_model(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    field: &str,
) -> Option<i64> {
    state
        .values
        .get(field)
        .and_then(|value| int_value(Some(value)))
        .or_else(|| {
            descriptor
                .prop(field)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(|value| int_value(Some(value)))
        })
}

fn bool_setting_model(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
    field: &str,
    default: bool,
) -> bool {
    state
        .values
        .get(field)
        .and_then(|value| match value {
            UiValue::Bool(value) => Some(*value),
            _ => None,
        })
        .or_else(|| {
            descriptor
                .prop(field)
                .and_then(|schema| schema.default_value.as_ref())
                .and_then(|value| match value {
                    UiValue::Bool(value) => Some(*value),
                    _ => None,
                })
        })
        .unwrap_or(default)
}

fn state_string_setting(state: &UiComponentState, field: &str) -> Option<String> {
    state
        .values
        .get(field)
        .and_then(string_value_ref)
        .map(str::to_owned)
}

fn tree_expanded_property(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> &'static str {
    for property in ["expanded_items", "expandedItems"] {
        if state.values.contains_key(property) || descriptor.prop(property).is_some() {
            return property;
        }
    }
    if state.values.contains_key("defaultExpandedItems")
        || descriptor.prop("defaultExpandedItems").is_some()
    {
        return "expandedItems";
    }
    "expanded_items"
}

fn tree_selected_property(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> &'static str {
    for property in ["selected_items", "selectedItems"] {
        if state.values.contains_key(property) || descriptor.prop(property).is_some() {
            return property;
        }
    }
    "selected_items"
}

fn is_selected_control_property_model(property: &str) -> bool {
    matches!(property, "selected_items" | "selectedItems")
}

fn tree_range_selecting_model(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> bool {
    [
        "range_selecting",
        "rangeSelecting",
        "shift_selecting",
        "shiftSelecting",
    ]
    .into_iter()
    .any(|field| bool_setting_model(state, descriptor, field, false))
}

fn normalize_sort_direction_model(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "asc" | "ascending" => Some("asc"),
        "desc" | "descending" => Some("desc"),
        "none" | "" => Some("none"),
        _ => None,
    }
}

fn table_uses_client_sorting_model(
    state: &UiComponentState,
    descriptor: &UiComponentDescriptor,
) -> bool {
    let mode = state
        .values
        .get("sortingMode")
        .or_else(|| state.values.get("sorting_mode"))
        .or_else(|| {
            descriptor
                .prop("sortingMode")
                .and_then(|schema| schema.default_value.as_ref())
        })
        .or_else(|| {
            descriptor
                .prop("sorting_mode")
                .and_then(|schema| schema.default_value.as_ref())
        })
        .and_then(string_value_ref);
    !matches!(mode, Some("server"))
}

fn reorder_table_rows(rows: &UiValue, permutation: &[usize]) -> Option<UiValue> {
    let UiValue::Array(rows) = rows else {
        return None;
    };
    if permutation.len() != rows.len() {
        return None;
    }
    Some(UiValue::Array(
        permutation
            .iter()
            .filter_map(|index| rows.get(*index).cloned())
            .collect(),
    ))
}

#[cfg(test)]
#[path = "tests/state_model.rs"]
mod tests;
