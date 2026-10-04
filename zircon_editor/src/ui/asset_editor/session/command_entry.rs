use std::collections::{BTreeMap, HashSet};

use crate::ui::asset_editor::UiDesignerSelectionModel;
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiComponentDefinition, UiNodeDefinition,
};

use super::{
    command::{
        UiAssetEditorCommand, UiAssetEditorDocumentReplayBundle,
        UiAssetEditorDocumentReplayCommand, UiAssetEditorTreeEdit, UiAssetEditorTreeEditKind,
    },
    ui_asset_editor_session::{
        remap_source_byte_offset, UiAssetEditorReplayResult, UiAssetEditorSession,
        UiAssetEditorSessionError, UiAssetSourceSchema,
    },
    undo_stack::UiAssetEditorUndoExternalEffects,
};

impl UiAssetEditorSession {
    pub fn apply_command(
        &mut self,
        command: UiAssetEditorCommand,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_command_with_effects(command, UiAssetEditorUndoExternalEffects::default())
    }

    pub(super) fn apply_command_with_effects(
        &mut self,
        command: UiAssetEditorCommand,
        external_effects: UiAssetEditorUndoExternalEffects,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_command_with_effects_and_theme_source(command, external_effects, None)
    }

    pub(super) fn apply_command_with_effects_and_theme_source(
        &mut self,
        command: UiAssetEditorCommand,
        external_effects: UiAssetEditorUndoExternalEffects,
        next_theme_source_key: Option<String>,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_command_with_effects_theme_and_style_rule(
            command,
            external_effects,
            next_theme_source_key,
            None,
        )
    }

    pub(super) fn apply_command_with_effects_theme_and_style_rule(
        &mut self,
        command: UiAssetEditorCommand,
        external_effects: UiAssetEditorUndoExternalEffects,
        next_theme_source_key: Option<String>,
        next_style_rule_id: Option<Option<String>>,
    ) -> Result<(), UiAssetEditorSessionError> {
        // A valid document of the wrong kind is a rejected command. Check it
        // before changing the source buffer so a failed command cannot leave
        // an unregistered dirty buffer behind.
        let mut prebuilt_v2_palette = None;
        match self.source_schema {
            UiAssetSourceSchema::LayoutDocument => {
                if let Ok(document) = super::lifecycle::parse_ui_asset_source(command.next_source())
                {
                    super::session_state::ensure_asset_kind(
                        self.route.asset_kind,
                        document.asset.kind,
                    )?;
                }
            }
            UiAssetSourceSchema::V2 => {
                if let Ok(document) = UiV2AssetLoader::load_toml_str(command.next_source()) {
                    let projection =
                        super::lifecycle::v2_projection::v2_document_to_legacy_projection_document(
                            &document,
                        )?;
                    super::session_state::ensure_asset_kind(
                        self.route.asset_kind,
                        projection.asset.kind,
                    )?;
                    prebuilt_v2_palette = super::lifecycle::preflight_v2_palette_catalog(
                        &projection,
                        &self.v2_compiler_imports.widgets,
                    )?;
                }
            }
        }
        let before_source = self.source_buffer.text().to_string();
        let before_selection = self.selection.clone();
        let before_source_cursor = self.source_cursor_snapshot();
        let before_selected_theme_source_key = self.selected_theme_source_key.clone();
        let before_selected_style_rule_id = self.selected_style_rule_id.clone();
        let tree_edit = command.structured_tree_edit().cloned();
        let document_replay = command.document_replay().cloned();
        let before_document = tree_edit.as_ref().map(|_| self.last_valid_document.clone());
        self.source_buffer
            .replace(command.next_source().to_string());
        self.source_cursor_byte_offset = remap_source_byte_offset(
            &before_source,
            self.source_buffer.text(),
            self.source_cursor_byte_offset,
        );
        if let Some(next_selection) = command.next_selection() {
            self.selection = next_selection.clone();
        }
        if let Some(next_theme_source_key) = next_theme_source_key {
            self.selected_theme_source_key = Some(next_theme_source_key);
        }
        if let Some(next_style_rule_id) = next_style_rule_id {
            self.selected_style_rule_id = next_style_rule_id;
        }
        let validation = if let Some(palette) = prebuilt_v2_palette {
            self.palette_catalog = palette;
            #[cfg(test)]
            {
                self.palette_catalog_build_count += 1;
            }
            self.revalidate_without_palette_catalog()
        } else {
            self.revalidate()
        };
        validation.map(|_| {
            if command.next_selection().is_some() {
                self.set_source_cursor_to_selected_node_start();
            } else if self.diagnostics.is_empty() {
                self.reconcile_source_cursor_state();
            }
            let after_document = tree_edit.as_ref().map(|_| self.last_valid_document.clone());
            let after_source_cursor = self.source_cursor_snapshot();
            self.undo_stack.push_edit_with_style_rule_selection(
                command.label().to_string(),
                tree_edit,
                document_replay,
                before_source,
                before_selection,
                before_source_cursor,
                before_selected_theme_source_key,
                before_selected_style_rule_id,
                before_document,
                self.source_buffer.text().to_string(),
                self.selection.clone(),
                after_source_cursor,
                self.selected_theme_source_key.clone(),
                self.selected_style_rule_id.clone(),
                after_document,
                external_effects,
            );
        })
    }

    pub fn undo_replay(&mut self) -> Result<UiAssetEditorReplayResult, UiAssetEditorSessionError> {
        let Some(record) = self.undo_stack.undo_record() else {
            return Ok(UiAssetEditorReplayResult::default());
        };
        let changed = self.apply_undo_transition(record.transition.clone())?;
        Ok(UiAssetEditorReplayResult {
            changed,
            label: record.label,
            external_effects: record.transition.external_effects,
        })
    }

    pub fn undo(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.undo_replay().map(|result| result.changed)
    }

    pub fn redo_replay(&mut self) -> Result<UiAssetEditorReplayResult, UiAssetEditorSessionError> {
        let Some(record) = self.undo_stack.redo_record() else {
            return Ok(UiAssetEditorReplayResult::default());
        };
        let changed = self.apply_undo_transition(record.transition.clone())?;
        Ok(UiAssetEditorReplayResult {
            changed,
            label: record.label,
            external_effects: record.transition.external_effects,
        })
    }

    pub fn redo(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.redo_replay().map(|result| result.changed)
    }

    pub(super) fn apply_document_edit(
        &mut self,
        document: UiAssetDocument,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_document_edit_with_kind(
            document,
            UiAssetEditorTreeEditKind::DocumentEdit,
            "Document Edit",
        )
    }

    pub(super) fn apply_document_edit_with_kind(
        &mut self,
        document: UiAssetDocument,
        kind: UiAssetEditorTreeEditKind,
        label: &str,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_document_edit_with_tree_edit(
            document,
            UiAssetEditorTreeEdit::generic(kind),
            label,
        )
    }

    pub(super) fn apply_document_edit_with_tree_edit(
        &mut self,
        document: UiAssetDocument,
        edit: UiAssetEditorTreeEdit,
        label: &str,
    ) -> Result<(), UiAssetEditorSessionError> {
        let replay = tree_document_replay_bundle(&self.last_valid_document, &document);
        self.apply_document_edit_with_tree_edit_and_replay(document, edit, label, replay)
    }

    pub(super) fn apply_document_edit_with_tree_edit_and_replay(
        &mut self,
        document: UiAssetDocument,
        edit: UiAssetEditorTreeEdit,
        label: &str,
        replay: UiAssetEditorDocumentReplayBundle,
    ) -> Result<(), UiAssetEditorSessionError> {
        let next_source = self.serialize_document_for_current_schema(&document)?;
        self.apply_command(
            UiAssetEditorCommand::tree_edit_structured(edit, label, next_source)
                .with_document_replay(replay),
        )?;
        Ok(())
    }

    pub(super) fn apply_document_edit_with_label(
        &mut self,
        document: UiAssetDocument,
        label: &str,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_document_edit_with_kind(document, UiAssetEditorTreeEditKind::DocumentEdit, label)
    }

    pub(super) fn apply_document_edit_with_label_and_replay(
        &mut self,
        document: UiAssetDocument,
        label: &str,
        replay: UiAssetEditorDocumentReplayBundle,
    ) -> Result<(), UiAssetEditorSessionError> {
        self.apply_document_edit_with_tree_edit_and_replay(
            document,
            UiAssetEditorTreeEdit::generic(UiAssetEditorTreeEditKind::DocumentEdit),
            label,
            replay,
        )
    }

    pub(super) fn apply_document_edit_with_label_replay_and_style_rule_selection(
        &mut self,
        document: UiAssetDocument,
        label: &str,
        replay: UiAssetEditorDocumentReplayBundle,
        selected_style_rule_id: Option<String>,
    ) -> Result<(), UiAssetEditorSessionError> {
        let next_source = self.serialize_document_for_current_schema(&document)?;
        self.apply_command_with_effects_theme_and_style_rule(
            UiAssetEditorCommand::tree_edit_structured(
                UiAssetEditorTreeEdit::generic(UiAssetEditorTreeEditKind::DocumentEdit),
                label,
                next_source,
            )
            .with_document_replay(replay),
            UiAssetEditorUndoExternalEffects::default(),
            None,
            Some(selected_style_rule_id),
        )?;
        Ok(())
    }

    pub(super) fn apply_binding_document_edit_with_label(
        &mut self,
        document: UiAssetDocument,
        label: &str,
    ) -> Result<(), UiAssetEditorSessionError> {
        let Some(node_id) = self.selection.primary_node_id.clone() else {
            return self.apply_document_edit_with_label(document, label);
        };
        let replay = binding_document_replay_bundle(&self.last_valid_document, &document, &node_id);
        self.apply_document_edit_with_label_and_replay(document, label, replay)
    }

    pub(super) fn apply_document_edit_with_tree_edit_and_selection(
        &mut self,
        document: UiAssetDocument,
        edit: UiAssetEditorTreeEdit,
        label: &str,
        selection: UiDesignerSelectionModel,
    ) -> Result<(), UiAssetEditorSessionError> {
        let replay = tree_document_replay_bundle(&self.last_valid_document, &document);
        let next_source = self.serialize_document_for_current_schema(&document)?;
        self.apply_command(
            UiAssetEditorCommand::tree_edit_structured_with_selection(
                edit,
                label,
                next_source,
                selection,
            )
            .with_document_replay(replay),
        )?;
        Ok(())
    }

    fn apply_undo_transition(
        &mut self,
        transition: super::undo_stack::UiAssetEditorUndoTransition,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let mut source = self.source_buffer.text().to_string();
        let source_changed = transition
            .apply_to_source(&mut source)
            .map_err(|_| UiAssetEditorSessionError::InvalidSourceBuffer)?;
        let mut replay_document = self.last_valid_document.clone();
        let document_changed = transition
            .apply_to_document(&mut replay_document)
            .map_err(|_| UiAssetEditorSessionError::InvalidSourceBuffer)?;
        let super::undo_stack::UiAssetEditorUndoTransition {
            selection,
            source_cursor,
            selected_theme_source_key,
            selected_style_rule_id,
            ..
        } = transition;
        self.selection = selection;
        self.selected_theme_source_key = selected_theme_source_key;
        self.selected_style_rule_id = selected_style_rule_id;
        self.source_buffer.replace(source);
        self.restore_source_cursor_snapshot(&source_cursor);
        if document_changed
            && self.source_schema
                == super::ui_asset_editor_session::UiAssetSourceSchema::LayoutDocument
        {
            self.apply_valid_document(replay_document)?;
            self.clear_palette_drag_state();
            return Ok(source_changed || document_changed);
        }
        self.revalidate()?;
        self.clear_palette_drag_state();
        Ok(source_changed)
    }
}

pub(super) fn tree_document_replay_bundle(
    before_document: &UiAssetDocument,
    after_document: &UiAssetDocument,
) -> UiAssetEditorDocumentReplayBundle {
    UiAssetEditorDocumentReplayBundle {
        undo: tree_document_replay_commands(after_document, before_document),
        redo: tree_document_replay_commands(before_document, after_document),
    }
}

fn tree_document_replay_commands(
    current: &UiAssetDocument,
    target: &UiAssetDocument,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    let mut commands = Vec::new();
    let current_nodes = current.node_map();
    let target_nodes = target.node_map();
    commands.extend(build_widget_import_replay_commands(
        &current.imports.widgets,
        &target.imports.widgets,
    ));
    commands.extend(upsert_component_replay_commands(
        &current.components,
        &target.components,
    ));
    commands.extend(upsert_node_replay_commands(&current_nodes, &target_nodes));
    if current.root != target.root {
        commands.push(UiAssetEditorDocumentReplayCommand::SetRoot {
            root: target.root.clone(),
        });
    }
    commands.extend(remove_component_replay_commands(
        &current.components,
        &target.components,
    ));
    commands.extend(remove_node_replay_commands(&current_nodes, &target_nodes));
    commands
}

#[cfg(test)]
#[path = "tests/command_entry_close_dirty_preflight_tests.rs"]
mod close_dirty_preflight_tests;

fn build_widget_import_replay_commands(
    current: &[String],
    target: &[String],
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    if current == target {
        return Vec::new();
    }
    if has_duplicate_string_entries(current) || has_duplicate_string_entries(target) {
        return vec![UiAssetEditorDocumentReplayCommand::SetWidgetImports {
            references: target.to_vec(),
        }];
    }

    let target_entries = widget_import_target_entries(target);
    let mut working = current.to_vec();
    let mut commands = Vec::with_capacity(current.len().saturating_add(target.len()));

    for index in (0..working.len()).rev() {
        if target_entries.contains(working[index].as_str()) {
            continue;
        }
        let reference = working.remove(index);
        commands.push(UiAssetEditorDocumentReplayCommand::RemoveWidgetImport { index, reference });
    }

    for (target_index, target_reference) in target.iter().enumerate() {
        match working
            .iter()
            .position(|reference| reference == target_reference)
        {
            Some(current_index) => {
                if current_index != target_index {
                    let moved = working.remove(current_index);
                    working.insert(target_index, moved);
                    commands.push(UiAssetEditorDocumentReplayCommand::MoveWidgetImport {
                        from_index: current_index,
                        to_index: target_index,
                        reference: target_reference.clone(),
                    });
                }
            }
            None => {
                working.insert(target_index, target_reference.clone());
                commands.push(UiAssetEditorDocumentReplayCommand::InsertWidgetImport {
                    index: target_index,
                    reference: target_reference.clone(),
                });
            }
        }
    }

    if working != target {
        return vec![UiAssetEditorDocumentReplayCommand::SetWidgetImports {
            references: target.to_vec(),
        }];
    }

    commands
}

fn widget_import_target_entries(target: &[String]) -> HashSet<&str> {
    let mut target_entries = HashSet::with_capacity(target.len());
    target_entries.extend(target.iter().map(String::as_str));
    target_entries
}

fn upsert_node_replay_commands(
    current: &BTreeMap<String, UiNodeDefinition>,
    target: &BTreeMap<String, UiNodeDefinition>,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    target
        .iter()
        .filter_map(|(node_id, node)| {
            (current.get(node_id) != Some(node)).then(|| {
                UiAssetEditorDocumentReplayCommand::UpsertNode {
                    node_id: node_id.clone(),
                    node: node.clone(),
                }
            })
        })
        .collect()
}

fn remove_node_replay_commands(
    current: &BTreeMap<String, UiNodeDefinition>,
    target: &BTreeMap<String, UiNodeDefinition>,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    current
        .keys()
        .filter(|node_id| !target.contains_key(*node_id))
        .map(|node_id| UiAssetEditorDocumentReplayCommand::RemoveNode {
            node_id: node_id.clone(),
        })
        .collect()
}

fn upsert_component_replay_commands(
    current: &BTreeMap<String, UiComponentDefinition>,
    target: &BTreeMap<String, UiComponentDefinition>,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    target
        .iter()
        .filter_map(|(component_name, component)| {
            (current.get(component_name) != Some(component)).then(|| {
                UiAssetEditorDocumentReplayCommand::UpsertComponent {
                    component_name: component_name.clone(),
                    component: component.clone(),
                }
            })
        })
        .collect()
}

fn remove_component_replay_commands(
    current: &BTreeMap<String, UiComponentDefinition>,
    target: &BTreeMap<String, UiComponentDefinition>,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    current
        .keys()
        .filter(|component_name| !target.contains_key(*component_name))
        .map(
            |component_name| UiAssetEditorDocumentReplayCommand::RemoveComponent {
                component_name: component_name.clone(),
            },
        )
        .collect()
}

pub(super) fn binding_document_replay_bundle(
    before_document: &UiAssetDocument,
    after_document: &UiAssetDocument,
    node_id: &str,
) -> UiAssetEditorDocumentReplayBundle {
    UiAssetEditorDocumentReplayBundle {
        undo: binding_document_replay_commands(after_document, before_document, node_id),
        redo: binding_document_replay_commands(before_document, after_document, node_id),
    }
}

fn binding_document_replay_commands(
    current: &UiAssetDocument,
    target: &UiAssetDocument,
    node_id: &str,
) -> Vec<UiAssetEditorDocumentReplayCommand> {
    let current_bindings = current
        .node(node_id)
        .map(|node| node.bindings.clone())
        .unwrap_or_default();
    let target_bindings = target
        .node(node_id)
        .map(|node| node.bindings.clone())
        .unwrap_or_default();
    if current_bindings == target_bindings {
        return Vec::new();
    }
    vec![UiAssetEditorDocumentReplayCommand::SetNodeBindings {
        node_id: node_id.to_string(),
        bindings: target_bindings,
    }]
}

fn has_duplicate_string_entries(entries: &[String]) -> bool {
    let mut seen = HashSet::with_capacity(entries.len());
    entries
        .iter()
        .map(String::as_str)
        .any(|entry| !seen.insert(entry))
}

#[cfg(test)]
#[path = "command_entry/tests/optimization_batch_jn_editor653_tests.rs"]
mod optimization_batch_jn_editor653_tests;

#[cfg(test)]
#[path = "tests/command_entry_optimization_tests.rs"]
mod optimization_tests;
