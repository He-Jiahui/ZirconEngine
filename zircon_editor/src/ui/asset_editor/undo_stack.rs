use std::collections::BTreeMap;

use crate::ui::asset_editor::UiDesignerSelectionModel;
use zircon_runtime::ui::template::UiAssetDocument;

use super::command::{
    UiAssetEditorDocumentReplayBundle, UiAssetEditorDocumentReplayCommand,
    UiAssetEditorInverseTreeEdit, UiAssetEditorTreeEdit, UiAssetEditorTreeEditKind,
};
use super::document_diff::UiAssetDocumentDiff;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiAssetEditorExternalEffect {
    UpsertAssetSource { asset_id: String, source: String },
    RestoreAssetSource { asset_id: String, source: String },
    RemoveAssetSource { asset_id: String },
}

impl UiAssetEditorExternalEffect {
    pub fn apply_to_asset_sources(&self, asset_sources: &mut BTreeMap<String, String>) -> bool {
        match self {
            Self::UpsertAssetSource { asset_id, source }
            | Self::RestoreAssetSource { asset_id, source } => {
                apply_asset_source(asset_sources, asset_id, source)
            }
            Self::RemoveAssetSource { asset_id } => asset_sources.remove(asset_id).is_some(),
        }
    }
}

fn apply_asset_source(
    asset_sources: &mut BTreeMap<String, String>,
    asset_id: &str,
    source: &str,
) -> bool {
    if asset_sources
        .get(asset_id)
        .is_some_and(|existing| existing == source)
    {
        return false;
    }
    asset_sources.insert(asset_id.to_string(), source.to_string());
    true
}

pub fn apply_external_effects_to_asset_sources(
    asset_sources: &mut BTreeMap<String, String>,
    effects: &[UiAssetEditorExternalEffect],
) -> bool {
    let mut changed = false;
    for effect in effects {
        changed |= effect.apply_to_asset_sources(asset_sources);
    }
    changed
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetEditorUndoExternalEffects {
    pub undo: Vec<UiAssetEditorExternalEffect>,
    pub redo: Vec<UiAssetEditorExternalEffect>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetEditorSourceCursorSnapshot {
    pub byte_offset: usize,
    pub anchor_node_id: Option<String>,
    pub line_offset: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiAssetEditorUndoTransition {
    pub selection: UiDesignerSelectionModel,
    pub source_cursor: UiAssetEditorSourceCursorSnapshot,
    pub selected_theme_source_key: Option<String>,
    pub document: Option<UiAssetEditorUndoDocumentReplay>,
    pub document_commands: Vec<UiAssetEditorDocumentReplayCommand>,
    pub external_effects: Vec<UiAssetEditorExternalEffect>,
    source: UiAssetEditorSourceReplay,
}

impl UiAssetEditorUndoTransition {
    pub fn apply_to_source(&self, source: &mut String) -> Result<bool, &'static str> {
        self.source.apply_to(source)
    }

    pub fn apply_to_document(&self, document: &mut UiAssetDocument) -> Result<bool, &'static str> {
        let original_document = document.clone();
        let mut changed = false;
        if !self.document_commands.is_empty() {
            for command in &self.document_commands {
                changed |= apply_document_replay_command(document, command)?;
            }
        }
        if let Some(replay) = self.document.as_ref() {
            let mut target_document = original_document;
            let _ = replay.apply_to_document(&mut target_document)?;
            if *document != target_document {
                *document = target_document;
                changed = true;
            }
        }
        Ok(changed)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UiAssetEditorSourceReplay {
    replace_start: usize,
    replace_len: usize,
    insert: String,
}

impl UiAssetEditorSourceReplay {
    fn between(current: &str, target: &str) -> Self {
        let prefix_len = common_prefix_len(current, target);
        let current_suffix = &current[prefix_len..];
        let target_suffix = &target[prefix_len..];
        let suffix_len = common_suffix_len(current_suffix, target_suffix);
        let current_end = current.len().saturating_sub(suffix_len);
        let target_end = target.len().saturating_sub(suffix_len);
        Self {
            replace_start: prefix_len,
            replace_len: current_end.saturating_sub(prefix_len),
            insert: target[prefix_len..target_end].to_string(),
        }
    }

    fn apply_to(&self, source: &mut String) -> Result<bool, &'static str> {
        let replace_end = self
            .replace_start
            .checked_add(self.replace_len)
            .ok_or("invalid source replay")?;
        if replace_end > source.len()
            || !source.is_char_boundary(self.replace_start)
            || !source.is_char_boundary(replace_end)
        {
            return Err("invalid source replay");
        }
        let changed = source[self.replace_start..replace_end] != self.insert;
        source.replace_range(self.replace_start..replace_end, &self.insert);
        Ok(changed)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiAssetEditorUndoDocumentReplay {
    diff: UiAssetDocumentDiff,
}

impl UiAssetEditorUndoDocumentReplay {
    fn between(current: &UiAssetDocument, target: &UiAssetDocument) -> Self {
        Self {
            diff: UiAssetDocumentDiff::between(current, target),
        }
    }

    pub fn apply_to_document(&self, document: &mut UiAssetDocument) -> Result<bool, &'static str> {
        Ok(self.diff.apply_to(document))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SourceEditEntry {
    label: String,
    tree_edit: Option<UiAssetEditorTreeEdit>,
    inverse_tree_edit: Option<UiAssetEditorInverseTreeEdit>,
    undo: UiAssetEditorUndoTransition,
    redo: UiAssetEditorUndoTransition,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiAssetEditorUndoStack {
    undo_stack: Vec<SourceEditEntry>,
    redo_stack: Vec<SourceEditEntry>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiAssetEditorUndoReplayRecord {
    pub label: String,
    pub transition: UiAssetEditorUndoTransition,
}

impl UiAssetEditorUndoStack {
    fn preview_redo_entry(&self) -> Option<&SourceEditEntry> {
        self.redo_stack.last().or_else(|| self.undo_stack.last())
    }

    pub fn push_edit(
        &mut self,
        label: impl Into<String>,
        tree_edit: Option<UiAssetEditorTreeEdit>,
        document_replay: Option<UiAssetEditorDocumentReplayBundle>,
        before_source: String,
        before_selection: UiDesignerSelectionModel,
        before_source_cursor: UiAssetEditorSourceCursorSnapshot,
        before_selected_theme_source_key: Option<String>,
        before_document: Option<UiAssetDocument>,
        after_source: String,
        after_selection: UiDesignerSelectionModel,
        after_source_cursor: UiAssetEditorSourceCursorSnapshot,
        after_selected_theme_source_key: Option<String>,
        after_document: Option<UiAssetDocument>,
        external_effects: UiAssetEditorUndoExternalEffects,
    ) {
        let inverse_tree_edit = match (
            tree_edit.as_ref(),
            before_document.as_ref(),
            after_document.as_ref(),
        ) {
            (Some(tree_edit), Some(before_document), Some(after_document)) => {
                build_inverse_tree_edit(tree_edit, before_document, after_document)
            }
            _ => None,
        };
        self.undo_stack.push(SourceEditEntry {
            label: label.into(),
            tree_edit,
            inverse_tree_edit,
            undo: UiAssetEditorUndoTransition {
                selection: before_selection,
                source_cursor: before_source_cursor,
                selected_theme_source_key: before_selected_theme_source_key,
                document: match (before_document.as_ref(), after_document.as_ref()) {
                    (Some(before_document), Some(after_document)) => Some(
                        UiAssetEditorUndoDocumentReplay::between(after_document, before_document),
                    ),
                    _ => None,
                },
                document_commands: document_replay
                    .as_ref()
                    .map(|replay| replay.undo.clone())
                    .unwrap_or_default(),
                external_effects: external_effects.undo,
                source: UiAssetEditorSourceReplay::between(&after_source, &before_source),
            },
            redo: UiAssetEditorUndoTransition {
                selection: after_selection,
                source_cursor: after_source_cursor,
                selected_theme_source_key: after_selected_theme_source_key,
                document: match (before_document.as_ref(), after_document.as_ref()) {
                    (Some(before_document), Some(after_document)) => Some(
                        UiAssetEditorUndoDocumentReplay::between(before_document, after_document),
                    ),
                    _ => None,
                },
                document_commands: document_replay
                    .as_ref()
                    .map(|replay| replay.redo.clone())
                    .unwrap_or_default(),
                external_effects: external_effects.redo,
                source: UiAssetEditorSourceReplay::between(&before_source, &after_source),
            },
        });
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<UiAssetEditorUndoTransition> {
        self.undo_record().map(|record| record.transition)
    }

    pub fn redo(&mut self) -> Option<UiAssetEditorUndoTransition> {
        self.redo_record().map(|record| record.transition)
    }

    pub fn undo_record(&mut self) -> Option<UiAssetEditorUndoReplayRecord> {
        let entry = self.undo_stack.pop()?;
        let record = UiAssetEditorUndoReplayRecord {
            label: entry.label.clone(),
            transition: entry.undo.clone(),
        };
        self.redo_stack.push(entry);
        Some(record)
    }

    pub fn redo_record(&mut self) -> Option<UiAssetEditorUndoReplayRecord> {
        let entry = self.redo_stack.pop()?;
        let record = UiAssetEditorUndoReplayRecord {
            label: entry.label.clone(),
            transition: entry.redo.clone(),
        };
        self.undo_stack.push(entry);
        Some(record)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn next_undo_label(&self) -> Option<String> {
        self.undo_stack.last().map(|entry| entry.label.clone())
    }

    pub fn next_redo_label(&self) -> Option<String> {
        self.preview_redo_entry().map(|entry| entry.label.clone())
    }

    pub fn next_undo_tree_edit_kind(&self) -> Option<UiAssetEditorTreeEditKind> {
        self.undo_stack
            .last()
            .and_then(|entry| entry.tree_edit.as_ref().map(UiAssetEditorTreeEdit::kind))
    }

    pub fn next_redo_tree_edit_kind(&self) -> Option<UiAssetEditorTreeEditKind> {
        self.preview_redo_entry()
            .and_then(|entry| entry.tree_edit.as_ref().map(UiAssetEditorTreeEdit::kind))
    }

    pub fn next_undo_tree_edit(&self) -> Option<UiAssetEditorTreeEdit> {
        self.undo_stack
            .last()
            .and_then(|entry| entry.tree_edit.clone())
    }

    pub fn next_redo_tree_edit(&self) -> Option<UiAssetEditorTreeEdit> {
        self.preview_redo_entry()
            .and_then(|entry| entry.tree_edit.clone())
    }

    pub fn next_undo_inverse_tree_edit(&self) -> Option<UiAssetEditorInverseTreeEdit> {
        self.undo_stack
            .last()
            .and_then(|entry| entry.inverse_tree_edit.clone())
    }

    pub fn next_redo_inverse_tree_edit(&self) -> Option<UiAssetEditorInverseTreeEdit> {
        self.preview_redo_entry()
            .and_then(|entry| entry.inverse_tree_edit.clone())
    }

    pub fn next_undo_external_effect(&self) -> Option<UiAssetEditorExternalEffect> {
        self.next_undo_external_effects().into_iter().next()
    }

    pub fn next_redo_external_effect(&self) -> Option<UiAssetEditorExternalEffect> {
        self.next_redo_external_effects().into_iter().next()
    }

    pub fn next_undo_external_effects(&self) -> Vec<UiAssetEditorExternalEffect> {
        self.undo_stack
            .last()
            .map(|entry| entry.undo.external_effects.clone())
            .unwrap_or_default()
    }

    pub fn next_undo_document_replay_commands(&self) -> Vec<UiAssetEditorDocumentReplayCommand> {
        self.undo_stack
            .last()
            .map(|entry| entry.undo.document_commands.clone())
            .unwrap_or_default()
    }

    pub fn next_redo_document_replay_commands(&self) -> Vec<UiAssetEditorDocumentReplayCommand> {
        self.preview_redo_entry()
            .map(|entry| entry.redo.document_commands.clone())
            .unwrap_or_default()
    }

    pub fn next_redo_external_effects(&self) -> Vec<UiAssetEditorExternalEffect> {
        self.preview_redo_entry()
            .map(|entry| entry.redo.external_effects.clone())
            .unwrap_or_default()
    }
}

fn apply_document_replay_command(
    document: &mut UiAssetDocument,
    command: &UiAssetEditorDocumentReplayCommand,
) -> Result<bool, &'static str> {
    match command {
        UiAssetEditorDocumentReplayCommand::SetWidgetImports { references } => {
            if document.imports.widgets == *references {
                return Ok(false);
            }
            document.imports.widgets = references.clone();
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::InsertWidgetImport { index, reference } => {
            let insert_index = (*index).min(document.imports.widgets.len());
            if document
                .imports
                .widgets
                .get(insert_index)
                .is_some_and(|existing| existing == reference)
            {
                return Ok(false);
            }
            document
                .imports
                .widgets
                .insert(insert_index, reference.clone());
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::RemoveWidgetImport { index, reference } => {
            if *index >= document.imports.widgets.len() {
                return Ok(false);
            }
            if document.imports.widgets[*index] != *reference {
                return Err("invalid widget import replay");
            }
            let _ = document.imports.widgets.remove(*index);
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::MoveWidgetImport {
            from_index,
            to_index,
            reference,
        } => {
            if *from_index >= document.imports.widgets.len()
                || *to_index >= document.imports.widgets.len()
            {
                return Ok(false);
            }
            if from_index == to_index {
                return Ok(false);
            }
            if document.imports.widgets[*from_index] != *reference {
                return Err("invalid widget import replay");
            }
            let widget_import = document.imports.widgets.remove(*from_index);
            document.imports.widgets.insert(*to_index, widget_import);
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::SetRoot { root } => {
            if document.root == *root {
                return Ok(false);
            }
            document.root = root.clone();
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::UpsertNode { node_id, node } => {
            if document
                .node(node_id)
                .is_some_and(|existing| existing == node)
            {
                return Ok(false);
            }
            Ok(document.replace_node(node_id, node.clone()))
        }
        UiAssetEditorDocumentReplayCommand::RemoveNode { node_id } => {
            Ok(document.remove_node(node_id).is_some())
        }
        UiAssetEditorDocumentReplayCommand::UpsertComponent {
            component_name,
            component,
        } => {
            if document
                .components
                .get(component_name)
                .is_some_and(|existing| existing == component)
            {
                return Ok(false);
            }
            document
                .components
                .insert(component_name.clone(), component.clone());
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::RemoveComponent { component_name } => {
            Ok(document.components.remove(component_name).is_some())
        }
        UiAssetEditorDocumentReplayCommand::SetNodeBindings { node_id, bindings } => {
            let Some(node) = document.node_mut(node_id) else {
                return Ok(false);
            };
            if node.bindings == *bindings {
                return Ok(false);
            }
            node.bindings = bindings.clone();
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::SetStyleImports { references } => {
            if document.imports.styles == *references {
                return Ok(false);
            }
            document.imports.styles = references.clone();
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::InsertStyleImport { index, reference } => {
            let insert_index = (*index).min(document.imports.styles.len());
            if document
                .imports
                .styles
                .get(insert_index)
                .is_some_and(|existing| existing == reference)
            {
                return Ok(false);
            }
            document
                .imports
                .styles
                .insert(insert_index, reference.clone());
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::RemoveStyleImport { index, reference } => {
            if *index >= document.imports.styles.len() {
                return Ok(false);
            }
            if document.imports.styles[*index] != *reference {
                return Err("invalid style import replay");
            }
            let _ = document.imports.styles.remove(*index);
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::MoveStyleImport {
            from_index,
            to_index,
            reference,
        } => {
            if *from_index >= document.imports.styles.len()
                || *to_index >= document.imports.styles.len()
            {
                return Ok(false);
            }
            if from_index == to_index {
                return Ok(false);
            }
            if document.imports.styles[*from_index] != *reference {
                return Err("invalid style import replay");
            }
            let import = document.imports.styles.remove(*from_index);
            document.imports.styles.insert(*to_index, import);
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::SetStyleTokens { tokens } => {
            if document.tokens == *tokens {
                return Ok(false);
            }
            document.tokens = tokens.clone();
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::UpsertStyleToken { token_name, value } => {
            if document.tokens.get(token_name) == Some(value) {
                return Ok(false);
            }
            document.tokens.insert(token_name.clone(), value.clone());
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::RemoveStyleToken { token_name } => {
            Ok(document.tokens.remove(token_name).is_some())
        }
        UiAssetEditorDocumentReplayCommand::SetStyleSheets { stylesheets } => {
            if document.stylesheets == *stylesheets {
                return Ok(false);
            }
            document.stylesheets = stylesheets.clone();
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::InsertStyleSheet {
            index,
            stylesheet_id,
            stylesheet,
        } => {
            let Some(stylesheet) = stylesheet.clone().or_else(|| {
                Some(zircon_runtime::ui::template::UiStyleSheet {
                    id: stylesheet_id.clone(),
                    rules: Vec::new(),
                })
            }) else {
                return Err("invalid stylesheet replay");
            };
            let insert_index = (*index).min(document.stylesheets.len());
            if document
                .stylesheets
                .get(insert_index)
                .is_some_and(|existing| existing.id == stylesheet.id)
            {
                return Ok(false);
            }
            document.stylesheets.insert(insert_index, stylesheet);
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::RemoveStyleSheet {
            index,
            stylesheet_id,
        } => {
            if *index >= document.stylesheets.len() {
                return Ok(false);
            }
            if document.stylesheets[*index].id != *stylesheet_id {
                return Err("invalid stylesheet replay");
            }
            let _ = document.stylesheets.remove(*index);
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::ReplaceStyleSheet {
            index,
            stylesheet_id,
            stylesheet,
        } => {
            let Some(existing) = document.stylesheets.get_mut(*index) else {
                return Ok(false);
            };
            if existing.id != *stylesheet_id {
                return Err("invalid stylesheet replay");
            }
            if *existing == *stylesheet {
                return Ok(false);
            }
            *existing = stylesheet.clone();
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::MoveStyleSheet {
            from_index,
            to_index,
            stylesheet_id,
        } => {
            if *from_index >= document.stylesheets.len() || *to_index >= document.stylesheets.len()
            {
                return Ok(false);
            }
            if from_index == to_index {
                return Ok(false);
            }
            if document.stylesheets[*from_index].id != *stylesheet_id {
                return Err("invalid stylesheet replay");
            }
            let stylesheet = document.stylesheets.remove(*from_index);
            document.stylesheets.insert(*to_index, stylesheet);
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::InsertStyleRule {
            stylesheet_index,
            index,
            rule,
            ..
        } => {
            let Some(stylesheet) = document.stylesheets.get_mut(*stylesheet_index) else {
                return Ok(false);
            };
            let Some(rule) = rule.clone() else {
                return Err("invalid style rule replay");
            };
            let insert_index = (*index).min(stylesheet.rules.len());
            if stylesheet
                .rules
                .get(insert_index)
                .is_some_and(|existing| *existing == rule)
            {
                return Ok(false);
            }
            stylesheet.rules.insert(insert_index, rule);
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::RemoveStyleRule {
            stylesheet_index,
            index,
            selector,
        } => {
            let Some(stylesheet) = document.stylesheets.get_mut(*stylesheet_index) else {
                return Ok(false);
            };
            if *index >= stylesheet.rules.len() {
                return Ok(false);
            }
            if stylesheet.rules[*index].selector != *selector {
                return Err("invalid style rule replay");
            }
            let _ = stylesheet.rules.remove(*index);
            Ok(true)
        }
        UiAssetEditorDocumentReplayCommand::MoveStyleRule {
            stylesheet_index,
            from_index,
            to_index,
        } => {
            let Some(stylesheet) = document.stylesheets.get_mut(*stylesheet_index) else {
                return Ok(false);
            };
            if *from_index >= stylesheet.rules.len() || *to_index >= stylesheet.rules.len() {
                return Ok(false);
            }
            if from_index == to_index {
                return Ok(false);
            }
            let rule = stylesheet.rules.remove(*from_index);
            stylesheet.rules.insert(*to_index, rule);
            Ok(true)
        }
    }
}

fn build_inverse_tree_edit(
    edit: &UiAssetEditorTreeEdit,
    before_document: &UiAssetDocument,
    after_document: &UiAssetDocument,
) -> Option<UiAssetEditorInverseTreeEdit> {
    match edit {
        UiAssetEditorTreeEdit::MoveNode { node_id, direction } => {
            Some(UiAssetEditorInverseTreeEdit::MoveNode {
                node_id: node_id.clone(),
                direction: inverse_move_direction(direction)?,
            })
        }
        UiAssetEditorTreeEdit::InsertPaletteItem {
            node_id,
            parent_node_id,
            ..
        } => Some(UiAssetEditorInverseTreeEdit::RemoveNode {
            node_id: node_id.clone(),
            parent_node_id: parent_node_id.clone(),
        }),
        UiAssetEditorTreeEdit::ReparentNode {
            node_id, direction, ..
        } => Some(UiAssetEditorInverseTreeEdit::ReparentNode {
            node_id: node_id.clone(),
            parent_node_id: child_parent_id(before_document, node_id),
            direction: inverse_reparent_direction(
                direction,
                before_document,
                after_document,
                node_id,
            )?,
        }),
        UiAssetEditorTreeEdit::WrapNode {
            node_id,
            wrapper_node_id,
            ..
        } => Some(UiAssetEditorInverseTreeEdit::UnwrapNode {
            wrapper_node_id: wrapper_node_id.clone(),
            child_node_id: node_id.clone(),
        }),
        UiAssetEditorTreeEdit::UnwrapNode {
            wrapper_node_id,
            child_node_id,
        } => Some(UiAssetEditorInverseTreeEdit::WrapNode {
            node_id: child_node_id.clone(),
            wrapper_node_id: wrapper_node_id.clone(),
            wrapper_widget_type: before_document.node(wrapper_node_id)?.widget_type.clone()?,
        }),
        UiAssetEditorTreeEdit::ConvertToReference { node_id, .. } => {
            let node = before_document.node(node_id)?;
            Some(UiAssetEditorInverseTreeEdit::RestoreNodeDefinition {
                node_id: node_id.clone(),
                kind: node.kind,
                widget_type: node.widget_type.clone(),
                component: node.component.clone(),
                component_ref: node.component_ref.clone(),
            })
        }
        UiAssetEditorTreeEdit::ExtractComponent {
            node_id,
            component_name,
            component_root_id,
        } => Some(UiAssetEditorInverseTreeEdit::InlineExtractedComponent {
            node_id: node_id.clone(),
            component_name: component_name.clone(),
            component_root_id: component_root_id.clone(),
        }),
        UiAssetEditorTreeEdit::PromoteToExternalWidget {
            source_component_name,
            asset_id,
            component_name,
            document_id,
        } => Some(UiAssetEditorInverseTreeEdit::RestorePromotedComponent {
            source_component_name: source_component_name.clone(),
            asset_id: asset_id.clone(),
            component_name: component_name.clone(),
            document_id: document_id.clone(),
        }),
        _ => None,
    }
}

fn inverse_move_direction(direction: &str) -> Option<String> {
    match direction.to_ascii_lowercase().as_str() {
        "up" => Some(preserve_direction_case(direction, "Down", "down")),
        "down" => Some(preserve_direction_case(direction, "Up", "up")),
        _ => None,
    }
}

fn inverse_reparent_direction(
    direction: &str,
    before_document: &UiAssetDocument,
    after_document: &UiAssetDocument,
    node_id: &str,
) -> Option<String> {
    match direction.to_ascii_lowercase().as_str() {
        "into_previous" | "into_next" => {
            Some(preserve_direction_case(direction, "Outdent", "outdent"))
        }
        "outdent" => inverse_outdent_direction(before_document, after_document, node_id).map(
            |direction_label| {
                preserve_direction_case(direction, direction_label.0, direction_label.1)
            },
        ),
        _ => None,
    }
}

fn inverse_outdent_direction<'a>(
    before_document: &UiAssetDocument,
    after_document: &UiAssetDocument,
    node_id: &str,
) -> Option<(&'a str, &'a str)> {
    let original_parent_id = child_parent_id(before_document, node_id)?;
    let (after_parent_id, after_child_index) = child_index_in_parent(after_document, node_id)?;
    let after_parent = after_document.node(&after_parent_id)?;
    if after_child_index > 0
        && after_parent.children[after_child_index - 1].node.node_id == original_parent_id
    {
        Some(("IntoPrevious", "into_previous"))
    } else if after_child_index + 1 < after_parent.children.len()
        && after_parent.children[after_child_index + 1].node.node_id == original_parent_id
    {
        Some(("IntoNext", "into_next"))
    } else {
        None
    }
}

fn preserve_direction_case(direction: &str, title_case: &str, lower_case: &str) -> String {
    if direction
        .chars()
        .next()
        .map(|ch| ch.is_ascii_uppercase())
        .unwrap_or(false)
    {
        title_case.to_string()
    } else {
        lower_case.to_string()
    }
}

fn child_parent_id(document: &UiAssetDocument, child_id: &str) -> Option<String> {
    child_index_in_parent(document, child_id).map(|(parent_id, _)| parent_id)
}

fn child_index_in_parent(document: &UiAssetDocument, child_id: &str) -> Option<(String, usize)> {
    document.child_index_in_parent(child_id)
}

fn common_prefix_len(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left_char, right_char)| left_char == right_char)
        .map(|(character, _)| character.len_utf8())
        .sum()
}

fn common_suffix_len(left: &str, right: &str) -> usize {
    left.chars()
        .rev()
        .zip(right.chars().rev())
        .take_while(|(left_char, right_char)| left_char == right_char)
        .map(|(character, _)| character.len_utf8())
        .sum()
}
