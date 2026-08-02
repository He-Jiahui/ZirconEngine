//! Editor UI host ownership and orchestration surfaces.

pub(crate) mod animation_editor_sessions;
pub(crate) mod asset_editor_sessions;
mod builtin_layout;
mod builtin_views;
pub(crate) mod command_eval_projection;
pub(crate) mod editor_asset_manager;
mod editor_capabilities;
mod editor_error;
mod editor_event_control_requests;
mod editor_event_dispatch;
mod editor_event_execution;
mod editor_event_runtime_access;
mod editor_event_runtime_reflection;
mod editor_extension_registration;
mod editor_extension_views;
mod editor_host_event_controller;
mod editor_host_visible_spatial_query;
mod editor_host_startup;
mod editor_manager;
mod editor_manager_animation_editor;
mod editor_manager_asset_editor;
mod editor_manager_asset_workspace;
mod editor_manager_layout;
mod editor_manager_minimal_host;
mod editor_manager_plugins_export;
mod editor_manager_project;
mod editor_manager_runtime_diagnostics;
mod editor_manager_startup;
mod editor_manager_workspace;
mod editor_operation_dispatch;
mod editor_scene_document_submission;
mod editor_scene_mode_lifecycle;
mod editor_session_state;
mod editor_subsystems;
mod editor_ui_host;
mod export_cargo_process;
mod export_process_support;
mod host_capability_bridge;
mod layout_commands;
mod layout_hosts;
mod layout_persistence;
pub(crate) mod minimal_host_contract;
pub(crate) mod module;
mod native_dynamic_export_preparation;
pub(crate) mod play_pending_decision;
pub(crate) mod project_access;
pub(crate) mod resource_access;
mod scene_inspection_publication;
mod startup;
mod ui_asset_promotion;
mod view_registry;
mod window_host_manager;
mod workspace_state;

pub use asset_editor_sessions::{
    UiAssetWorkspaceWatchDiagnostics, UiAssetWorkspaceWatchPollReport,
};
pub(crate) use builtin_layout::builtin_hybrid_layout;
pub use editor_capabilities::EditorCapabilitySnapshot;
pub use editor_error::EditorError;
pub use editor_host_event_controller::EditorHostEventController;
pub(crate) use editor_host_startup::resolve_editor_startup_session;
pub use editor_host_startup::EditorHostStartupSession;
pub use editor_manager::EditorManager;
pub(crate) use editor_manager_layout::DirtyDocumentToolkitView;
pub use editor_manager_plugins_export::{
    apply_export_wizard_panel_template_state, execute_export_wizard_pipeline,
    execute_export_wizard_stage, export_wizard_compile_host_executable_path,
    export_wizard_compile_host_target_dir, export_wizard_panel_action_call,
    export_wizard_panel_action_for_control, export_wizard_panel_binding_entries,
    export_wizard_panel_bindings, export_wizard_panel_retained_projection,
    export_wizard_panel_template_state, export_wizard_pipeline_plan, project_export_wizard_panel,
    register_export_wizard_panel_bindings, register_export_wizard_panel_template,
    run_export_wizard_job, EditorExportBuildError, EditorExportBuildProgress,
    EditorExportBuildReport, EditorExportCargoInvocation, EditorPluginEnableReport,
    EditorPluginFeatureDependencyStatus, EditorPluginFeatureSelectionUpdateReport,
    EditorPluginFeatureStatus, EditorPluginSelectionUpdateReport, EditorPluginStatus,
    EditorPluginStatusReport, ExportStageProgressKind, ExportWizardCancelSignal,
    ExportWizardCommandExecution, ExportWizardCommandRunner, ExportWizardControlState,
    ExportWizardJobCompletion, ExportWizardJobController, ExportWizardJobEvent,
    ExportWizardJobEventKind, ExportWizardJobSnapshot, ExportWizardJobState, ExportWizardJobStatus,
    ExportWizardNeverCancel, ExportWizardPanelAction, ExportWizardPanelBinding,
    ExportWizardPanelControlBindingState, ExportWizardPanelEntrySeverity, ExportWizardPanelRequest,
    ExportWizardPanelSession, ExportWizardPanelSessionError, ExportWizardPanelSlotEntry,
    ExportWizardPanelSlotKind, ExportWizardPanelSlotState, ExportWizardPanelTemplateState,
    ExportWizardPanelUpdate, ExportWizardPanelViewModel, ExportWizardPipelineExecution,
    ExportWizardPipelineOptions, ExportWizardPipelinePlan, ExportWizardPipelineStageCommand,
    ExportWizardProgressState, ExportWizardStageArtifactPath, ExportWizardStageExecution,
    ExportWizardStageMissingInputs, ExportWizardStagePlannedArtifacts,
    ExportWizardStageProgressSnapshot, ExportWizardStageViewRow, ExportWizardStreamEvent,
    ProcessCommandRunner, DESKTOP_EXPORT_ARTIFACT_PATHS_SLOT, DESKTOP_EXPORT_CANCEL_BINDING_ID,
    DESKTOP_EXPORT_CANCEL_BUTTON, DESKTOP_EXPORT_GENERATE_PLAN_BINDING_ID,
    DESKTOP_EXPORT_GENERATE_PLAN_BUTTON, DESKTOP_EXPORT_MISSING_INPUTS_SLOT,
    DESKTOP_EXPORT_REPORT_BODY_SLOT, DESKTOP_EXPORT_STAGE_ROWS_SLOT,
    DESKTOP_EXPORT_START_BINDING_ID, DESKTOP_EXPORT_START_BUTTON,
    DESKTOP_EXPORT_TERMINAL_OUTPUT_SLOT, EXPORT_WIZARD_BINDING_SYMBOL,
    EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID, EXPORT_WIZARD_VIEW_ID,
};
pub use editor_subsystems::{
    EditorSubsystemReport, EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
    EDITOR_RUNTIME_SANDBOX_ENABLED_CONFIG_KEY, EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
    EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING, EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
    EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
};
pub use export_process_support::ExportProcessError;
pub use host_capability_bridge::{EditorHostVmBridgeReport, EditorVmExtensionLoadReport};
pub use minimal_host_contract::{
    editor_host_minimal_contract, EditorHostMinimalContract, EditorHostMinimalReport,
};
pub use native_dynamic_export_preparation::NativeDynamicPreparationError;
pub use window_host_manager::NativeWindowHostState;
