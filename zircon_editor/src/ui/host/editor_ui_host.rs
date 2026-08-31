use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::asset::{DirtyExternalEffectId, DirtyExternalEffectRevision, DirtyRegistry};
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::EditorTransactionEngine;
use crate::core::extension::{
    DocumentAutosavePayload, DocumentCloseLease, DocumentSaveReport, DocumentToolkit,
    DocumentToolkitDescriptor, DocumentToolkitRegistry, DocumentToolkitSnapshot, SaveCtx,
    SaveReason, ToolkitInstanceId, ToolkitLayout, ToolkitSaveFailure,
};
use crate::core::jobs::EditorJobSystem;
use crate::core::logging::EditorLogService;
use crate::core::settings::SettingsAuthority;
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
use super::editor_subsystems::EditorSubsystemReport;
use super::host_capability_bridge::{register_vm_host_capabilities, EditorHostVmBridgeReport};
use super::minimal_host_contract::{editor_host_minimal_contract, EditorHostMinimalReport};
use super::runtime_services::EditorHostRuntimeServices;
use super::window_host_manager::WindowHostManager;

pub(super) struct EditorUiHost {
    pub(super) runtime_services: EditorHostRuntimeServices,
    pub(super) settings: Arc<SettingsAuthority>,
    pub(super) logs: Arc<EditorLogService>,
    pub(super) jobs: EditorJobSystem,
    pub(super) transactions: Arc<EditorTransactionEngine>,
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
    pub(super) document_toolkits: DocumentToolkitRegistry<EditorUiHost>,
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

    pub(super) fn new(
        runtime_services: EditorHostRuntimeServices,
        jobs: EditorJobSystem,
        transactions: Arc<EditorTransactionEngine>,
        logs: Arc<EditorLogService>,
        dirty_documents: DirtyRegistry,
        settings: Arc<SettingsAuthority>,
    ) -> Result<Self, EditorError> {
        let minimal_report = editor_host_minimal_contract().self_check();
        let subsystem_report = runtime_services.subsystem_report()?;
        let capability_snapshot =
            EditorCapabilitySnapshot::from_reports(&minimal_report, &subsystem_report);
        let runtime_sandbox_enabled = runtime_services.runtime_sandbox_enabled()?;
        let vm_bridge_report =
            register_vm_host_capabilities(&runtime_services, runtime_sandbox_enabled);

        Ok(Self {
            runtime_services,
            settings,
            logs,
            jobs: jobs.clone(),
            transactions,
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
        })
    }

    pub(super) fn bootstrap(
        runtime_services: EditorHostRuntimeServices,
        jobs: EditorJobSystem,
        transactions: Arc<EditorTransactionEngine>,
        logs: Arc<EditorLogService>,
        dirty_documents: DirtyRegistry,
        settings: Arc<SettingsAuthority>,
    ) -> Result<Self, EditorError> {
        let host = Self::new(
            runtime_services,
            jobs,
            transactions,
            logs,
            dirty_documents,
            settings,
        )?;
        host.register_builtin_views()?;
        host.bootstrap_default_layout()?;
        Ok(host)
    }

    pub(super) fn refresh_capabilities(&self) -> Result<EditorCapabilitySnapshot, EditorError> {
        let subsystem_report = self.runtime_services.subsystem_report()?;
        self.apply_capability_report(subsystem_report)
    }

    pub(super) fn register_document_toolkit(
        &self,
        instance_id: &ViewInstanceId,
        layout_id: &'static str,
        tab_id: &'static str,
        validate_references: HostDocumentReferenceValidationHook,
        save: HostDocumentSaveHook,
        autosave_source_path: HostDocumentAutosaveSourcePathHook,
        capture_autosave: HostDocumentAutosaveHook,
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
        let descriptor =
            DocumentToolkitDescriptor::new(document, toolkit_instance.clone(), title, layout);
        self.document_toolkits
            .register(Arc::new(HostDocumentToolkit {
                descriptor,
                instance: instance_id.clone(),
                validate_references,
                save,
                autosave_source_path,
                capture_autosave,
            }))?;
        if let Err(error) = self.dirty_documents.register_document(document) {
            let _ = self.document_toolkits.unregister(&toolkit_instance);
            return Err(error.into());
        }
        Ok(document)
    }

    pub(super) fn unregister_document_toolkit(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        let toolkit_instance = ToolkitInstanceId::parse(instance_id.0.clone())?;
        let Some(descriptor) = self.document_toolkits.unregister(&toolkit_instance)? else {
            return Ok(());
        };
        self.dirty_documents
            .unregister_document(descriptor.document_id())?;
        Ok(())
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
        let source_path = self
            .document_toolkits
            .autosave_source_path(document, self)?;
        let save_mutex = super::editor_document_autosave::document_save_mutex_group(&source_path)?;
        let job = self
            .runtime_services
            .foreground_document_save_job(instance_id.clone(), reason)?;
        let ticket = self
            .jobs
            .submit(
                super::editor_document_autosave::ForegroundDocumentSaveJob::spec(
                    document, save_mutex,
                ),
                job,
            )
            .map_err(|error| EditorError::Project(error.to_string()))?;
        ticket
            .wait()
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub(super) fn save_document_toolkit_canonical(
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
        let report = self.write_document_toolkit(document, reason)?;
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

    pub(super) fn write_document_toolkit(
        &self,
        document: crate::core::editor_message::DocumentId,
        reason: SaveReason,
    ) -> Result<DocumentSaveReport, EditorError> {
        Ok(self.document_toolkits.save(document, self, reason)?)
    }

    pub(super) fn validate_document_toolkit_references(
        &self,
        document: crate::core::editor_message::DocumentId,
    ) -> Result<(), EditorError> {
        Ok(self.document_toolkits.validate_references(document, self)?)
    }

    pub(super) fn capture_document_autosave(
        &self,
        document: crate::core::editor_message::DocumentId,
        expected_dirty_generation: u64,
    ) -> Result<DocumentAutosavePayload, EditorError> {
        let dirty = self.dirty_documents.snapshot(document)?;
        if !dirty.is_dirty() || dirty.generation() != expected_dirty_generation {
            return Err(EditorError::Project(format!(
                "autosave intent for document {document:?} generation {expected_dirty_generation} was superseded"
            )));
        }
        Ok(self.document_toolkits.capture_autosave(document, self)?)
    }

    pub(super) fn document_autosave_source_path(
        &self,
        document: crate::core::editor_message::DocumentId,
    ) -> Result<std::path::PathBuf, EditorError> {
        Ok(self
            .document_toolkits
            .autosave_source_path(document, self)?)
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
        let documents = descriptors
            .iter()
            .map(DocumentToolkitDescriptor::document_id)
            .collect::<Vec<_>>();
        for descriptor in descriptors {
            self.dirty_documents
                .unregister_document(descriptor.document_id())?;
        }
        self.detach_animation_authoring_documents(&documents)?;
        Ok(())
    }

    pub(super) fn commit_document_close(
        &self,
        close: DocumentCloseLease<'_, EditorUiHost>,
    ) -> Result<DocumentToolkitDescriptor, EditorError> {
        let descriptor = close.commit()?;
        self.dirty_documents
            .unregister_document(descriptor.document_id())?;
        self.detach_animation_authoring_documents(&[descriptor.document_id()])?;
        Ok(descriptor)
    }

    fn detach_animation_authoring_documents(
        &self,
        documents: &[crate::core::editor_message::DocumentId],
    ) -> Result<(), EditorError> {
        if documents.is_empty() {
            return Ok(());
        }
        self.transactions
            .with_context_mut::<CoreEditContext, _>(|context| {
                for document in documents {
                    context.animation_documents_mut().detach(*document);
                }
            })
            .map_err(|error| EditorError::UiAsset(error.to_string()))?
            .ok_or_else(|| {
                EditorError::UiAsset("animation transaction context type mismatch".to_string())
            })?;
        Ok(())
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

    pub(super) fn sync_document_dirty_projection_for_document(
        &self,
        document: crate::core::editor_message::DocumentId,
    ) -> Result<(), EditorError> {
        let toolkit_snapshot = self.document_toolkits.snapshot();
        let Some(descriptor) = toolkit_snapshot
            .descriptors()
            .iter()
            .find(|descriptor| descriptor.document_id() == document)
        else {
            return Ok(());
        };
        let instance_id = ViewInstanceId::new(descriptor.instance_id().as_str());
        self.sync_document_dirty_projection(&instance_id)
    }

    pub(super) fn document_toolkit_snapshot(&self) -> DocumentToolkitSnapshot {
        self.document_toolkits.snapshot()
    }
}

type HostDocumentSaveHook =
    fn(&EditorUiHost, &ViewInstanceId, &mut SaveCtx) -> Result<(), ToolkitSaveFailure>;
type HostDocumentReferenceValidationHook =
    fn(&EditorUiHost, &ViewInstanceId) -> Result<(), ToolkitSaveFailure>;
type HostDocumentAutosaveSourcePathHook =
    fn(&EditorUiHost, &ViewInstanceId) -> Result<std::path::PathBuf, ToolkitSaveFailure>;
type HostDocumentAutosaveHook =
    fn(&EditorUiHost, &ViewInstanceId) -> Result<DocumentAutosavePayload, ToolkitSaveFailure>;

struct HostDocumentToolkit {
    descriptor: DocumentToolkitDescriptor,
    instance: ViewInstanceId,
    validate_references: HostDocumentReferenceValidationHook,
    save: HostDocumentSaveHook,
    autosave_source_path: HostDocumentAutosaveSourcePathHook,
    capture_autosave: HostDocumentAutosaveHook,
}

impl DocumentToolkit<EditorUiHost> for HostDocumentToolkit {
    fn descriptor(&self) -> &DocumentToolkitDescriptor {
        &self.descriptor
    }

    fn validate_references(&self, host: &EditorUiHost) -> Result<(), ToolkitSaveFailure> {
        (self.validate_references)(host, &self.instance)
    }

    fn save(&self, host: &EditorUiHost, context: &mut SaveCtx) -> Result<(), ToolkitSaveFailure> {
        (self.save)(host, &self.instance, context)
    }

    fn autosave_source_path(
        &self,
        host: &EditorUiHost,
    ) -> Result<std::path::PathBuf, ToolkitSaveFailure> {
        (self.autosave_source_path)(host, &self.instance)
    }

    fn capture_autosave(
        &self,
        host: &EditorUiHost,
    ) -> Result<DocumentAutosavePayload, ToolkitSaveFailure> {
        (self.capture_autosave)(host, &self.instance)
    }
}
