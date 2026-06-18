use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstance, ViewInstanceId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) enum ClosePromptTarget {
    MainWindow,
    FloatingWindow(MainPageId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct DirtyCloseView {
    pub instance_id: ViewInstanceId,
    pub descriptor_id: ViewDescriptorId,
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
    instances: &[ViewInstance],
    candidate_ids: impl IntoIterator<Item = ViewInstanceId>,
) -> Vec<DirtyCloseView> {
    let candidates = candidate_ids.into_iter().collect::<Vec<_>>();
    instances
        .iter()
        .filter(|instance| {
            instance.dirty && candidates.iter().any(|id| id == &instance.instance_id)
        })
        .map(dirty_close_view_from_instance)
        .collect()
}

pub(in crate::ui::retained_host::app) fn all_dirty_close_views(
    instances: &[ViewInstance],
) -> Vec<DirtyCloseView> {
    instances
        .iter()
        .filter(|instance| instance.dirty)
        .map(dirty_close_view_from_instance)
        .collect()
}

pub(in crate::ui::retained_host::app) fn can_save_dirty_view(view: &DirtyCloseView) -> bool {
    matches!(
        view.descriptor_id.0.as_str(),
        "editor.ui_asset" | "editor.animation_sequence" | "editor.animation_graph"
    )
}

fn dirty_close_view_from_instance(instance: &ViewInstance) -> DirtyCloseView {
    DirtyCloseView {
        instance_id: instance.instance_id.clone(),
        descriptor_id: instance.descriptor_id.clone(),
        title: instance.title.clone(),
    }
}
