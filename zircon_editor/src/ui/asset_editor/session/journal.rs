use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{
    command::{
        UiAssetEditorCommand, UiAssetEditorDocumentReplayBundle, UiAssetEditorTreeEdit,
        UiAssetEditorTreeEditKind,
    },
    ui_asset_editor_session::{UiAssetEditorSession, UiAssetEditorSessionError},
    undo_stack::UiAssetEditorUndoExternalEffects,
};
use crate::ui::asset_editor::UiDesignerSelectionModel;

pub const UI_ASSET_EDITOR_COMMAND_JOURNAL_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAssetEditorCommandJournal {
    pub schema_version: u32,
    #[serde(default)]
    pub entries: Vec<UiAssetEditorCommandJournalEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiAssetEditorCommandJournalEntry {
    pub sequence: usize,
    pub command: UiAssetEditorJournalCommand,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UiAssetEditorJournalCommand {
    SourceEdit {
        label: String,
        next_source: String,
        #[serde(default)]
        external_effects: UiAssetEditorUndoExternalEffects,
    },
    TreeEdit {
        edit: UiAssetEditorTreeEdit,
        label: String,
        next_source: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        next_selection: Option<UiDesignerSelectionModel>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        document_replay: Option<UiAssetEditorDocumentReplayBundle>,
        #[serde(default)]
        external_effects: UiAssetEditorUndoExternalEffects,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAssetEditorCommandJournalReplayReport {
    pub applied_entries: usize,
    pub labels: Vec<String>,
}

#[derive(Debug, Error)]
pub enum UiAssetEditorCommandJournalReplayError {
    #[error(
        "unsupported ui asset editor command journal schema version {actual}; expected {expected}"
    )]
    UnsupportedSchemaVersion { expected: u32, actual: u32 },
    #[error("ui asset editor command journal sequence {current} must be greater than {previous}")]
    NonMonotonicSequence { previous: usize, current: usize },
    #[error(transparent)]
    Session(#[from] UiAssetEditorSessionError),
}

impl UiAssetEditorCommandJournal {
    pub fn new(entries: Vec<UiAssetEditorCommandJournalEntry>) -> Self {
        Self {
            schema_version: UI_ASSET_EDITOR_COMMAND_JOURNAL_SCHEMA_VERSION,
            entries,
        }
    }
}

impl UiAssetEditorCommandJournalEntry {
    pub fn source_edit(
        sequence: usize,
        label: impl Into<String>,
        next_source: impl Into<String>,
    ) -> Self {
        Self {
            sequence,
            command: UiAssetEditorJournalCommand::SourceEdit {
                label: label.into(),
                next_source: next_source.into(),
                external_effects: UiAssetEditorUndoExternalEffects::default(),
            },
        }
    }

    pub fn tree_edit(
        sequence: usize,
        edit: UiAssetEditorTreeEdit,
        label: impl Into<String>,
        next_source: impl Into<String>,
    ) -> Self {
        Self {
            sequence,
            command: UiAssetEditorJournalCommand::TreeEdit {
                edit,
                label: label.into(),
                next_source: next_source.into(),
                next_selection: None,
                document_replay: None,
                external_effects: UiAssetEditorUndoExternalEffects::default(),
            },
        }
    }

    pub fn with_external_effects(mut self, effects: UiAssetEditorUndoExternalEffects) -> Self {
        match &mut self.command {
            UiAssetEditorJournalCommand::SourceEdit {
                external_effects, ..
            }
            | UiAssetEditorJournalCommand::TreeEdit {
                external_effects, ..
            } => *external_effects = effects,
        }
        self
    }
}

impl UiAssetEditorJournalCommand {
    fn label(&self) -> &str {
        match self {
            Self::SourceEdit { label, .. } | Self::TreeEdit { label, .. } => label,
        }
    }

    fn external_effects(&self) -> UiAssetEditorUndoExternalEffects {
        match self {
            Self::SourceEdit {
                external_effects, ..
            }
            | Self::TreeEdit {
                external_effects, ..
            } => external_effects.clone(),
        }
    }

    fn to_editor_command(&self) -> UiAssetEditorCommand {
        match self {
            Self::SourceEdit {
                label, next_source, ..
            } => UiAssetEditorCommand::tree_edit(
                UiAssetEditorTreeEditKind::DocumentEdit,
                label.clone(),
                next_source.clone(),
            ),
            Self::TreeEdit {
                edit,
                label,
                next_source,
                next_selection,
                document_replay,
                ..
            } => {
                let mut command = if let Some(next_selection) = next_selection.clone() {
                    UiAssetEditorCommand::tree_edit_structured_with_selection(
                        edit.clone(),
                        label.clone(),
                        next_source.clone(),
                        next_selection,
                    )
                } else {
                    UiAssetEditorCommand::tree_edit_structured(
                        edit.clone(),
                        label.clone(),
                        next_source.clone(),
                    )
                };
                if let Some(document_replay) = document_replay.clone() {
                    command = command.with_document_replay(document_replay);
                }
                command
            }
        }
    }
}

impl UiAssetEditorSession {
    pub fn apply_command_journal(
        &mut self,
        journal: &UiAssetEditorCommandJournal,
    ) -> Result<UiAssetEditorCommandJournalReplayReport, UiAssetEditorCommandJournalReplayError>
    {
        if journal.schema_version != UI_ASSET_EDITOR_COMMAND_JOURNAL_SCHEMA_VERSION {
            return Err(
                UiAssetEditorCommandJournalReplayError::UnsupportedSchemaVersion {
                    expected: UI_ASSET_EDITOR_COMMAND_JOURNAL_SCHEMA_VERSION,
                    actual: journal.schema_version,
                },
            );
        }

        let mut previous_sequence = 0;
        for entry in &journal.entries {
            if entry.sequence <= previous_sequence {
                return Err(
                    UiAssetEditorCommandJournalReplayError::NonMonotonicSequence {
                        previous: previous_sequence,
                        current: entry.sequence,
                    },
                );
            }
            previous_sequence = entry.sequence;
        }

        let mut labels = Vec::with_capacity(journal.entries.len());
        for entry in &journal.entries {
            labels.push(entry.command.label().to_string());
            self.apply_command_with_effects(
                entry.command.to_editor_command(),
                entry.command.external_effects(),
            )?;
        }

        Ok(UiAssetEditorCommandJournalReplayReport {
            applied_entries: labels.len(),
            labels,
        })
    }
}
