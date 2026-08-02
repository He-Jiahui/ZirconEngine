use std::collections::BTreeSet;

use crate::core::editor_message::DocumentId;
use crate::ui::host::DirtyDocumentToolkitView;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::ViewInstanceId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) enum ClosePromptTarget {
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
        }
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
