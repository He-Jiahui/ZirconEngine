mod command;
mod controller;
mod execution;
mod job;
mod options;
mod panel_host_projection;
mod panel_projection;
mod plan;
mod progress;
mod run;
mod session;
mod stage;
mod view_model;

pub use command::ExportWizardPipelineStageCommand;
pub use controller::{ExportWizardJobController, ExportWizardJobHandle};
pub use execution::{
    execute_export_wizard_pipeline, execute_export_wizard_stage,
    execute_export_wizard_stage_with_output, execute_export_wizard_stage_with_output_and_cancel,
    ExportWizardCommandExecution, ExportWizardCommandOutputLine, ExportWizardCommandOutputStream,
    ExportWizardCommandRunner, ExportWizardPipelineExecution, ExportWizardStageExecution,
    ProcessCommandRunner,
};
pub use job::{
    ExportWizardJobSnapshot, ExportWizardJobState, ExportWizardJobStatus,
    ExportWizardStageOutputBuffer,
};
pub use options::ExportWizardPipelineOptions;
pub use panel_host_projection::{
    apply_export_wizard_panel_template_state, export_wizard_panel_retained_projection,
};
pub use panel_projection::{
    export_wizard_panel_template_state, ExportWizardPanelControlBindingState,
    ExportWizardPanelEntrySeverity, ExportWizardPanelSlotEntry, ExportWizardPanelSlotKind,
    ExportWizardPanelSlotState, ExportWizardPanelTemplateState, DESKTOP_EXPORT_ARTIFACT_PATHS_SLOT,
    DESKTOP_EXPORT_MISSING_INPUTS_SLOT, DESKTOP_EXPORT_REPORT_BODY_SLOT,
    DESKTOP_EXPORT_STAGE_ROWS_SLOT, DESKTOP_EXPORT_TERMINAL_OUTPUT_SLOT,
};
pub use plan::{
    export_wizard_compile_host_executable_path, export_wizard_compile_host_target_dir,
    export_wizard_pipeline_plan, ExportWizardPipelinePlan,
};
pub use progress::{
    export_pipeline_stages, parse_export_pipeline_stage, ExportStageProgressKind,
    ExportWizardProgressState, ExportWizardStageArtifactPath, ExportWizardStageProgressSnapshot,
    ExportWizardStreamEvent,
};
pub use run::{
    run_export_wizard_job, ExportWizardCancelSignal, ExportWizardJobEvent,
    ExportWizardJobEventKind, ExportWizardNeverCancel,
};
pub use session::{
    export_wizard_panel_action_call, export_wizard_panel_action_for_control,
    export_wizard_panel_binding_entries, export_wizard_panel_bindings, project_export_wizard_panel,
    register_export_wizard_panel_bindings, register_export_wizard_panel_template,
    ExportWizardPanelAction, ExportWizardPanelBinding, ExportWizardPanelRequest,
    ExportWizardPanelSession, ExportWizardPanelSessionError, ExportWizardPanelUpdate,
    DESKTOP_EXPORT_CANCEL_BINDING_ID, DESKTOP_EXPORT_CANCEL_BUTTON,
    DESKTOP_EXPORT_GENERATE_PLAN_BINDING_ID, DESKTOP_EXPORT_GENERATE_PLAN_BUTTON,
    DESKTOP_EXPORT_START_BINDING_ID, DESKTOP_EXPORT_START_BUTTON, EXPORT_WIZARD_BINDING_SYMBOL,
    EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID, EXPORT_WIZARD_VIEW_ID,
};
pub use stage::{export_pipeline_stage_cli_id, export_pipeline_stage_report_name};
pub use view_model::{
    ExportWizardControlState, ExportWizardPanelViewModel, ExportWizardStageMissingInputs,
    ExportWizardStagePlannedArtifacts, ExportWizardStageViewRow,
};

#[cfg(test)]
mod cancellation_tests;
#[cfg(test)]
mod panel_host_projection_tests;
#[cfg(test)]
mod panel_output_tests;
#[cfg(test)]
mod panel_report_body_tests;
#[cfg(test)]
mod pipeline_handoff_tests;
#[cfg(test)]
mod pipeline_launch_tests;
#[cfg(test)]
mod pipeline_report_tests;
#[cfg(test)]
mod session_control_tests;
#[cfg(test)]
mod streaming_output_tests;
#[cfg(test)]
mod tests;
