use crate::core::asset::{DirtyExternalEffectId, DirtyExternalEffectRevision};
use crate::core::editor_event::DocumentCloseRevision;
use crate::core::editor_message::DocumentId;
use crate::core::extension::{
    DocumentToolkitDescriptor, DocumentToolkitSnapshot, ToolkitInstanceId,
};
use crate::ui::workbench::layout::{LayoutCommand, MainPageId};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstanceId};
use crate::ui::workbench::LayoutPresetRestoreResult;

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DirtyDocumentToolkitView {
    pub(crate) document_id: DocumentId,
    pub(crate) dirty_generation: u64,
    pub(crate) close_revision: DocumentCloseRevision,
    pub(crate) instance_id: ViewInstanceId,
    pub(crate) title: String,
}

impl EditorManager {
    pub fn apply_layout_command(&self, cmd: LayoutCommand) -> Result<bool, EditorError> {
        if let LayoutCommand::CloseView { instance_id } = &cmd {
            return self.close_view(instance_id);
        }
        if let LayoutCommand::CloseViews {
            window_id,
            instance_ids,
        } = &cmd
        {
            return self.close_views_with_discard(window_id, instance_ids, &[]);
        }
        self.host.apply_layout_command(cmd)
    }

    pub fn open_view(
        &self,
        descriptor_id: ViewDescriptorId,
        target_host: Option<ViewHost>,
    ) -> Result<ViewInstanceId, EditorError> {
        self.host.open_view(descriptor_id, target_host)
    }

    pub fn close_view(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        let save = self
            .dirty_save
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if save.owner().is_some() {
            return Err(EditorError::DocumentCloseSaveInProgress);
        }
        self.host.close_view(instance_id)
    }

    pub(crate) fn close_view_discarding(
        &self,
        instance_id: &ViewInstanceId,
        document: DocumentId,
        close_revision: DocumentCloseRevision,
    ) -> Result<bool, EditorError> {
        let save = self
            .dirty_save
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if save.owner().is_some() {
            return Err(EditorError::DocumentCloseSaveInProgress);
        }
        self.host
            .close_view_discarding(instance_id, document, close_revision)
    }

    pub(crate) fn close_views_with_discard(
        &self,
        window_id: &MainPageId,
        instance_ids: &[ViewInstanceId],
        discard: &[(ViewInstanceId, DocumentId, DocumentCloseRevision)],
    ) -> Result<bool, EditorError> {
        let save = self
            .dirty_save
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if save.owner().is_some() {
            return Err(EditorError::DocumentCloseSaveInProgress);
        }
        self.host
            .close_views_with_discard(window_id, instance_ids, discard)
    }

    pub fn focus_view(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        self.host.focus_view(instance_id)
    }

    pub fn detach_view_to_window(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        self.host.detach_view_to_window(instance_id)
    }

    pub fn attach_view_to_target(
        &self,
        instance_id: &ViewInstanceId,
        drop_target: ViewHost,
    ) -> Result<bool, EditorError> {
        self.host.attach_view_to_target(instance_id, drop_target)
    }

    pub fn save_global_default_layout(&self) -> Result<(), EditorError> {
        self.host.save_global_default_layout()
    }

    pub fn save_page_layout(&self, user_id: &str, page_id: &MainPageId) -> Result<(), EditorError> {
        self.host.save_page_layout(user_id, page_id)
    }

    pub fn restore_page_layout(
        &self,
        user_id: &str,
        page_id: &MainPageId,
    ) -> Result<LayoutPresetRestoreResult, EditorError> {
        self.host.restore_page_layout(user_id, page_id)
    }

    pub fn preset_names(&self) -> Result<Vec<String>, EditorError> {
        self.host.preset_names()
    }

    pub fn document_toolkit_snapshot(&self) -> DocumentToolkitSnapshot {
        self.host.document_toolkit_snapshot()
    }

    pub(crate) fn focused_document_toolkit(&self) -> Option<DocumentToolkitDescriptor> {
        let focused = self.current_focused_view()?;
        let instance = ToolkitInstanceId::parse(focused.0).ok()?;
        self.document_toolkit_snapshot()
            .descriptor_for_instance(&instance)
            .cloned()
    }

    pub(crate) fn dirty_document_toolkits(
        &self,
    ) -> Result<Vec<DirtyDocumentToolkitView>, EditorError> {
        let toolkits = self.host.document_toolkit_snapshot();
        let mut dirty = Vec::new();
        for descriptor in toolkits.descriptors() {
            let instance_id = ViewInstanceId::new(descriptor.instance_id().as_str());
            let (is_dirty, close_revision) = self
                .host
                .document_close_state(&instance_id, descriptor.document_id())?;
            if is_dirty {
                dirty.push(DirtyDocumentToolkitView {
                    document_id: descriptor.document_id(),
                    dirty_generation: close_revision.external_generation,
                    close_revision,
                    instance_id,
                    title: descriptor.title().to_string(),
                });
            }
        }
        Ok(dirty)
    }

    pub(crate) fn mark_document_external_effect(
        &self,
        instance_id: &ViewInstanceId,
        effect: DirtyExternalEffectId,
    ) -> Result<DirtyExternalEffectRevision, EditorError> {
        self.host.mark_document_external_effect(instance_id, effect)
    }
}
