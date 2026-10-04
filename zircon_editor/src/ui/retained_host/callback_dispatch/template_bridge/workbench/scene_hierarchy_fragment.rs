use std::collections::BTreeSet;

use zircon_runtime::scene::{EntityId, WorldInspectionHierarchyRow};
use zircon_runtime_interface::ui::component::UiValue;

use crate::ui::workbench::snapshot::{SceneEntries, SceneInspectionHierarchyFragment};

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
    scene_hierarchy_projection::SceneHierarchyLogicalRowPatch,
};

const TREE_ROW_INDENT_STEP: f64 = 20.0;
const SCENE_NODE_ID: &str = "scene_node_id";
const SCENE_PARENT_ID: &str = "scene_parent_id";
const SCENE_SUBTREE_HASH: &str = "scene_subtree_hash";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SceneHierarchyFragmentApply {
    Applied {
        changed_control_ids: Vec<String>,
        logical_row_patches: Vec<SceneHierarchyLogicalRowPatch>,
        reflowed: bool,
    },
    SelectionResyncRequired,
    ResyncRequired,
}

impl SceneHierarchyFragmentApply {
    pub(crate) const fn applied(&self) -> bool {
        matches!(self, Self::Applied { .. })
    }

    pub(crate) fn updated_rows(&self) -> usize {
        match self {
            Self::Applied {
                logical_row_patches,
                ..
            } => logical_row_patches.len(),
            Self::SelectionResyncRequired | Self::ResyncRequired => 0,
        }
    }

    pub(crate) const fn reflowed(&self) -> bool {
        matches!(self, Self::Applied { reflowed: true, .. })
    }

    pub(crate) const fn selection_resync_required(&self) -> bool {
        matches!(self, Self::SelectionResyncRequired)
    }

    pub(crate) fn changed_control_ids(&self) -> &[String] {
        match self {
            Self::Applied {
                changed_control_ids,
                ..
            } => changed_control_ids,
            Self::SelectionResyncRequired | Self::ResyncRequired => &[],
        }
    }

    pub(crate) fn logical_row_patches(&self) -> &[SceneHierarchyLogicalRowPatch] {
        match self {
            Self::Applied {
                logical_row_patches,
                ..
            } => logical_row_patches,
            Self::SelectionResyncRequired | Self::ResyncRequired => &[],
        }
    }
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn set_scene_filter_query(&mut self, query: &str) {
        self.scene_filter_query = query.to_string();
    }

    pub(crate) fn invalidate_scene_hierarchy_projection(&mut self) {
        self.scene_hierarchy_projection = Default::default();
    }

    /// Applies an O(Delta) hierarchy patch. Complete rows are accepted only by
    /// `resync_scene_hierarchy`, which is invoked explicitly for a reflow.
    pub(crate) fn apply_scene_hierarchy_fragment(
        &mut self,
        fragment: &SceneInspectionHierarchyFragment,
    ) -> Result<SceneHierarchyFragmentApply, BuiltinHostWindowTemplateBridgeError> {
        zircon_runtime::profile_scope!("editor", "scene_inspection", "hierarchy_fragment_apply");
        let Some(changed_rows) = fragment.changed_rows() else {
            return Ok(SceneHierarchyFragmentApply::ResyncRequired);
        };
        let message = fragment.message();
        zircon_runtime::profile_counter!(
            "editor",
            "scene_inspection_hierarchy_fragment_patch_rows",
            changed_rows.len()
        );
        zircon_runtime::profile_counter!(
            "editor",
            "scene_inspection_hierarchy_fragment_selection_added",
            message.selection().added_entities().len()
        );
        zircon_runtime::profile_counter!(
            "editor",
            "scene_inspection_hierarchy_fragment_selection_removed",
            message.selection().removed_entities().len()
        );
        if message.requires_resync()
            || self.scene_hierarchy_projection.generation() != message.previous_generation()
            || !self.patch_rows_match_current_projection(changed_rows)
            || !self.selection_entities_exist(message)
        {
            zircon_runtime::profile_counter!(
                "editor",
                "scene_inspection_hierarchy_fragment_resync_required",
                1
            );
            return Ok(SceneHierarchyFragmentApply::ResyncRequired);
        }
        let selection_already_current = self.scene_hierarchy_projection.selection_revision()
            == Some(message.selection().revision());
        let expected_selection_revision = if message.selection().requires_resync() {
            Some(message.selection().revision())
        } else {
            message.selection().previous_revision()
        };
        if !selection_already_current
            && self.scene_hierarchy_projection.selection_revision() != expected_selection_revision
        {
            zircon_runtime::profile_counter!(
                "editor",
                "scene_inspection_hierarchy_fragment_selection_resync_required",
                1
            );
            return Ok(SceneHierarchyFragmentApply::SelectionResyncRequired);
        }

        let mut changed_controls = BTreeSet::new();
        let selection_changed_entities = message
            .selection()
            .added_entities()
            .iter()
            .copied()
            .chain(message.selection().removed_entities().iter().copied())
            .collect::<BTreeSet<_>>();
        for row in changed_rows {
            let control_id = self
                .scene_hierarchy_projection
                .control_for(row.entity)
                .map(str::to_string);
            if let Some(control_id) = control_id {
                self.sync_scene_row(
                    &control_id,
                    row,
                    self.scene_hierarchy_projection.is_selected(row.entity),
                    self.scene_expanded_by_entity
                        .get(&row.entity)
                        .copied()
                        .unwrap_or(false)
                        || self
                            .scene_filter_forced_expanded_entities
                            .contains(&row.entity),
                )?;
                changed_controls.insert(control_id);
            }
        }
        let selection_applied = selection_already_current
            || self.apply_selection_delta(message, &mut changed_controls)?;
        if !selection_applied {
            return Ok(SceneHierarchyFragmentApply::ResyncRequired);
        }
        if !changed_controls.is_empty() {
            self.template_surface
                .refresh_after_state_change(self.runtime.as_ref())?;
        }
        if !selection_already_current {
            self.commit_selection_delta(message);
        }
        for row in changed_rows {
            self.scene_hierarchy_projection.patch_row(row);
        }
        let changed_row_entities = changed_rows
            .iter()
            .map(|row| row.entity)
            .collect::<BTreeSet<_>>();
        let mut logical_row_patches = changed_rows
            .iter()
            .filter_map(|row| self.scene_hierarchy_projection.logical_content_patch(row))
            .collect::<Vec<_>>();
        logical_row_patches.extend(
            selection_changed_entities
                .difference(&changed_row_entities)
                .filter_map(|entity| {
                    self.scene_hierarchy_projection
                        .logical_selection_patch(*entity)
                }),
        );
        logical_row_patches.sort_unstable_by_key(SceneHierarchyLogicalRowPatch::row_index);
        self.scene_hierarchy_projection
            .replace_generation(Some(message.generation()));
        self.scene_hierarchy_projection
            .replace_selection_revision(Some(message.selection().revision()));
        zircon_runtime::profile_counter!(
            "editor",
            "scene_inspection_hierarchy_fragment_updated_rows",
            logical_row_patches.len()
        );
        zircon_runtime::profile_counter!(
            "editor",
            "scene_inspection_hierarchy_fragment_reflowed",
            0
        );
        Ok(SceneHierarchyFragmentApply::Applied {
            changed_control_ids: changed_controls.into_iter().collect(),
            logical_row_patches,
            reflowed: false,
        })
    }

    fn apply_selection_delta(
        &mut self,
        message: &crate::core::editor_message::SceneInspectionMessage,
        changed_controls: &mut BTreeSet<String>,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        for entity in message.selection().removed_entities() {
            if !self.scene_hierarchy_projection.contains_entity(*entity) {
                return Ok(false);
            }
            if let Some(control_id) = self
                .scene_hierarchy_projection
                .control_for(*entity)
                .map(str::to_string)
            {
                self.set_selected(&control_id, false)?;
                changed_controls.insert(control_id);
            }
        }
        for entity in message.selection().added_entities() {
            if !self.scene_hierarchy_projection.contains_entity(*entity) {
                return Ok(false);
            }
            if let Some(control_id) = self
                .scene_hierarchy_projection
                .control_for(*entity)
                .map(str::to_string)
            {
                self.set_selected(&control_id, true)?;
                changed_controls.insert(control_id);
            }
        }
        Ok(true)
    }

    fn commit_selection_delta(
        &mut self,
        message: &crate::core::editor_message::SceneInspectionMessage,
    ) {
        for entity in message.selection().removed_entities() {
            self.scene_hierarchy_projection.deselect(*entity);
        }
        for entity in message.selection().added_entities() {
            self.scene_hierarchy_projection.select(*entity);
        }
    }

    fn apply_selection_snapshot(
        &mut self,
        selected_entities: &[EntityId],
        changed_controls: &mut BTreeSet<String>,
    ) -> Result<Option<BTreeSet<EntityId>>, BuiltinHostWindowTemplateBridgeError> {
        let selected_entities = selected_entities.iter().copied().collect::<BTreeSet<_>>();
        let removed_entities = self
            .scene_hierarchy_projection
            .selected_entities()
            .difference(&selected_entities)
            .copied()
            .collect::<Vec<_>>();
        let added_entities = selected_entities
            .difference(self.scene_hierarchy_projection.selected_entities())
            .copied()
            .collect::<Vec<_>>();
        if removed_entities
            .iter()
            .chain(&added_entities)
            .any(|entity| !self.scene_hierarchy_projection.contains_entity(*entity))
        {
            return Ok(None);
        }
        for entity in removed_entities {
            if let Some(control_id) = self
                .scene_hierarchy_projection
                .control_for(entity)
                .map(str::to_string)
            {
                self.set_selected(&control_id, false)?;
                changed_controls.insert(control_id);
            }
        }
        for entity in added_entities {
            if let Some(control_id) = self
                .scene_hierarchy_projection
                .control_for(entity)
                .map(str::to_string)
            {
                self.set_selected(&control_id, true)?;
                changed_controls.insert(control_id);
            }
        }
        Ok(Some(selected_entities))
    }

    /// Explicitly rebuilds only the hierarchy rows after a generation gap, topology change, or
    /// filtered view. This is the only retained bridge path that accepts a complete hierarchy.
    pub(crate) fn resync_scene_hierarchy(
        &mut self,
        scene_entries: &SceneEntries,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.resync_scene_hierarchy_at_selection(scene_entries, 0)
    }

    pub(crate) fn resync_scene_hierarchy_at_selection(
        &mut self,
        scene_entries: &SceneEntries,
        selection_revision: u64,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        zircon_runtime::profile_scope!("editor", "scene_inspection", "hierarchy_full_resync");
        self.sync_scene_entries(scene_entries, Some(selection_revision))?;
        if let Err(error) = self
            .template_surface
            .refresh_after_state_change(self.runtime.as_ref())
        {
            self.scene_hierarchy_projection.replace_generation(None);
            self.scene_hierarchy_projection
                .replace_selection_revision(None);
            return Err(error.into());
        }
        zircon_runtime::profile_counter!(
            "editor",
            "scene_inspection_hierarchy_full_resync_rows",
            scene_entries.len()
        );
        Ok(())
    }

    /// Synchronize a full product hierarchy snapshot and its entity-owned tree view state.
    /// Search results reveal their matching ancestor path without changing saved disclosure
    /// state; clearing the query restores the user's per-entity expansion choices.
    pub(crate) fn sync_scene_view_state(
        &mut self,
        scene_entries: &SceneEntries,
        filter_query: &str,
        expanded_ids: &[EntityId],
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let expanded_ids = expanded_ids.iter().copied().collect::<BTreeSet<_>>();
        self.scene_expanded_by_entity = scene_entries
            .iter()
            .map(|row| {
                (
                    row.entity,
                    row.has_children && expanded_ids.contains(&row.entity),
                )
            })
            .collect();
        self.sync_scene_query_view(scene_entries, filter_query, None, false)
    }

    /// Apply the current query to authoritative rows while preserving the surface's
    /// entity-keyed disclosure state.
    pub(crate) fn sync_scene_query_view(
        &mut self,
        scene_entries: &SceneEntries,
        filter_query: &str,
        selection_revision: Option<u64>,
        remember_surface_state: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        if remember_surface_state {
            self.remember_scene_expansion_by_entity();
        }
        let matching_entries = scene_entries.filtered_by_hierarchy_query(filter_query);
        let forced_expanded = if filter_query.trim().is_empty() {
            BTreeSet::new()
        } else {
            hierarchy_ancestor_entities(&matching_entries)
        };
        let rows = if filter_query.trim().is_empty() {
            rows_visible_with_expansion(&matching_entries, &self.scene_expanded_by_entity)
        } else {
            matching_entries.hierarchy_rows_arc().to_vec()
        };
        let projected_entries = matching_entries.with_hierarchy_rows(rows);
        self.sync_scene_entries_internal(
            &projected_entries,
            selection_revision,
            filter_query,
            forced_expanded,
        )
    }

    pub(crate) fn effective_expanded_ids(&self, scene_entries: &SceneEntries) -> Vec<EntityId> {
        scene_entries
            .iter()
            .filter(|row| {
                row.has_children && self.scene_expanded_by_entity.get(&row.entity) == Some(&true)
            })
            .map(|row| row.entity)
            .collect()
    }

    /// Repairs only the editor-owned selection overlay after a Latest delivery gap.
    pub(crate) fn resync_scene_hierarchy_selection(
        &mut self,
        selection_revision: u64,
        selected_entities: &[EntityId],
    ) -> Result<SceneHierarchyFragmentApply, BuiltinHostWindowTemplateBridgeError> {
        zircon_runtime::profile_scope!("editor", "scene_inspection", "selection_overlay_resync");
        if !selected_entities
            .iter()
            .all(|entity| self.scene_hierarchy_projection.contains_entity(*entity))
        {
            return Ok(SceneHierarchyFragmentApply::ResyncRequired);
        }
        let mut changed_controls = BTreeSet::new();
        let next_selected_entities = selected_entities.iter().copied().collect::<BTreeSet<_>>();
        let changed_entities = self
            .scene_hierarchy_projection
            .selected_entities()
            .symmetric_difference(&next_selected_entities)
            .copied()
            .collect::<Vec<_>>();
        let Some(selected_entities) =
            self.apply_selection_snapshot(selected_entities, &mut changed_controls)?
        else {
            return Ok(SceneHierarchyFragmentApply::ResyncRequired);
        };
        let selected_entity_count = selected_entities.len();
        if !changed_controls.is_empty() {
            self.template_surface
                .refresh_after_state_change(self.runtime.as_ref())?;
        }
        self.scene_hierarchy_projection
            .replace_selected_entities(selected_entities);
        self.scene_hierarchy_projection
            .replace_selection_revision(Some(selection_revision));
        let logical_row_patches = changed_entities
            .into_iter()
            .filter_map(|entity| {
                self.scene_hierarchy_projection
                    .logical_selection_patch(entity)
            })
            .collect();
        zircon_runtime::profile_counter!(
            "editor",
            "scene_inspection_hierarchy_selection_resync_entities",
            selected_entity_count
        );
        Ok(SceneHierarchyFragmentApply::Applied {
            changed_control_ids: changed_controls.into_iter().collect(),
            logical_row_patches,
            reflowed: false,
        })
    }

    pub(super) fn sync_scene_entries(
        &mut self,
        scene_entries: &SceneEntries,
        selection_revision: Option<u64>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let filter_query = self.scene_filter_query.clone();
        self.sync_scene_query_view(scene_entries, &filter_query, selection_revision, true)
    }

    fn sync_scene_entries_internal(
        &mut self,
        scene_entries: &SceneEntries,
        selection_revision: Option<u64>,
        filter_query: &str,
        forced_expanded: BTreeSet<EntityId>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.scene_filter_query = filter_query.to_string();
        self.scene_filter_forced_expanded_entities = forced_expanded;
        self.reconcile_scene_tree_row_capacity(scene_entries.len())?;
        let controls = self.scene_tree_control_ids()?;
        for (index, control_id) in controls.iter().enumerate() {
            let Some(row) = scene_entries.get(index) else {
                self.set_selected(control_id, false)?;
                self.set_visible(control_id, false)?;
                continue;
            };
            let expanded = self
                .scene_expanded_by_entity
                .get(&row.entity)
                .copied()
                .unwrap_or(false)
                || self
                    .scene_filter_forced_expanded_entities
                    .contains(&row.entity);
            self.scene_expanded_by_entity
                .entry(row.entity)
                .or_insert(false);
            self.sync_scene_row(
                control_id,
                row,
                scene_entries.is_selected(row.entity),
                expanded,
            )?;
        }
        self.mutate_control_property(
            "WorkbenchSceneSearchField",
            "query",
            UiValue::String(filter_query.to_string()),
        )?;
        self.scene_hierarchy_projection.replace(
            scene_entries.inspection_generation(),
            selection_revision,
            scene_entries,
            &controls,
            scene_entries.selected_entities(),
        );
        Ok(())
    }

    fn remember_scene_expansion_by_entity(&mut self) {
        let Ok(controls) = self.scene_tree_control_ids() else {
            return;
        };
        for control_id in controls {
            let Some(entity) = self.scene_node_id_for_control(&control_id) else {
                continue;
            };
            let expanded = self.scene_row_expanded(&control_id);
            if self.scene_filter_forced_expanded_entities.contains(&entity) && expanded {
                continue;
            }
            self.scene_expanded_by_entity.insert(entity, expanded);
        }
    }

    fn scene_row_expanded(&self, control_id: &str) -> bool {
        let Some(node_id) = self.control_node_id(control_id) else {
            return false;
        };
        let node = self.template_surface.surface.tree.nodes.get(&node_id);
        if let Some(value) = self
            .template_surface
            .surface
            .component_states
            .get(node_id)
            .and_then(|state| state.value("expanded"))
        {
            if let UiValue::Bool(expanded) = value {
                return *expanded;
            }
        }
        if self
            .template_surface
            .surface
            .component_states
            .get(node_id)
            .is_some_and(|state| state.flags.expanded)
        {
            return true;
        }
        node.and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.attributes.get("expanded"))
            .and_then(toml::Value::as_bool)
            .unwrap_or(false)
    }

    fn patch_rows_match_current_projection(&self, rows: &[WorldInspectionHierarchyRow]) -> bool {
        rows.iter().all(|row| {
            self.scene_hierarchy_projection.row_identity_matches(row)
                && self
                    .scene_hierarchy_projection
                    .control_for(row.entity)
                    .is_none_or(|control_id| {
                        self.scene_node_id_for_control(control_id) == Some(row.entity)
                            && self.control_integer(control_id, "tree_depth")
                                == Some(row.depth as i64)
                            && self.control_string(control_id, SCENE_PARENT_ID)
                                == Some(scene_parent_id(row.parent))
                    })
        })
    }

    fn scene_node_id_for_control(&self, control_id: &str) -> Option<EntityId> {
        let node_id = self.control_node_id(control_id)?;
        self.template_surface
            .surface
            .tree
            .nodes
            .get(&node_id)
            .and_then(|node| node.template_metadata.as_ref())
            .and_then(|metadata| metadata.attributes.get(SCENE_NODE_ID))
            .and_then(toml::Value::as_str)
            .and_then(|id| id.parse().ok())
    }

    fn selection_entities_exist(
        &self,
        message: &crate::core::editor_message::SceneInspectionMessage,
    ) -> bool {
        message
            .selection()
            .added_entities()
            .iter()
            .chain(message.selection().removed_entities())
            .all(|entity| self.scene_hierarchy_projection.contains_entity(*entity))
    }

    fn sync_scene_row(
        &mut self,
        control_id: &str,
        row: &WorldInspectionHierarchyRow,
        selected: bool,
        expanded: bool,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.set_visible(control_id, true)?;
        self.mutate_control_property(
            control_id,
            "text",
            UiValue::String(non_empty_label(&row.display_name, "Entity")),
        )?;
        self.mutate_control_property(control_id, "tree_depth", UiValue::Int(row.depth as i64))?;
        self.mutate_control_property(
            control_id,
            "tree_indent_px",
            UiValue::Float(row.depth as f64 * TREE_ROW_INDENT_STEP),
        )?;
        self.mutate_control_property(
            control_id,
            SCENE_NODE_ID,
            UiValue::String(row.entity.to_string()),
        )?;
        self.mutate_control_property(
            control_id,
            SCENE_PARENT_ID,
            UiValue::String(scene_parent_id(row.parent)),
        )?;
        self.mutate_control_property(
            control_id,
            SCENE_SUBTREE_HASH,
            UiValue::String(row.subtree_hash.to_string()),
        )?;
        self.mutate_control_property(control_id, "expanded", UiValue::Bool(expanded))?;
        self.set_selected(control_id, selected)?;
        Ok(())
    }
}

fn scene_parent_id(parent: Option<u64>) -> String {
    parent.map_or_else(String::new, |entity| entity.to_string())
}

fn rows_visible_with_expansion(
    rows: &[WorldInspectionHierarchyRow],
    expanded_by_entity: &std::collections::BTreeMap<EntityId, bool>,
) -> Vec<WorldInspectionHierarchyRow> {
    let mut visible_entities = BTreeSet::new();
    let mut visible_rows = Vec::with_capacity(rows.len());
    for row in rows {
        let visible = row.parent.is_none_or(|parent| {
            visible_entities.contains(&parent)
                && expanded_by_entity.get(&parent).copied().unwrap_or(false)
        });
        if visible {
            visible_entities.insert(row.entity);
            visible_rows.push(row.clone());
        }
    }
    visible_rows
}

fn hierarchy_ancestor_entities(rows: &[WorldInspectionHierarchyRow]) -> BTreeSet<EntityId> {
    rows.iter().filter_map(|row| row.parent).collect()
}

fn non_empty_label(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
#[path = "tests/scene_hierarchy_fragment.rs"]
mod tests;
