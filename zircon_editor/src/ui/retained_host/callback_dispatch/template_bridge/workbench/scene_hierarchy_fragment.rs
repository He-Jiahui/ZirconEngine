use std::collections::BTreeSet;

use zircon_runtime::scene::{EntityId, WorldInspectionHierarchyRow};
use zircon_runtime_interface::ui::component::UiValue;

use crate::ui::workbench::snapshot::{SceneEntries, SceneInspectionHierarchyFragment};

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const TREE_ROW_INDENT_STEP: f64 = 20.0;
const SCENE_PARENT_ID: &str = "scene_parent_id";
const SCENE_SUBTREE_HASH: &str = "scene_subtree_hash";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SceneHierarchyFragmentApply {
    Applied {
        changed_control_ids: Vec<String>,
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
                changed_control_ids,
                ..
            } => changed_control_ids.len(),
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
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
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
        for row in changed_rows {
            let Some(control_id) = self
                .scene_hierarchy_projection
                .control_for(row.entity)
                .map(str::to_string)
            else {
                return Ok(SceneHierarchyFragmentApply::ResyncRequired);
            };
            self.sync_scene_row(
                &control_id,
                row,
                self.scene_hierarchy_projection.is_selected(row.entity),
            )?;
            changed_controls.insert(control_id);
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
        self.scene_hierarchy_projection
            .replace_generation(Some(message.generation()));
        self.scene_hierarchy_projection
            .replace_selection_revision(Some(message.selection().revision()));
        zircon_runtime::profile_counter!(
            "editor",
            "scene_inspection_hierarchy_fragment_updated_rows",
            changed_controls.len()
        );
        zircon_runtime::profile_counter!(
            "editor",
            "scene_inspection_hierarchy_fragment_reflowed",
            0
        );
        Ok(SceneHierarchyFragmentApply::Applied {
            changed_control_ids: changed_controls.into_iter().collect(),
            reflowed: false,
        })
    }

    fn apply_selection_delta(
        &mut self,
        message: &crate::core::editor_message::SceneInspectionMessage,
        changed_controls: &mut BTreeSet<String>,
    ) -> Result<bool, BuiltinHostWindowTemplateBridgeError> {
        for entity in message.selection().removed_entities() {
            let Some(control_id) = self
                .scene_hierarchy_projection
                .control_for(*entity)
                .map(str::to_string)
            else {
                return Ok(false);
            };
            self.set_selected(&control_id, false)?;
            changed_controls.insert(control_id);
        }
        for entity in message.selection().added_entities() {
            let Some(control_id) = self
                .scene_hierarchy_projection
                .control_for(*entity)
                .map(str::to_string)
            else {
                return Ok(false);
            };
            self.set_selected(&control_id, true)?;
            changed_controls.insert(control_id);
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
        let removed_controls = removed_entities
            .iter()
            .map(|entity| {
                self.scene_hierarchy_projection
                    .control_for(*entity)
                    .map(str::to_string)
            })
            .collect::<Option<Vec<_>>>();
        let added_controls = added_entities
            .iter()
            .map(|entity| {
                self.scene_hierarchy_projection
                    .control_for(*entity)
                    .map(str::to_string)
            })
            .collect::<Option<Vec<_>>>();
        let (Some(removed_controls), Some(added_controls)) = (removed_controls, added_controls)
        else {
            return Ok(None);
        };
        for control_id in removed_controls {
            self.set_selected(&control_id, false)?;
            changed_controls.insert(control_id);
        }
        for control_id in added_controls {
            self.set_selected(&control_id, true)?;
            changed_controls.insert(control_id);
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

    /// Repairs only the editor-owned selection overlay after a Latest delivery gap.
    pub(crate) fn resync_scene_hierarchy_selection(
        &mut self,
        selection_revision: u64,
        selected_entities: &[EntityId],
    ) -> Result<SceneHierarchyFragmentApply, BuiltinHostWindowTemplateBridgeError> {
        zircon_runtime::profile_scope!("editor", "scene_inspection", "selection_overlay_resync");
        if !selected_entities.iter().all(|entity| {
            self.scene_hierarchy_projection
                .control_for(*entity)
                .is_some()
        }) {
            return Ok(SceneHierarchyFragmentApply::ResyncRequired);
        }
        let mut changed_controls = BTreeSet::new();
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
        zircon_runtime::profile_counter!(
            "editor",
            "scene_inspection_hierarchy_selection_resync_entities",
            selected_entity_count
        );
        Ok(SceneHierarchyFragmentApply::Applied {
            changed_control_ids: changed_controls.into_iter().collect(),
            reflowed: false,
        })
    }

    pub(super) fn sync_scene_entries(
        &mut self,
        scene_entries: &SceneEntries,
        selection_revision: Option<u64>,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.reconcile_scene_tree_row_capacity(scene_entries.len())?;
        let controls = self.scene_tree_control_ids()?;
        for (index, control_id) in controls.iter().enumerate() {
            let Some(row) = scene_entries.get(index) else {
                self.set_selected(control_id, false)?;
                self.set_visible(control_id, false)?;
                continue;
            };
            self.sync_scene_row(control_id, row, scene_entries.is_selected(row.entity))?;
        }
        self.scene_hierarchy_projection.replace(
            scene_entries.inspection_generation(),
            selection_revision,
            scene_entries,
            &controls,
            scene_entries.selected_entities(),
        );
        Ok(())
    }

    fn patch_rows_match_current_projection(&self, rows: &[WorldInspectionHierarchyRow]) -> bool {
        rows.iter().all(|row| {
            let Some(control_id) = self.scene_hierarchy_projection.control_for(row.entity) else {
                return false;
            };
            self.control_integer(control_id, "scene_node_id") == Some(scene_node_id(row.entity))
                && self.control_integer(control_id, "tree_depth") == Some(row.depth as i64)
                && self.control_string(control_id, SCENE_PARENT_ID)
                    == Some(scene_parent_id(row.parent))
        })
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
            .all(|entity| {
                self.scene_hierarchy_projection
                    .control_for(*entity)
                    .is_some()
            })
    }

    fn sync_scene_row(
        &mut self,
        control_id: &str,
        row: &WorldInspectionHierarchyRow,
        selected: bool,
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
            "scene_node_id",
            UiValue::Int(scene_node_id(row.entity)),
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
        self.mutate_control_property(control_id, "expanded", UiValue::Bool(row.has_children))?;
        self.set_selected(control_id, selected)?;
        Ok(())
    }
}

fn scene_node_id(entity: u64) -> i64 {
    entity.min(i64::MAX as u64) as i64
}

fn scene_parent_id(parent: Option<u64>) -> String {
    parent.map_or_else(String::new, |entity| entity.to_string())
}

fn non_empty_label(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}
