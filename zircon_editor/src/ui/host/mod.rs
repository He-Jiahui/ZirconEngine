//! Editor UI host ownership and orchestration surfaces.

#[path = "animation_editor_sessions/mod.rs"]
pub(crate) mod animation_editor_sessions;
#[path = "asset_editor_sessions/mod.rs"]
pub(crate) mod asset_editor_sessions;
#[path = "builtin_layout/mod.rs"]
mod builtin_layout;
#[path = "builtin_views/mod.rs"]
mod builtin_views;
#[path = "command_eval_projection.rs"]
pub(crate) mod command_eval_projection;
#[path = "editor_activity_log.rs"]
pub(crate) mod editor_activity_log;
#[path = "assets/mod.rs"]
pub(crate) mod editor_asset_manager;
#[path = "editor_capabilities.rs"]
mod editor_capabilities;
#[path = "editor_capability_report.rs"]
mod editor_capability_report;
#[path = "editor_document_autosave.rs"]
mod editor_document_autosave;
#[path = "editor_error.rs"]
mod editor_error;
#[path = "editor_event_control_requests.rs"]
mod editor_event_control_requests;
#[path = "editor_event_dispatch.rs"]
mod editor_event_dispatch;
#[path = "editor_event_execution/mod.rs"]
mod editor_event_execution;
#[path = "editor_event_runtime_access/mod.rs"]
mod editor_event_runtime_access;
#[path = "editor_event_runtime_reflection.rs"]
mod editor_event_runtime_reflection;
#[path = "editor_extension_registration.rs"]
mod editor_extension_registration;
#[path = "editor_extension_views.rs"]
mod editor_extension_views;
#[path = "editor_host_activity_decision.rs"]
mod editor_host_activity_decision;
#[path = "editor_host_event_controller.rs"]
mod editor_host_event_controller;
#[path = "editor_host_startup.rs"]
mod editor_host_startup;
#[path = "editor_host_visible_spatial_query.rs"]
mod editor_host_visible_spatial_query;
#[path = "editor_manager.rs"]
mod editor_manager;
#[path = "editor_manager_animation_editor.rs"]
mod editor_manager_animation_editor;
#[path = "editor_manager_asset_editor.rs"]
mod editor_manager_asset_editor;
#[path = "editor_manager_asset_workspace.rs"]
mod editor_manager_asset_workspace;
#[path = "editor_manager_layout.rs"]
mod editor_manager_layout;
#[path = "editor_manager_minimal_host.rs"]
mod editor_manager_minimal_host;
#[path = "exports/mod.rs"]
mod editor_manager_plugins_export;
#[path = "editor_manager_project.rs"]
mod editor_manager_project;
#[path = "editor_manager_project_activation_effects.rs"]
mod editor_manager_project_activation_effects;
#[path = "editor_manager_project_session.rs"]
mod editor_manager_project_session;
#[path = "editor_manager_runtime_diagnostics.rs"]
mod editor_manager_runtime_diagnostics;
#[path = "editor_manager_startup.rs"]
mod editor_manager_startup;
#[path = "editor_manager_workspace.rs"]
mod editor_manager_workspace;
#[path = "editor_operation_dispatch.rs"]
mod editor_operation_dispatch;
#[path = "editor_runtime_event_pump_error.rs"]
mod editor_runtime_event_pump_error;
#[path = "editor_save_batch.rs"]
mod editor_save_batch;
#[path = "editor_scene_document_submission.rs"]
mod editor_scene_document_submission;
#[path = "editor_scene_mode_lifecycle.rs"]
mod editor_scene_mode_lifecycle;
#[path = "editor_session_state.rs"]
mod editor_session_state;
#[path = "editor_subsystems.rs"]
mod editor_subsystems;
#[path = "editor_ui_host.rs"]
mod editor_ui_host;
#[path = "editor_world_sync.rs"]
mod editor_world_sync;
#[path = "export_cargo_process.rs"]
mod export_cargo_process;
#[path = "export_process_support/mod.rs"]
mod export_process_support;
#[path = "host_capability_bridge.rs"]
mod host_capability_bridge;
#[path = "layout_commands.rs"]
mod layout_commands;
#[path = "layout_hosts/mod.rs"]
mod layout_hosts;
#[path = "layout_persistence.rs"]
mod layout_persistence;
#[path = "minimal_host_contract.rs"]
pub(crate) mod minimal_host_contract;
#[path = "module.rs"]
pub(crate) mod module;
#[path = "native_dynamic_export_preparation/mod.rs"]
mod native_dynamic_export_preparation;
#[path = "play_hierarchy_projection.rs"]
mod play_hierarchy_projection;
#[path = "play_inspector_projection.rs"]
mod play_inspector_projection;
#[path = "play_pending_decision/mod.rs"]
pub(crate) mod play_pending_decision;
#[path = "project_access.rs"]
pub(crate) mod project_access;
#[path = "project_recovery_decision/mod.rs"]
mod project_recovery_decision;
#[path = "project_session_close/mod.rs"]
mod project_session_close;
#[path = "project_session_transition.rs"]
mod project_session_transition;
#[path = "resource_access.rs"]
pub(crate) mod resource_access;
#[path = "runtime_services.rs"]
mod runtime_services;
#[path = "scene_inspection_publication.rs"]
mod scene_inspection_publication;
#[path = "startup/mod.rs"]
mod startup;
#[path = "ui_asset_promotion.rs"]
mod ui_asset_promotion;
#[path = "view_registry.rs"]
mod view_registry;
#[path = "window_host_manager.rs"]
mod window_host_manager;
#[path = "workspace_state.rs"]
mod workspace_state;

pub use asset_editor_sessions::{
    UiAssetWorkspaceWatchDiagnostics, UiAssetWorkspaceWatchPollReport,
};
pub(crate) use builtin_layout::builtin_hybrid_layout;
pub use editor_capabilities::EditorCapabilitySnapshot;
pub(crate) use editor_document_autosave::CloseAutosavePayloadBudget;
pub use editor_error::{
    AnimationEditorDocumentLoadDiagnostic, AnimationEditorTargetDiagnostic,
    AnimationEditorTargetKind, AnimationEditorTargetUnavailableReason, EditorError,
    UiAssetSaveStage,
};
pub use editor_event_dispatch::{
    EditorEventBindingDispatchError, EditorEventDispatchError, EditorEventDispatcherError,
};
pub use editor_event_execution::{
    AssetEventExecutionError, AssetKindFilterError, EditorEventExecutionError,
    MenuActionExecutionError,
};
pub use editor_event_runtime_access::EditorAssetOperationInvokeError;
pub use editor_host_event_controller::{
    EditorHostEventController, EditorPlaySessionShutdownReceipt,
    EditorPlayStateShutdownDisposition, EditorRuntimeSessionShutdownReceipt,
    EditorTerminalPlayDetachError, RuntimeEventConsumerShutdownDisposition,
    RuntimePlayBackendRetirementDisposition, RuntimePlayGatewayShutdownDisposition,
    RuntimePlaySessionShutdownDisposition,
};
pub(crate) use editor_host_event_controller::{PlayGizmoOverlaySnapshot, PlayGizmoPointerOutcome};
pub(crate) use editor_host_startup::resolve_editor_startup_session;
pub use editor_host_startup::EditorHostStartupSession;
pub use editor_manager::{EditorKeymapService, EditorManager};
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
    EditorPluginFeatureStatus, EditorPluginPackageMetadata, EditorPluginSelectionUpdateReport,
    EditorPluginStatus, EditorPluginStatusReport, ExportStageProgressKind,
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
    PluginEnablementApplied, PluginEnablementCause, PluginEnablementCauseKind,
    PluginEnablementDurable, PluginEnablementOperationReport, PluginEnablementPhase,
    PluginEnablementRecovery, ProcessCommandRunner, DESKTOP_EXPORT_ARTIFACT_PATHS_SLOT,
    DESKTOP_EXPORT_CANCEL_BINDING_ID, DESKTOP_EXPORT_CANCEL_BUTTON,
    DESKTOP_EXPORT_GENERATE_PLAN_BINDING_ID, DESKTOP_EXPORT_GENERATE_PLAN_BUTTON,
    DESKTOP_EXPORT_MISSING_INPUTS_SLOT, DESKTOP_EXPORT_REPORT_BODY_SLOT,
    DESKTOP_EXPORT_STAGE_ROWS_SLOT, DESKTOP_EXPORT_START_BINDING_ID, DESKTOP_EXPORT_START_BUTTON,
    DESKTOP_EXPORT_TERMINAL_OUTPUT_SLOT, EXPORT_WIZARD_BINDING_SYMBOL,
    EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID, EXPORT_WIZARD_VIEW_ID,
};
pub use editor_operation_dispatch::EditorOperationDispatchError;
pub use editor_runtime_event_pump_error::EditorRuntimeEventPumpError;
pub use editor_save_batch::EditorDirtySaveError;
pub(crate) use editor_save_batch::{DirtyDocumentSaveOwner, DirtyDocumentSaveStart};
pub(crate) use editor_scene_document_submission::{
    PreparedActiveSceneReloadDirtyPolicy, PreparedActiveSceneReloadOutcome,
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
pub use play_pending_decision::{
    PlayPendingDecisionPublishError, PlayPendingDecisionReceiptDispatchError,
    PlayPendingDecisionReceiptError, PlayPendingDecisionReceiptRecoveryError,
};
pub(crate) use project_session_close::{
    ProjectCloseCommit, ProjectCloseCoordinator, ProjectCloseCoordinatorPhase, ProjectCloseError,
    ProjectCloseOperation, ProjectCloseReceipt, ProjectCloseTransitionError,
};
pub use window_host_manager::NativeWindowHostState;

#[cfg(test)]
pub(crate) use editor_ui_host::save_completion_tests::{
    register_completion_toolkit, CompletionAction,
};

pub use editor_manager_plugins_export::{
    PluginEnablementAdmission, PluginEnablementRecoveryCommand,
};
