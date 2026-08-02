use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

use zircon_runtime::core::{CoreError, CoreHandle, CoreWeak};

use crate::core::asset::{DirtyExternalEffectId, DirtyExternalEffectRevision, DirtyRegistry};
use crate::core::extension::{
    DocumentCloseLease, DocumentSaveReport, DocumentToolkit, DocumentToolkitDescriptor,
    DocumentToolkitRegistry, DocumentToolkitSnapshot, SaveCtx, SaveReason, ToolkitInstanceId,
    ToolkitLayout, ToolkitSaveFailure,
};
use crate::core::jobs::EditorJobSystem;
use crate::ui::workbench::layout::LayoutManager;
use crate::ui::workbench::view::{ViewInstanceId, ViewRegistry};
use crate::ui::workbench::window_registry::EditorWindowRegistry;

use super::animation_editor_sessions::AnimationEditorWorkspaceEntry;
use super::asset_editor_sessions::{
    UiAssetDependencyGeneration, UiAssetWorkspaceEntry, UiAssetWorkspaceRefreshPipeline,
    UiAssetWorkspaceWatcher,
};
use super::editor_capabilities::EditorCapabilitySnapshot;
use super::editor_error::EditorError;
use super::editor_session_state::EditorSessionState;
use super::editor_subsystems::{
    editor_runtime_sandbox_enabled, editor_subsystem_report_from_core, EditorSubsystemReport,
};
use super::host_capability_bridge::{register_vm_host_capabilities, EditorHostVmBridgeReport};
use super::minimal_host_contract::{editor_host_minimal_contract, EditorHostMinimalReport};
use super::window_host_manager::WindowHostManager;

pub(super) struct EditorUiHost {
    // EditorManager is registry-owned; the host must upgrade only at operation boundaries.
    core: CoreWeak,
    pub(super) view_registry: Mutex<ViewRegistry>,
    pub(super) layout_manager: LayoutManager,
    pub(super) window_host_manager: Mutex<WindowHostManager>,
    pub(super) window_registry: Mutex<EditorWindowRegistry>,
    pub(super) session: Mutex<EditorSessionState>,
    pub(super) animation_editor_sessions:
        Mutex<BTreeMap<ViewInstanceId, AnimationEditorWorkspaceEntry>>,
    pub(super) ui_asset_sessions: Mutex<BTreeMap<ViewInstanceId, UiAssetWorkspaceEntry>>,
    pub(super) ui_asset_dependency_generation: Mutex<UiAssetDependencyGeneration>,
    pub(super) ui_asset_refresh_pipeline: Mutex<UiAssetWorkspaceRefreshPipeline>,
    pub(super) ui_asset_workspace_watcher: Mutex<Option<UiAssetWorkspaceWatcher>>,
    document_toolkits: DocumentToolkitRegistry<EditorUiHost>,
    dirty_documents: DirtyRegistry,
    pub(super) minimal_report: EditorHostMinimalReport,
    pub(super) subsystem_report: Mutex<EditorSubsystemReport>,
    pub(super) capability_snapshot: Mutex<EditorCapabilitySnapshot>,
    pub(super) vm_bridge_report: EditorHostVmBridgeReport,
}

impl EditorUiHost {
    fn recover_lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(super) fn lock_view_registry(&self) -> MutexGuard<'_, ViewRegistry> {
        Self::recover_lock(&self.view_registry)
    }

    pub(super) fn lock_window_host_manager(&self) -> MutexGuard<'_, WindowHostManager> {
        Self::recover_lock(&self.window_host_manager)
    }

    pub(super) fn lock_window_registry(&self) -> MutexGuard<'_, EditorWindowRegistry> {
        Self::recover_lock(&self.window_registry)
    }

    pub(super) fn lock_session(&self) -> MutexGuard<'_, EditorSessionState> {
        Self::recover_lock(&self.session)
    }

    pub(super) fn lock_animation_editor_sessions(
        &self,
    ) -> MutexGuard<'_, BTreeMap<ViewInstanceId, AnimationEditorWorkspaceEntry>> {
        Self::recover_lock(&self.animation_editor_sessions)
    }

    pub(super) fn lock_ui_asset_sessions(
        &self,
    ) -> MutexGuard<'_, BTreeMap<ViewInstanceId, UiAssetWorkspaceEntry>> {
        Self::recover_lock(&self.ui_asset_sessions)
    }

    pub(super) fn lock_ui_asset_dependency_generation(
        &self,
    ) -> MutexGuard<'_, UiAssetDependencyGeneration> {
        Self::recover_lock(&self.ui_asset_dependency_generation)
    }

    pub(super) fn lock_ui_asset_refresh_pipeline(
        &self,
    ) -> MutexGuard<'_, UiAssetWorkspaceRefreshPipeline> {
        Self::recover_lock(&self.ui_asset_refresh_pipeline)
    }

    pub(super) fn lock_ui_asset_workspace_watcher(
        &self,
    ) -> MutexGuard<'_, Option<UiAssetWorkspaceWatcher>> {
        Self::recover_lock(&self.ui_asset_workspace_watcher)
    }

    pub(super) fn lock_subsystem_report(&self) -> MutexGuard<'_, EditorSubsystemReport> {
        Self::recover_lock(&self.subsystem_report)
    }

    pub(super) fn lock_capability_snapshot(&self) -> MutexGuard<'_, EditorCapabilitySnapshot> {
        Self::recover_lock(&self.capability_snapshot)
    }

    pub(super) fn runtime_core(&self) -> Result<CoreHandle, EditorError> {
        self.core
            .upgrade()
            .ok_or_else(|| CoreError::RuntimeUnavailable.into())
    }

    pub(super) fn new(
        core: &CoreHandle,
        jobs: EditorJobSystem,
        dirty_documents: DirtyRegistry,
    ) -> Self {
        let minimal_report = editor_host_minimal_contract().self_check();
        let subsystem_report = editor_subsystem_report_from_core(core);
        let capability_snapshot =
            EditorCapabilitySnapshot::from_reports(&minimal_report, &subsystem_report);
        let runtime_sandbox_enabled = editor_runtime_sandbox_enabled(core);
        let vm_bridge_report = register_vm_host_capabilities(core, runtime_sandbox_enabled);

        Self {
            core: core.downgrade(),
            view_registry: Mutex::new(ViewRegistry::default()),
            layout_manager: LayoutManager,
            window_host_manager: Mutex::new(WindowHostManager::default()),
            window_registry: Mutex::new(EditorWindowRegistry::default()),
            session: Mutex::new(EditorSessionState::default()),
            animation_editor_sessions: Mutex::new(BTreeMap::new()),
            ui_asset_sessions: Mutex::new(BTreeMap::new()),
            ui_asset_dependency_generation: Mutex::new(UiAssetDependencyGeneration::default()),
            ui_asset_refresh_pipeline: Mutex::new(UiAssetWorkspaceRefreshPipeline::new(jobs)),
            ui_asset_workspace_watcher: Mutex::new(None),
            document_toolkits: DocumentToolkitRegistry::default(),
            dirty_documents,
            minimal_report,
            subsystem_report: Mutex::new(subsystem_report),
            capability_snapshot: Mutex::new(capability_snapshot),
            vm_bridge_report,
        }
    }

    pub(super) fn bootstrap(
        core: &CoreHandle,
        jobs: EditorJobSystem,
        dirty_documents: DirtyRegistry,
    ) -> Result<Self, EditorError> {
        let host = Self::new(core, jobs, dirty_documents);
        host.register_builtin_views()?;
        host.bootstrap_default_layout()?;
        Ok(host)
    }

    pub(super) fn refresh_capabilities(&self) -> Result<EditorCapabilitySnapshot, EditorError> {
        let core = self.runtime_core()?;
        self.refresh_capabilities_from_core(&core)
    }

    pub(super) fn refresh_capabilities_from_core(
        &self,
        core: &CoreHandle,
    ) -> Result<EditorCapabilitySnapshot, EditorError> {
        let subsystem_report = editor_subsystem_report_from_core(core);
        let snapshot =
            EditorCapabilitySnapshot::from_reports(&self.minimal_report, &subsystem_report);
        *self.lock_subsystem_report() = subsystem_report;
        *self.lock_capability_snapshot() = snapshot.clone();
        self.register_builtin_views()?;
        Ok(snapshot)
    }

    pub(super) fn register_document_toolkit(
        &self,
        instance_id: &ViewInstanceId,
        layout_id: &'static str,
        tab_id: &'static str,
        save: HostDocumentSaveHook,
    ) -> Result<crate::core::editor_message::DocumentId, EditorError> {
        let toolkit_instance = ToolkitInstanceId::parse(instance_id.0.clone())?;
        if let Some(document) = self
            .document_toolkits
            .document_for_instance(&toolkit_instance)
        {
            self.dirty_documents.register_document(document)?;
            return Ok(document);
        }
        let title = self
            .lock_session()
            .open_view_instances
            .get(instance_id)
            .map(|instance| instance.title.clone())
            .ok_or_else(|| {
                EditorError::Registry(format!("missing view instance {}", instance_id.0))
            })?;
        let layout = ToolkitLayout::single_tab(layout_id, tab_id)?;
        let document = self.document_toolkits.allocate_document_id()?;
        let descriptor = DocumentToolkitDescriptor::new(document, toolkit_instance, title, layout);
        self.document_toolkits
            .register(Arc::new(HostDocumentToolkit {
                descriptor,
                instance: instance_id.clone(),
                save,
            }))?;
        if let Err(error) = self.dirty_documents.register_document(document) {
            let _ = self.document_toolkits.unregister(&toolkit_instance);
            return Err(error.into());
        }
        Ok(document)
    }

    pub(super) fn save_document_toolkit(
        &self,
        instance_id: &ViewInstanceId,
        reason: SaveReason,
    ) -> Result<DocumentSaveReport, EditorError> {
        let toolkit_instance = ToolkitInstanceId::parse(instance_id.0.clone())?;
        let document = self
            .document_toolkits
            .document_for_instance(&toolkit_instance)
            .ok_or_else(|| EditorError::DocumentToolkitNotRegistered {
                instance: instance_id.0.clone(),
            })?;
        let dirty_snapshot = self.dirty_documents.snapshot(document)?;
        let save_token = self.dirty_documents.capture_save_token(document)?;
        let report = self.document_toolkits.save(document, self, reason)?;
        self.dirty_documents
            .mark_saved_if_unchanged(document, save_token)?;
        if !self
            .dirty_documents
            .clear_saved_external_effects(&dirty_snapshot)?
        {
            return Err(
                crate::core::asset::DirtyRegistryError::DocumentChangedDuringSave {
                    document,
                    expected_generation: dirty_snapshot.generation(),
                }
                .into(),
            );
        }
        self.sync_document_dirty_projection(instance_id)?;
        Ok(report)
    }

    pub(super) fn begin_document_close(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<Option<DocumentCloseLease<'_, EditorUiHost>>, EditorError> {
        let Ok(toolkit_instance) = ToolkitInstanceId::parse(instance_id.0.clone()) else {
            return Ok(None);
        };
        Ok(self.document_toolkits.begin_close(&toolkit_instance)?)
    }

    pub(super) fn clear_document_toolkits(&self) -> Result<(), EditorError> {
        let descriptors = self.document_toolkits.clear()?;
        for descriptor in descriptors {
            self.dirty_documents
                .unregister_document(descriptor.document_id())?;
        }
        Ok(())
    }

    pub(super) fn commit_document_close(
        &self,
        close: DocumentCloseLease<'_, EditorUiHost>,
    ) -> Result<DocumentToolkitDescriptor, EditorError> {
        let descriptor = close.commit()?;
        self.dirty_documents
            .unregister_document(descriptor.document_id())?;
        Ok(descriptor)
    }

    pub(super) fn mark_document_external_effect(
        &self,
        instance_id: &ViewInstanceId,
        effect: DirtyExternalEffectId,
    ) -> Result<DirtyExternalEffectRevision, EditorError> {
        let toolkit_instance = ToolkitInstanceId::parse(instance_id.0.clone())?;
        let document = self
            .document_toolkits
            .document_for_instance(&toolkit_instance)
            .ok_or_else(|| EditorError::DocumentToolkitNotRegistered {
                instance: instance_id.0.clone(),
            })?;
        let revision = self
            .dirty_documents
            .mark_external_effect(document, effect)?;
        self.sync_document_dirty_projection(instance_id)?;
        Ok(revision)
    }

    pub(super) fn ensure_document_external_effect(
        &self,
        instance_id: &ViewInstanceId,
        effect: DirtyExternalEffectId,
    ) -> Result<DirtyExternalEffectRevision, EditorError> {
        let toolkit_instance = ToolkitInstanceId::parse(instance_id.0.clone())?;
        let document = self
            .document_toolkits
            .document_for_instance(&toolkit_instance)
            .ok_or_else(|| EditorError::DocumentToolkitNotRegistered {
                instance: instance_id.0.clone(),
            })?;
        let snapshot = self.dirty_documents.snapshot(document)?;
        if let Some(revision) = snapshot.external_revision(&effect) {
            return Ok(revision);
        }
        Ok(self
            .dirty_documents
            .mark_external_effect(document, effect)?)
    }

    pub(super) fn document_dirty(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        let toolkit_instance = ToolkitInstanceId::parse(instance_id.0.clone())?;
        let document = self
            .document_toolkits
            .document_for_instance(&toolkit_instance)
            .ok_or_else(|| EditorError::DocumentToolkitNotRegistered {
                instance: instance_id.0.clone(),
            })?;
        Ok(self.dirty_documents.snapshot(document)?.is_dirty())
    }

    pub(super) fn document_dirty_if_registered(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<Option<bool>, EditorError> {
        let toolkit_instance = ToolkitInstanceId::parse(instance_id.0.clone())?;
        let Some(document) = self
            .document_toolkits
            .document_for_instance(&toolkit_instance)
        else {
            return Ok(None);
        };
        Ok(Some(self.dirty_documents.snapshot(document)?.is_dirty()))
    }

    fn sync_document_dirty_projection(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        let dirty = self.document_dirty(instance_id)?;
        let mut session = self.lock_session();
        let instance = session
            .open_view_instances
            .get_mut(instance_id)
            .ok_or_else(|| {
                EditorError::Registry(format!("missing view instance {}", instance_id.0))
            })?;
        instance.dirty = dirty;
        Ok(())
    }

    pub(super) fn document_toolkit_snapshot(&self) -> DocumentToolkitSnapshot {
        self.document_toolkits.snapshot()
    }
}

type HostDocumentSaveHook =
    fn(&EditorUiHost, &ViewInstanceId, &mut SaveCtx) -> Result<(), ToolkitSaveFailure>;

struct HostDocumentToolkit {
    descriptor: DocumentToolkitDescriptor,
    instance: ViewInstanceId,
    save: HostDocumentSaveHook,
}

impl DocumentToolkit<EditorUiHost> for HostDocumentToolkit {
    fn descriptor(&self) -> &DocumentToolkitDescriptor {
        &self.descriptor
    }

    fn save(&self, host: &EditorUiHost, context: &mut SaveCtx) -> Result<(), ToolkitSaveFailure> {
        (self.save)(host, &self.instance, context)
    }
}
