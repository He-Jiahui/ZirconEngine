use std::collections::BTreeSet;

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
    pub instance_id: ViewInstanceId,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct PendingClosePrompt {
    pub target: ClosePromptTarget,
    pub close_instances: Vec<ViewInstanceId>,
    pub dirty_views: Vec<DirtyCloseView>,
    dirty_project_scene_generation: Option<u64>,
    save_in_flight: bool,
}

impl PendingClosePrompt {
    pub(in crate::ui::retained_host::app) fn new(
        target: ClosePromptTarget,
        close_instances: Vec<ViewInstanceId>,
        dirty_views: Vec<DirtyCloseView>,
    ) -> Self {
        Self {
            target,
            close_instances,
            dirty_views,
            dirty_project_scene_generation: None,
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
        dirty_project_scene_generation: Option<u64>,
    ) {
        self.save_in_flight = false;
        self.dirty_views = dirty_views;
        self.dirty_project_scene_generation = dirty_project_scene_generation;
    }

    pub(in crate::ui::retained_host::app) fn with_dirty_project_scene(
        mut self,
        generation: u64,
    ) -> Self {
        self.dirty_project_scene_generation = Some(generation);
        self
    }

    pub(in crate::ui::retained_host::app) const fn has_dirty_project_scene(&self) -> bool {
        self.dirty_project_scene_generation.is_some()
    }

    pub(in crate::ui::retained_host::app) fn dirty_participant_count(&self) -> usize {
        self.dirty_views.len() + usize::from(self.has_dirty_project_scene())
    }

    /// A discard action may only consume the documents captured by this plan.
    /// Documents saved after planning are harmless; newly dirty documents and
    /// generation changes require a fresh decision instead.
    pub(in crate::ui::retained_host::app) fn permits_discard(
        &self,
        current_dirty_views: &[DirtyCloseView],
        current_project_scene_generation: Option<u64>,
    ) -> bool {
        let documents_match = current_dirty_views.iter().all(|current| {
            self.dirty_views.iter().any(|planned| {
                planned.document_id == current.document_id
                    && planned.dirty_generation == current.dirty_generation
                    && planned.instance_id == current.instance_id
            })
        });
        let scene_matches = current_project_scene_generation
            .is_none_or(|generation| self.dirty_project_scene_generation == Some(generation));
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
        instance_id: document.instance_id.clone(),
        title: document.title.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::core::editor_message::DocumentId;
    use crate::ui::workbench::view::ViewInstanceId;

    use super::{ClosePromptTarget, DirtyCloseView, PendingClosePrompt};

    fn dirty_view(document: u64, generation: u64, instance: &str) -> DirtyCloseView {
        DirtyCloseView {
            document_id: DocumentId::new(document),
            dirty_generation: generation,
            instance_id: ViewInstanceId::new(instance),
            title: instance.to_string(),
        }
    }

    #[test]
    fn discard_requires_every_current_dirty_document_to_match_the_captured_plan() {
        let planned = dirty_view(7, 3, "editor.asset#7");
        let prompt = PendingClosePrompt::new(
            ClosePromptTarget::Project,
            Vec::new(),
            vec![planned.clone()],
        );

        assert!(prompt.permits_discard(&[planned], None));
        assert!(!prompt.permits_discard(&[dirty_view(7, 4, "editor.asset#7")], None));
        assert!(!prompt.permits_discard(&[dirty_view(8, 1, "editor.asset#8")], None));
    }

    #[test]
    fn discard_requires_a_dirty_scene_generation_to_match_the_captured_plan() {
        let prompt = PendingClosePrompt::new(ClosePromptTarget::Project, Vec::new(), Vec::new())
            .with_dirty_project_scene(11);

        assert!(prompt.permits_discard(&[], Some(11)));
        assert!(prompt.permits_discard(&[], None));
        assert!(!prompt.permits_discard(&[], Some(12)));
    }
}
