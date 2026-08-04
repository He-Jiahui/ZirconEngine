use std::path::{Path, PathBuf};
use std::sync::Arc;

use zircon_runtime::plugin::RuntimePluginCatalog;

use crate::core::plugin::{EditorPluginCatalogSnapshot, EditorPluginPanelSource};

pub(in crate::ui::host) use self::status::ProjectPluginStatusSnapshot;
use super::editor_manager::EditorManager;

mod enablement;
mod export_build;
mod manifest_completion;
mod native_registration;
mod package_projection;
mod reports;
mod status;

pub use self::export_build::{
    apply_export_wizard_panel_template_state, execute_export_wizard_pipeline,
    execute_export_wizard_stage, export_wizard_compile_host_executable_path,
    export_wizard_compile_host_target_dir, export_wizard_panel_action_call,
    export_wizard_panel_action_for_control, export_wizard_panel_binding_entries,
    export_wizard_panel_bindings, export_wizard_panel_retained_projection,
    export_wizard_panel_template_state, export_wizard_pipeline_plan, project_export_wizard_panel,
    register_export_wizard_panel_bindings, register_export_wizard_panel_template,
    run_export_wizard_job, EditorExportBuildError, EditorExportBuildProgress,
    EditorExportBuildReport, EditorExportCargoInvocation, ExportStageProgressKind,
    ExportWizardCancelSignal, ExportWizardCommandExecution, ExportWizardCommandRunner,
    ExportWizardControlState, ExportWizardJobCompletion, ExportWizardJobController,
    ExportWizardJobEvent, ExportWizardJobEventKind, ExportWizardJobSnapshot, ExportWizardJobState,
    ExportWizardJobStatus, ExportWizardNeverCancel, ExportWizardPanelAction,
    ExportWizardPanelBinding, ExportWizardPanelControlBindingState, ExportWizardPanelEntrySeverity,
    ExportWizardPanelRequest, ExportWizardPanelSession, ExportWizardPanelSessionError,
    ExportWizardPanelSlotEntry, ExportWizardPanelSlotKind, ExportWizardPanelSlotState,
    ExportWizardPanelTemplateState, ExportWizardPanelUpdate, ExportWizardPanelViewModel,
    ExportWizardPipelineExecution, ExportWizardPipelineOptions, ExportWizardPipelinePlan,
    ExportWizardPipelineStageCommand, ExportWizardProgressState, ExportWizardStageArtifactPath,
    ExportWizardStageExecution, ExportWizardStageMissingInputs, ExportWizardStagePlannedArtifacts,
    ExportWizardStageProgressSnapshot, ExportWizardStageViewRow, ExportWizardStreamEvent,
    ProcessCommandRunner, DESKTOP_EXPORT_ARTIFACT_PATHS_SLOT, DESKTOP_EXPORT_CANCEL_BINDING_ID,
    DESKTOP_EXPORT_CANCEL_BUTTON, DESKTOP_EXPORT_GENERATE_PLAN_BINDING_ID,
    DESKTOP_EXPORT_GENERATE_PLAN_BUTTON, DESKTOP_EXPORT_MISSING_INPUTS_SLOT,
    DESKTOP_EXPORT_REPORT_BODY_SLOT, DESKTOP_EXPORT_STAGE_ROWS_SLOT,
    DESKTOP_EXPORT_START_BINDING_ID, DESKTOP_EXPORT_START_BUTTON,
    DESKTOP_EXPORT_TERMINAL_OUTPUT_SLOT, EXPORT_WIZARD_BINDING_SYMBOL,
    EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID, EXPORT_WIZARD_VIEW_ID,
};
pub use self::reports::{
    EditorPluginEnableReport, EditorPluginFeatureDependencyStatus,
    EditorPluginFeatureSelectionUpdateReport, EditorPluginFeatureStatus,
    EditorPluginSelectionUpdateReport, EditorPluginStatus, EditorPluginStatusReport,
};

impl EditorManager {
    pub fn plugin_directory(&self, project_root: impl AsRef<Path>) -> PathBuf {
        project_root.as_ref().join("zircon_plugins")
    }

    pub fn plugin_catalog(&self) -> Arc<EditorPluginCatalogSnapshot> {
        self.editor_plugin_catalog()
    }

    pub fn runtime_plugin_catalog(&self) -> &'static RuntimePluginCatalog {
        RuntimePluginCatalog::builtin()
    }

    pub fn editor_plugin_catalog(&self) -> Arc<EditorPluginCatalogSnapshot> {
        self.plugin_manager().catalog_snapshot()
    }

    /// Returns one immutable manager generation for plugin-panel consumers.
    pub fn plugin_panel_source(&self) -> EditorPluginPanelSource {
        EditorPluginPanelSource::from_manager(self.plugin_manager())
    }
}
