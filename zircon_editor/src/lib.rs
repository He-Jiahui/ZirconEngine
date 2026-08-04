//! Editor host UI built on Retained, with viewport frames coming from core graphics.

pub mod core;
pub mod scene;
pub mod ui;

pub use core::commands::{
    CommandEvalCtx, CommandEvalSnapshotHandle, DocumentKind, DocumentKindError,
    EditorCommandAction, EditorCommandCategory, EditorCommandDescriptor,
    EditorCommandDispatchError, EditorCommandPaletteEntry, EditorCommandRegistry,
    EditorCommandRegistryError, EditorKeyBinding, EditorKeyChord, EditorKeyChordParseError,
    EditorKeymap, EditorKeymapConflict, EditorKeymapError, PlayModePredicate, WhenClause,
};
pub use core::editing::intent::EditorIntent;
pub use core::gateway::{
    DetachedEditorRuntimeGateway, EditorRuntimeFrame, EditorRuntimeGateway,
    EditorRuntimeGatewayHandle, GatewayError, InProcessGateway, PluginActivationState,
    PluginSummaryEntry, RuntimeCapabilities, SessionGateway, SessionProfileKind,
    SharedEditorRuntimeGateway,
};
pub use core::gui_startup_request::EditorGuiStartupRequest;
pub use core::plugin::{
    EditorExtensionCatalogReport, EditorPlugin, EditorPluginDescriptor,
    EditorPluginRegistrationReport,
};
pub use ui::host::module::{
    EDITOR_ASSET_MANAGER_NAME, EDITOR_COMMAND_REGISTRY_NAME, EDITOR_HOST_DRIVER_NAME,
    EDITOR_KEYMAP_NAME, EDITOR_MANAGER_NAME, EDITOR_MODULE_NAME, EditorHostDriver, EditorModule,
    module_descriptor,
};
pub use ui::host::{
    DESKTOP_EXPORT_ARTIFACT_PATHS_SLOT, DESKTOP_EXPORT_CANCEL_BINDING_ID,
    DESKTOP_EXPORT_CANCEL_BUTTON, DESKTOP_EXPORT_GENERATE_PLAN_BINDING_ID,
    DESKTOP_EXPORT_GENERATE_PLAN_BUTTON, DESKTOP_EXPORT_MISSING_INPUTS_SLOT,
    DESKTOP_EXPORT_REPORT_BODY_SLOT, DESKTOP_EXPORT_STAGE_ROWS_SLOT,
    DESKTOP_EXPORT_START_BINDING_ID, DESKTOP_EXPORT_START_BUTTON,
    DESKTOP_EXPORT_TERMINAL_OUTPUT_SLOT, EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
    EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY, EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
    EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING, EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
    EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING, EXPORT_WIZARD_BINDING_SYMBOL,
    EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID, EXPORT_WIZARD_VIEW_ID, EditorCapabilitySnapshot,
    EditorExportBuildReport, EditorExportCargoInvocation, EditorHostMinimalContract,
    EditorHostMinimalReport, EditorHostVmBridgeReport, EditorKeymapService,
    EditorPluginEnableReport,
    EditorPluginFeatureDependencyStatus, EditorPluginFeatureSelectionUpdateReport,
    EditorPluginFeatureStatus, EditorPluginSelectionUpdateReport, EditorPluginStatus,
    EditorPluginStatusReport, EditorSubsystemReport, EditorVmExtensionLoadReport,
    ExportStageProgressKind, ExportWizardCancelSignal, ExportWizardCommandExecution,
    ExportWizardCommandRunner, ExportWizardControlState, ExportWizardJobCompletion,
    ExportWizardJobController, ExportWizardJobEvent, ExportWizardJobEventKind,
    ExportWizardJobSnapshot, ExportWizardJobState, ExportWizardJobStatus, ExportWizardNeverCancel,
    ExportWizardPanelAction, ExportWizardPanelBinding, ExportWizardPanelControlBindingState,
    ExportWizardPanelEntrySeverity, ExportWizardPanelRequest, ExportWizardPanelSession,
    ExportWizardPanelSessionError, ExportWizardPanelSlotEntry, ExportWizardPanelSlotKind,
    ExportWizardPanelSlotState, ExportWizardPanelTemplateState, ExportWizardPanelUpdate,
    ExportWizardPanelViewModel, ExportWizardPipelineExecution, ExportWizardPipelineOptions,
    ExportWizardPipelinePlan, ExportWizardPipelineStageCommand, ExportWizardProgressState,
    ExportWizardStageArtifactPath, ExportWizardStageExecution, ExportWizardStageMissingInputs,
    ExportWizardStagePlannedArtifacts, ExportWizardStageProgressSnapshot, ExportWizardStageViewRow,
    ExportWizardStreamEvent, ProcessCommandRunner, UiAssetWorkspaceWatchDiagnostics,
    UiAssetWorkspaceWatchPollReport, apply_export_wizard_panel_template_state,
    editor_host_minimal_contract, execute_export_wizard_pipeline, execute_export_wizard_stage,
    export_wizard_panel_action_call, export_wizard_panel_action_for_control,
    export_wizard_panel_binding_entries, export_wizard_panel_bindings,
    export_wizard_panel_retained_projection, export_wizard_panel_template_state,
    export_wizard_pipeline_plan, project_export_wizard_panel,
    register_export_wizard_panel_bindings, register_export_wizard_panel_template,
    run_export_wizard_job,
};
pub use ui::retained_host::{
    EditorHostRunConfig, run_editor, run_editor_with_config, run_editor_with_startup_request,
};

#[cfg(test)]
mod tests;
