use std::{borrow::Cow, collections::BTreeMap, ops::Range, sync::Arc};

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    surface::UiEditableTextState,
    text::{UiTextByteSelection, UiTextDocumentKey, UiTextEditKind, UiTextEditSource},
    tree::UiTree,
};

use crate::{
    text::document::{
        PreparedTextDocumentStoreEdit, TextDocumentStore, TextDocumentStoreEditCommit,
    },
    ui::{surface::UiSurfaceSessionIdentityHandle, text::CommittedTextEditIntent},
};

use super::{
    binding::{UiTextDocumentBinding, UiTextDocumentBindingKey},
    error::UiTextDocumentSessionError,
    history::{
        UiTextDocumentHistory, UiTextHistoryCommit, UiTextHistoryDirection, UiTextHistoryEntry,
        MVP_TEXT_HISTORY_MAX_DELTA_BYTES,
    },
    limits::mvp_text_document_store_limits,
};

pub(in crate::ui) struct UiTextDocumentSession {
    active_surface: Option<UiSurfaceSessionIdentityHandle>,
    active_tree: Option<UiTreeId>,
    store: TextDocumentStore,
    bindings: BTreeMap<UiTextDocumentBindingKey, UiTextDocumentBinding>,
    histories: BTreeMap<UiTextDocumentBindingKey, UiTextDocumentHistory>,
    synchronization_errors: BTreeMap<UiTextDocumentBindingKey, UiTextDocumentSessionError>,
    observed_layout_order_generation: Option<u64>,
    observed_node_count: usize,
}

impl Default for UiTextDocumentSession {
    fn default() -> Self {
        Self {
            active_surface: None,
            active_tree: None,
            store: TextDocumentStore::with_limits(mvp_text_document_store_limits()),
            bindings: BTreeMap::new(),
            histories: BTreeMap::new(),
            synchronization_errors: BTreeMap::new(),
            observed_layout_order_generation: None,
            observed_node_count: 0,
        }
    }
}

impl UiTextDocumentSession {
    pub(in crate::ui) fn document_key(
        &self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
    ) -> Result<UiTextDocumentKey, UiTextDocumentSessionError> {
        let binding = self.binding(tree_id, node_id, source_epoch)?;
        Ok(UiTextDocumentKey {
            document_id: binding.document_id,
            revision: binding.revision,
        })
    }

    pub(in crate::ui) fn retained_grapheme_count(
        &mut self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        range: Range<usize>,
    ) -> Result<usize, UiTextDocumentSessionError> {
        let binding = self.binding(tree_id, node_id, source_epoch)?;
        self.store
            .retained_grapheme_count(binding.document_id, binding.revision, range)
            .map_err(Into::into)
    }

    pub(in crate::ui) fn synchronize_owners(
        &mut self,
        tree: &UiTree,
        surface_identity: UiSurfaceSessionIdentityHandle,
    ) {
        self.activate_surface(&tree.tree_id, surface_identity);
        let topology_changed = self.observed_layout_order_generation
            != Some(tree.layout_order_generation())
            || self.observed_node_count != tree.nodes.len();
        if topology_changed {
            let detached = self
                .bindings
                .keys()
                .filter(|key| !tree.nodes.contains_key(&key.node_id))
                .cloned()
                .collect::<Vec<_>>();
            for key in detached {
                self.close_binding(&key);
            }
        } else {
            let detached = tree
                .pending_mutation_node_ids()
                .iter()
                .filter(|node_id| !tree.nodes.contains_key(node_id))
                .map(|node_id| UiTextDocumentBindingKey {
                    tree_id: tree.tree_id.clone(),
                    node_id: *node_id,
                })
                .collect::<Vec<_>>();
            for key in detached {
                self.close_binding(&key);
            }
        }
        self.observed_layout_order_generation = Some(tree.layout_order_generation());
        self.observed_node_count = tree.nodes.len();
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::ui) fn prepare_edit<'session>(
        &'session mut self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        intent: &CommittedTextEditIntent,
        next_state: &UiEditableTextState,
        source: UiTextEditSource,
    ) -> Result<PreparedTextDocumentStoreEdit<'session>, UiTextDocumentSessionError> {
        if source_epoch.checked_add(1).is_none() {
            return Err(UiTextDocumentSessionError::SourceEpochExhausted);
        }
        let replacement = intent
            .replacement(next_state)
            .ok_or(UiTextDocumentSessionError::InvalidEditIntent)?;
        let selection = public_selection(next_state)?;
        let key = UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        };
        self.activate_tree(tree_id);
        let binding = self
            .bindings
            .get(&key)
            .copied()
            .filter(|binding| binding.source_epoch == source_epoch)
            .ok_or_else(|| {
                self.synchronization_errors
                    .get(&key)
                    .copied()
                    .unwrap_or(UiTextDocumentSessionError::SourceNotSynchronized)
            })?;
        self.store
            .prepare_replace_with_receipt(
                binding.document_id,
                binding.revision,
                intent.old.clone(),
                replacement,
                node_id,
                source,
                intent.kind,
                selection,
            )
            .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::ui) fn prepare_history_commit(
        &self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        intent: &CommittedTextEditIntent,
        current_state: &UiEditableTextState,
        next_state: &UiEditableTextState,
        secure: bool,
    ) -> Result<UiTextHistoryCommit, UiTextDocumentSessionError> {
        if secure {
            return Ok(UiTextHistoryCommit::Barrier);
        }
        match intent.kind {
            UiTextEditKind::Undo => return Ok(UiTextHistoryCommit::Undo),
            UiTextEditKind::Redo => return Ok(UiTextHistoryCommit::Redo),
            _ => {}
        }
        let binding = self.binding(tree_id, node_id, source_epoch)?;
        let inserted = intent
            .replacement(next_state)
            .ok_or(UiTextDocumentSessionError::InvalidEditIntent)?;
        let retained_bytes = intent.old.len().checked_add(inserted.len());
        if retained_bytes.map_or(true, |bytes| bytes > MVP_TEXT_HISTORY_MAX_DELTA_BYTES) {
            return Ok(UiTextHistoryCommit::Barrier);
        }
        let removed =
            self.store
                .source_range(binding.document_id, binding.revision, intent.old.clone())?;
        Ok(UiTextHistoryCommit::Record(UiTextHistoryEntry::new(
            intent,
            removed,
            inserted.to_string(),
            current_state,
            next_state,
        )))
    }

    pub(in crate::ui) fn prepare_history_transition(
        &self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        current_state: UiEditableTextState,
        direction: UiTextHistoryDirection,
    ) -> Result<Option<crate::ui::text::TextEditStateTransition>, UiTextDocumentSessionError> {
        let binding = self.binding(tree_id, node_id, source_epoch)?;
        let key = UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        };
        let Some(entry) = self
            .histories
            .get(&key)
            .and_then(|history| history.latest(direction))
        else {
            return Ok(None);
        };
        let range = entry.expected_range(direction);
        let source = self
            .store
            .source_range(binding.document_id, binding.revision, range)?;
        if source != entry.expected_text(direction) {
            return Err(UiTextDocumentSessionError::InvalidEditIntent);
        }
        entry
            .transition(current_state, direction)
            .map(Some)
            .ok_or(UiTextDocumentSessionError::InvalidEditIntent)
    }

    pub(in crate::ui) fn discard_history(&mut self, tree_id: &UiTreeId, node_id: UiNodeId) {
        self.histories.remove(&UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        });
    }

    pub(in crate::ui) fn discard_all_histories(&mut self) {
        self.histories.clear();
    }

    pub(in crate::ui) fn committed_source_matches(
        &self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        candidate: &str,
    ) -> Result<bool, UiTextDocumentSessionError> {
        let binding = self.binding(tree_id, node_id, source_epoch)?;
        self.store
            .source_equals(binding.document_id, binding.revision, candidate)
            .map_err(Into::into)
    }

    pub(in crate::ui) fn synchronize_editable_source(
        &mut self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        state: &UiEditableTextState,
    ) {
        self.activate_tree(tree_id);
        let key = UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        };
        if self
            .bindings
            .get(&key)
            .is_some_and(|binding| binding.source_epoch == source_epoch)
        {
            self.synchronization_errors.remove(&key);
            return;
        }

        // An active preedit is a visible projection. A new session must open its
        // restored source, while an existing source lease avoids this materialization.
        let source = match state.composition.as_ref().and_then(|composition| {
            composition
                .restore_text
                .as_deref()
                .map(|restore| (composition.range, restore))
        }) {
            Some((range, restore)) => {
                let Some(replaced) = state.text.get(range.start..range.end) else {
                    self.synchronization_errors
                        .insert(key, UiTextDocumentSessionError::InvalidEditIntent);
                    return;
                };
                if replaced == restore {
                    Cow::Borrowed(state.text.as_str())
                } else {
                    let mut source =
                        String::with_capacity(state.text.len() - replaced.len() + restore.len());
                    source.push_str(&state.text[..range.start]);
                    source.push_str(restore);
                    source.push_str(&state.text[range.end..]);
                    Cow::Owned(source)
                }
            }
            None => Cow::Borrowed(state.text.as_str()),
        };
        self.synchronize_source(tree_id, node_id, source_epoch, source.as_ref());
    }

    pub(in crate::ui) fn synchronize_source(
        &mut self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        current_text: &str,
    ) {
        self.activate_tree(tree_id);
        let key = UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        };
        if self
            .bindings
            .get(&key)
            .is_some_and(|binding| binding.source_epoch == source_epoch)
        {
            self.synchronization_errors.remove(&key);
            return;
        }
        if let Some(binding) = self.bindings.remove(&key) {
            self.store.close(binding.document_id);
        }
        self.histories.remove(&key);
        match self.store.open(Arc::<str>::from(current_text)) {
            Ok(opened) => {
                self.bindings.insert(
                    key.clone(),
                    UiTextDocumentBinding {
                        document_id: opened.document_id,
                        revision: opened.revision,
                        source_epoch,
                    },
                );
                self.synchronization_errors.remove(&key);
            }
            Err(error) => {
                self.synchronization_errors.insert(key, error.into());
            }
        }
    }

    pub(in crate::ui) fn finish_edit(
        &mut self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        commit: &TextDocumentStoreEditCommit,
        history_commit: UiTextHistoryCommit,
    ) {
        let (document_id, revision) = match commit {
            TextDocumentStoreEditCommit::Unchanged {
                document_id,
                revision,
            } => (*document_id, *revision),
            TextDocumentStoreEditCommit::Changed { public_receipt, .. } => {
                (public_receipt.document_id, public_receipt.revision)
            }
        };
        let key = UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        };
        self.synchronization_errors.remove(&key);
        if matches!(commit, TextDocumentStoreEditCommit::Changed { .. }) {
            match history_commit {
                UiTextHistoryCommit::Barrier => {
                    self.histories.remove(&key);
                }
                history_commit => self
                    .histories
                    .entry(key.clone())
                    .or_default()
                    .commit(history_commit),
            }
        }
        self.bindings.insert(
            key,
            UiTextDocumentBinding {
                document_id,
                revision,
                source_epoch,
            },
        );
    }

    fn binding(
        &self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
    ) -> Result<UiTextDocumentBinding, UiTextDocumentSessionError> {
        let key = UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        };
        self.bindings
            .get(&key)
            .copied()
            .filter(|binding| binding.source_epoch == source_epoch)
            .ok_or_else(|| {
                self.synchronization_errors
                    .get(&key)
                    .copied()
                    .unwrap_or(UiTextDocumentSessionError::SourceNotSynchronized)
            })
    }

    fn activate_tree(&mut self, tree_id: &UiTreeId) {
        if self.active_tree.as_ref() == Some(tree_id) {
            return;
        }
        self.reset_bindings();
        self.active_surface = None;
        self.active_tree = Some(tree_id.clone());
    }

    fn activate_surface(
        &mut self,
        tree_id: &UiTreeId,
        surface_identity: UiSurfaceSessionIdentityHandle,
    ) {
        if self.active_tree.as_ref() == Some(tree_id)
            && self.active_surface.as_ref() == Some(&surface_identity)
        {
            return;
        }
        self.reset_bindings();
        self.active_surface = Some(surface_identity);
        self.active_tree = Some(tree_id.clone());
    }

    fn reset_bindings(&mut self) {
        for binding in self.bindings.values() {
            self.store.close(binding.document_id);
        }
        self.bindings.clear();
        self.histories.clear();
        self.synchronization_errors.clear();
        self.observed_layout_order_generation = None;
        self.observed_node_count = 0;
    }

    fn close_binding(&mut self, key: &UiTextDocumentBindingKey) {
        if let Some(binding) = self.bindings.remove(key) {
            self.store.close(binding.document_id);
        }
        self.histories.remove(key);
        self.synchronization_errors.remove(key);
    }
}

fn public_selection(
    state: &UiEditableTextState,
) -> Result<UiTextByteSelection, UiTextDocumentSessionError> {
    let (anchor, focus) = state
        .selection
        .as_ref()
        .map(|selection| (selection.anchor, selection.focus))
        .unwrap_or((state.caret.offset, state.caret.offset));
    Ok(UiTextByteSelection {
        anchor_byte: u32::try_from(anchor)
            .map_err(|_| UiTextDocumentSessionError::ByteOffsetOverflow)?,
        focus_byte: u32::try_from(focus)
            .map_err(|_| UiTextDocumentSessionError::ByteOffsetOverflow)?,
        focus_affinity: state.caret.affinity,
    })
}

#[cfg(test)]
#[path = "session/tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "session/tests/committed_source_tests.rs"]
mod committed_source_tests;
