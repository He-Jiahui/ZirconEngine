mod capability;
mod export_wizard;
mod extension_ids;
mod plugin;

pub use capability::{
    CAPABILITY, DIAGNOSTICS_CAPABILITY, EDITOR_CAPABILITIES, NATIVE_DYNAMIC_REPORT_CAPABILITY,
    PLUGIN_ID,
};
pub use export_wizard::{
    export_wizard_descriptor, stage_progress_kinds, ExportWizardAction, ExportWizardDescriptor,
    ExportWizardRegion, ExportWizardRegionDescriptor, ExportWizardReportViewDescriptor,
    ExportWizardStageDescriptor, BUILD_EXPORT_LAYOUT_REFERENCE,
    EXPORT_PLAN_REPORT_SUMMARY_ENTRY_KEYS, LIBRARY_EMBED_REPORT_SUMMARY_ENTRY_KEYS,
    LIBRARY_EMBED_REPORT_TEMPLATE_CONTROL_IDS, NATIVE_DYNAMIC_REPORT_SUMMARY_ENTRY_KEYS,
    NATIVE_DYNAMIC_REPORT_TEMPLATE_CONTROL_IDS, PIPELINE_REPORT_PATH,
    REPORT_EXPORT_PLAN_COMPLETED_STAGES_ENTRY_KEY, REPORT_EXPORT_PLAN_REQUIRED_STAGES_ENTRY_KEY,
    REPORT_EXPORT_PLAN_STRATEGIES_ENTRY_KEY, REPORT_EXPORT_PLAN_UNSUPPORTED_STRATEGIES_ENTRY_KEY,
    REPORT_NATIVE_PLUGINS_PAYLOAD_BUNDLE_PATH_ENTRY_KEY,
    REPORT_NATIVE_PLUGINS_PAYLOAD_CONTENT_HASH_ENTRY_KEY,
    REPORT_NATIVE_PLUGINS_PAYLOAD_FILE_COUNT_ENTRY_KEY,
    REPORT_NATIVE_PLUGINS_PAYLOAD_PACKAGE_COUNT_ENTRY_KEY,
    REPORT_NATIVE_PLUGINS_PAYLOAD_PACKAGE_IDS_ENTRY_KEY, REPORT_PIPELINE_REPORT_ENTRY_KEY,
    SOURCE_TEMPLATE_REPORT_SUMMARY_ENTRY_KEYS, SOURCE_TEMPLATE_REPORT_TEMPLATE_CONTROL_IDS,
};
pub use extension_ids::{
    EXPORT_DRAWER_ID, EXPORT_OPERATION_CREATE_PROFILE, EXPORT_OPERATION_GENERATE_PLAN,
    EXPORT_OPERATION_LIBRARY_EMBED, EXPORT_OPERATION_NATIVE_DYNAMIC,
    EXPORT_OPERATION_OPEN_DIAGNOSTICS, EXPORT_OPERATION_OPEN_PROFILE,
    EXPORT_OPERATION_SOURCE_TEMPLATE, EXPORT_PANEL_TEMPLATE_DOCUMENT, EXPORT_PROFILE_ASSET_KIND,
    EXPORT_PROFILE_COMPONENT, EXPORT_PROFILE_DRAWER_DOCUMENT, EXPORT_PROFILE_TEMPLATE_DOCUMENT,
    EXPORT_REPORT_TEMPLATE_DOCUMENTS, EXPORT_TEMPLATE_ID, EXPORT_UI_TEMPLATE_DOCUMENTS,
    EXPORT_VIEW_ID, LIBRARY_EMBED_REPORT_DOCUMENT, LIBRARY_EMBED_REPORT_ID,
    NATIVE_DYNAMIC_REPORT_DOCUMENT, NATIVE_DYNAMIC_REPORT_ID, SOURCE_TEMPLATE_REPORT_DOCUMENT,
    SOURCE_TEMPLATE_REPORT_ID,
};
pub use plugin::{
    editor_build_export_desktop_dist_module_manifest, editor_capabilities, editor_plugin,
    editor_plugin_descriptor, package_manifest, plugin_registration,
    EditorBuildExportDesktopPlugin, EDITOR_BUILD_EXPORT_DESKTOP_DIST_CRATE_NAME,
    EDITOR_BUILD_EXPORT_DESKTOP_DIST_EDITOR_ENTRY,
};
pub use zircon_editor::{
    apply_export_wizard_panel_template_state, execute_export_wizard_pipeline,
    execute_export_wizard_stage, export_wizard_panel_action_call, export_wizard_panel_action_for_control,
    export_wizard_panel_binding_entries, export_wizard_panel_bindings,
    export_wizard_panel_retained_projection, export_wizard_panel_template_state,
    export_wizard_pipeline_plan, project_export_wizard_panel,
    register_export_wizard_panel_bindings, register_export_wizard_panel_template,
    run_export_wizard_job, ExportWizardCancelSignal, ExportWizardCommandExecution,
    ExportWizardCommandRunner, ExportWizardControlState, ExportWizardJobCompletion,
    ExportWizardJobController, ExportWizardJobEvent, ExportWizardJobEventKind,
    ExportWizardJobSnapshot, ExportWizardJobState, ExportWizardJobStatus, ExportWizardNeverCancel,
    ExportWizardPanelAction, ExportWizardPanelBinding, ExportWizardPanelControlBindingState,
    ExportWizardPanelEntrySeverity, ExportWizardPanelRequest, ExportWizardPanelSession,
    ExportWizardPanelSessionError, ExportWizardPanelSlotEntry, ExportWizardPanelSlotKind,
    ExportWizardPanelSlotState, ExportWizardPanelTemplateState, ExportWizardPanelUpdate,
    ExportWizardPanelViewModel, ExportWizardPipelineExecution, ExportWizardPipelineOptions,
    ExportWizardPipelinePlan, ExportWizardPipelineStageCommand, ExportWizardStageExecution,
    ExportWizardStageMissingInputs, ExportWizardStagePlannedArtifacts, ExportWizardStageViewRow,
    ProcessCommandRunner,
};
pub use zircon_editor::{
    ExportStageProgressKind,
    ExportWizardProgressState, ExportWizardStageArtifactPath, ExportWizardStageProgressSnapshot,
    ExportWizardStreamEvent, DESKTOP_EXPORT_ARTIFACT_PATHS_SLOT, DESKTOP_EXPORT_CANCEL_BINDING_ID,
    DESKTOP_EXPORT_CANCEL_BUTTON, DESKTOP_EXPORT_GENERATE_PLAN_BINDING_ID,
    DESKTOP_EXPORT_GENERATE_PLAN_BUTTON, DESKTOP_EXPORT_MISSING_INPUTS_SLOT,
    DESKTOP_EXPORT_REPORT_BODY_SLOT, DESKTOP_EXPORT_STAGE_ROWS_SLOT,
    DESKTOP_EXPORT_START_BINDING_ID, DESKTOP_EXPORT_START_BUTTON,
    DESKTOP_EXPORT_TERMINAL_OUTPUT_SLOT, EXPORT_WIZARD_BINDING_SYMBOL,
    EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID, EXPORT_WIZARD_VIEW_ID,
};

#[cfg(test)]
mod tests;
