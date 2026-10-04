use std::collections::BTreeSet;

use crate::core::editing::engine::HistoryDecisionToken;
use crate::core::editor_event::DocumentCloseRevision;
use crate::core::editor_message::DocumentId;
use crate::ui::host::DirtyDocumentToolkitView;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::ViewInstanceId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) enum ClosePromptTarget {
    Project,
    MainWindow,
    FloatingWindow(MainPageId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct DirtyCloseView {
    pub document_id: DocumentId,
    pub dirty_generation: u64,
    pub close_revision: DocumentCloseRevision,
    pub instance_id: ViewInstanceId,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct PendingClosePrompt {
    pub target: ClosePromptTarget,
    pub close_instances: Vec<ViewInstanceId>,
    pub dirty_views: Vec<DirtyCloseView>,
    dirty_project_scene_token: Option<HistoryDecisionToken>,
    save_in_flight: bool,
}

pub(in crate::ui::retained_host::app) struct FloatingClosePermit {
    window_id: MainPageId,
    instance_ids: Vec<ViewInstanceId>,
    discard: Vec<(ViewInstanceId, DocumentId, DocumentCloseRevision)>,
}

impl FloatingClosePermit {
    pub(in crate::ui::retained_host::app) fn into_parts(
        self,
    ) -> (
        MainPageId,
        Vec<ViewInstanceId>,
        Vec<(ViewInstanceId, DocumentId, DocumentCloseRevision)>,
    ) {
        (self.window_id, self.instance_ids, self.discard)
    }
}

impl PendingClosePrompt {
    pub(in crate::ui::retained_host::app) fn into_floating_close_permit(
        self,
    ) -> Option<FloatingClosePermit> {
        let ClosePromptTarget::FloatingWindow(window_id) = self.target else {
            return None;
        };
        Some(FloatingClosePermit {
            window_id,
            instance_ids: self.close_instances,
            discard: self
                .dirty_views
                .into_iter()
                .map(|view| (view.instance_id, view.document_id, view.close_revision))
                .collect(),
        })
    }

    pub(in crate::ui::retained_host::app) fn new(
        target: ClosePromptTarget,
        close_instances: Vec<ViewInstanceId>,
        dirty_views: Vec<DirtyCloseView>,
    ) -> Self {
        Self {
            target,
            close_instances,
            dirty_views,
            dirty_project_scene_token: None,
            save_in_flight: false,
        }
    }

    pub(in crate::ui::retained_host::app) const fn save_in_flight(&self) -> bool {
        self.save_in_flight
    }

    pub(in crate::ui::retained_host::app) fn begin_save(&mut self) {
        self.save_in_flight = true;
    }

    pub(in crate::ui::retained_host::app) fn finish_save(
        &mut self,
        dirty_views: Vec<DirtyCloseView>,
        dirty_project_scene_token: Option<HistoryDecisionToken>,
    ) {
        self.save_in_flight = false;
        self.dirty_views = dirty_views;
        self.dirty_project_scene_token = dirty_project_scene_token;
    }

    /// A failed observation preserves the last decision and permits a later retry.
    pub(in crate::ui::retained_host::app) fn finish_save_failed(&mut self) {
        self.save_in_flight = false;
    }

    pub(in crate::ui::retained_host::app) fn with_dirty_project_scene(
        mut self,
        token: HistoryDecisionToken,
    ) -> Self {
        self.dirty_project_scene_token = Some(token);
        self
    }

    pub(in crate::ui::retained_host::app) const fn has_dirty_project_scene(&self) -> bool {
        self.dirty_project_scene_token.is_some()
    }

    pub(in crate::ui::retained_host::app) fn dirty_participant_count(&self) -> usize {
        self.dirty_views.len() + usize::from(self.has_dirty_project_scene())
    }

    /// A discard action may only consume the documents captured by this plan.
    /// Documents saved after planning are harmless; newly dirty documents and
    /// scene decision changes require a fresh decision instead.
    pub(in crate::ui::retained_host::app) fn permits_discard(
        &self,
        current_dirty_views: &[DirtyCloseView],
        current_project_scene_token: Option<&HistoryDecisionToken>,
    ) -> bool {
        let documents_match = current_dirty_views.iter().all(|current| {
            self.dirty_views.iter().any(|planned| {
                planned.document_id == current.document_id
                    && planned.dirty_generation == current.dirty_generation
                    && planned.close_revision == current.close_revision
                    && planned.instance_id == current.instance_id
            })
        });
        let scene_matches = current_project_scene_token
            .is_none_or(|token| self.dirty_project_scene_token.as_ref() == Some(token));
        documents_match && scene_matches
    }
}

pub(in crate::ui::retained_host::app) fn dirty_close_views(
    documents: &[DirtyDocumentToolkitView],
    candidate_ids: impl IntoIterator<Item = ViewInstanceId>,
) -> Vec<DirtyCloseView> {
    let candidates = candidate_ids.into_iter().collect::<BTreeSet<_>>();
    documents
        .iter()
        .filter(|document| candidates.contains(&document.instance_id))
        .map(dirty_close_view_from_document)
        .collect()
}

pub(in crate::ui::retained_host::app) fn all_dirty_close_views(
    documents: &[DirtyDocumentToolkitView],
) -> Vec<DirtyCloseView> {
    documents
        .iter()
        .map(dirty_close_view_from_document)
        .collect()
}

fn dirty_close_view_from_document(document: &DirtyDocumentToolkitView) -> DirtyCloseView {
    DirtyCloseView {
        document_id: document.document_id,
        dirty_generation: document.dirty_generation,
        close_revision: document.close_revision,
        instance_id: document.instance_id.clone(),
        title: document.title.clone(),
    }
}

#[cfg(test)]
#[path = "tests/model.rs"]
mod tests;
