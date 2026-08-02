use std::collections::BTreeMap;

use crate::core::asset::{DirtyExternalEffectId, DirtyExternalEffectRevision};
use crate::core::editor_message::DocumentId;
use crate::core::extension::DocumentToolkitSnapshot;
use crate::ui::workbench::layout::{LayoutCommand, MainPageId};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstanceId};
use crate::ui::workbench::LayoutPresetRestoreResult;

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DirtyDocumentToolkitView {
    pub(crate) document_id: DocumentId,
    pub(crate) dirty_generation: u64,
    pub(crate) instance_id: ViewInstanceId,
    pub(crate) title: String,
}

impl EditorManager {
    pub fn apply_layout_command(&self, cmd: LayoutCommand) -> Result<bool, EditorError> {
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
        self.host.close_view(instance_id)
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

    pub(crate) fn dirty_document_toolkits(
        &self,
    ) -> Result<Vec<DirtyDocumentToolkitView>, EditorError> {
        let toolkits = self.host.document_toolkit_snapshot();
        let descriptors = toolkits
            .descriptors()
            .iter()
            .map(|descriptor| (descriptor.document_id(), descriptor))
            .collect::<BTreeMap<_, _>>();
        self.context()
            .dirty_documents()
            .changes_since(None)?
            .snapshots()
            .iter()
            .filter(|snapshot| snapshot.is_dirty())
            .map(|snapshot| {
                let descriptor = descriptors.get(&snapshot.document()).ok_or_else(|| {
                    EditorError::Registry(format!(
                        "dirty document {:?} has no document toolkit",
                        snapshot.document()
                    ))
                })?;
                Ok(DirtyDocumentToolkitView {
                    document_id: snapshot.document(),
                    dirty_generation: snapshot.generation(),
                    instance_id: ViewInstanceId::new(descriptor.instance_id().as_str()),
                    title: descriptor.title().to_string(),
                })
            })
            .collect()
    }

    pub(crate) fn mark_document_external_effect(
        &self,
        instance_id: &ViewInstanceId,
        effect: DirtyExternalEffectId,
    ) -> Result<DirtyExternalEffectRevision, EditorError> {
        self.host.mark_document_external_effect(instance_id, effect)
    }
}
