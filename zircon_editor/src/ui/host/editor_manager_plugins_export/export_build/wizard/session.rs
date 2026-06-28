use std::fmt;
use std::path::Path;

use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::template_runtime::{
    EditorUiHostRuntime, EditorUiHostRuntimeError, RetainedUiProjection,
};
use zircon_runtime_interface::ui::binding::{UiBindingCall, UiBindingValue};

use super::{
    export_wizard_pipeline_plan, ExportWizardCommandRunner, ExportWizardJobController,
    ExportWizardJobEventKind, ExportWizardJobSnapshot, ExportWizardPanelViewModel,
    ExportWizardPipelineOptions, ExportWizardPipelinePlan, ProcessCommandRunner,
};

pub const EXPORT_WIZARD_VIEW_ID: &str = "editor.build_export_desktop";
pub const EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID: &str = "editor_build_export_desktop.panel";
pub const EXPORT_WIZARD_BINDING_SYMBOL: &str = "DesktopExportWizard";

pub const DESKTOP_EXPORT_GENERATE_PLAN_BUTTON: &str = "DesktopExportGeneratePlanButton";
pub const DESKTOP_EXPORT_START_BUTTON: &str = "DesktopExportStartButton";
pub const DESKTOP_EXPORT_CANCEL_BUTTON: &str = "DesktopExportCancelButton";

pub const DESKTOP_EXPORT_GENERATE_PLAN_BINDING_ID: &str = "DesktopExportWizard/GeneratePlan";
pub const DESKTOP_EXPORT_START_BINDING_ID: &str = "DesktopExportWizard/Start";
pub const DESKTOP_EXPORT_CANCEL_BINDING_ID: &str = "DesktopExportWizard/Cancel";

const EXPORT_WIZARD_PANEL_BINDINGS: [ExportWizardPanelBinding; 3] = [
    ExportWizardPanelBinding {
        binding_id: DESKTOP_EXPORT_GENERATE_PLAN_BINDING_ID,
        control_id: DESKTOP_EXPORT_GENERATE_PLAN_BUTTON,
        event_kind: EditorUiEventKind::Click,
        action: ExportWizardPanelAction::GeneratePlan,
    },
    ExportWizardPanelBinding {
        binding_id: DESKTOP_EXPORT_START_BINDING_ID,
        control_id: DESKTOP_EXPORT_START_BUTTON,
        event_kind: EditorUiEventKind::Click,
        action: ExportWizardPanelAction::Start,
    },
    ExportWizardPanelBinding {
        binding_id: DESKTOP_EXPORT_CANCEL_BINDING_ID,
        control_id: DESKTOP_EXPORT_CANCEL_BUTTON,
        event_kind: EditorUiEventKind::Click,
        action: ExportWizardPanelAction::Cancel,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportWizardPanelAction {
    GeneratePlan,
    Start,
    Cancel,
    Poll,
}

impl ExportWizardPanelAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GeneratePlan => "generate_plan",
            Self::Start => "start",
            Self::Cancel => "cancel",
            Self::Poll => "poll",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "generate_plan" => Some(Self::GeneratePlan),
            "start" => Some(Self::Start),
            "cancel" => Some(Self::Cancel),
            "poll" => Some(Self::Poll),
            _ => None,
        }
    }

    pub fn from_call(call: &UiBindingCall) -> Option<Self> {
        if call.symbol != EXPORT_WIZARD_BINDING_SYMBOL {
            return None;
        }
        call.argument(0)
            .and_then(UiBindingValue::as_str)
            .and_then(Self::from_str)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportWizardPanelBinding {
    pub binding_id: &'static str,
    pub control_id: &'static str,
    pub event_kind: EditorUiEventKind,
    pub action: ExportWizardPanelAction,
}

impl ExportWizardPanelBinding {
    pub fn editor_binding(self) -> EditorUiBinding {
        EditorUiBinding::new(
            EXPORT_WIZARD_VIEW_ID,
            self.control_id,
            self.event_kind,
            EditorUiBindingPayload::Custom(export_wizard_panel_action_call(
                self.action,
                self.control_id,
            )),
        )
    }
}

pub fn export_wizard_panel_bindings() -> &'static [ExportWizardPanelBinding] {
    &EXPORT_WIZARD_PANEL_BINDINGS
}

pub fn export_wizard_panel_binding_entries() -> Vec<(String, EditorUiBinding)> {
    export_wizard_panel_bindings()
        .iter()
        .map(|binding| (binding.binding_id.to_string(), binding.editor_binding()))
        .collect()
}

pub fn register_export_wizard_panel_bindings(
    runtime: &mut EditorUiHostRuntime,
) -> Result<(), EditorUiHostRuntimeError> {
    for (binding_id, binding) in export_wizard_panel_binding_entries() {
        runtime.register_binding(binding_id, binding)?;
    }
    Ok(())
}

pub fn register_export_wizard_panel_template(
    runtime: &mut EditorUiHostRuntime,
    template_path: impl AsRef<Path>,
) -> Result<(), EditorUiHostRuntimeError> {
    runtime.load_builtin_host_templates()?;
    let template_path = template_path.as_ref().to_path_buf();
    let editor_base_style_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/ui/theme/editor_base.zui");
    runtime.register_v2_template_document_files(
        EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID,
        [template_path, editor_base_style_path],
    )?;
    register_export_wizard_panel_bindings(runtime)
}

pub fn project_export_wizard_panel(
    runtime: &EditorUiHostRuntime,
) -> Result<RetainedUiProjection, EditorUiHostRuntimeError> {
    runtime.project_document(EXPORT_WIZARD_TEMPLATE_DOCUMENT_ID)
}

pub fn export_wizard_panel_action_call(
    action: ExportWizardPanelAction,
    control_id: impl Into<String>,
) -> UiBindingCall {
    UiBindingCall::new(EXPORT_WIZARD_BINDING_SYMBOL)
        .with_argument(UiBindingValue::string(action.as_str()))
        .with_argument(UiBindingValue::string(control_id))
}

pub fn export_wizard_panel_action_for_control(
    control_id: &str,
    event_kind: EditorUiEventKind,
) -> Option<ExportWizardPanelAction> {
    export_wizard_panel_bindings()
        .iter()
        .find(|binding| binding.control_id == control_id && binding.event_kind == event_kind)
        .map(|binding| binding.action)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportWizardPanelRequest {
    GeneratePlan {
        job_id: String,
        options: ExportWizardPipelineOptions,
    },
    Start,
    Cancel,
    Poll,
}

impl ExportWizardPanelRequest {
    pub fn generate_plan(job_id: impl Into<String>, options: ExportWizardPipelineOptions) -> Self {
        Self::GeneratePlan {
            job_id: job_id.into(),
            options,
        }
    }

    pub fn from_action(action: ExportWizardPanelAction) -> Option<Self> {
        match action {
            ExportWizardPanelAction::GeneratePlan => None,
            ExportWizardPanelAction::Start => Some(Self::Start),
            ExportWizardPanelAction::Cancel => Some(Self::Cancel),
            ExportWizardPanelAction::Poll => Some(Self::Poll),
        }
    }

    pub fn action(&self) -> ExportWizardPanelAction {
        match self {
            Self::GeneratePlan { .. } => ExportWizardPanelAction::GeneratePlan,
            Self::Start => ExportWizardPanelAction::Start,
            Self::Cancel => ExportWizardPanelAction::Cancel,
            Self::Poll => ExportWizardPanelAction::Poll,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardPanelUpdate {
    pub action: ExportWizardPanelAction,
    pub events_drained: usize,
    pub active_job_id: Option<String>,
    pub latest_event_kind: Option<ExportWizardJobEventKind>,
    pub snapshot: ExportWizardJobSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportWizardPanelSessionError {
    ActionDisabled {
        action: ExportWizardPanelAction,
        reason: &'static str,
    },
    JobAlreadyActive {
        job_id: String,
    },
    NoActiveJob {
        job_id: String,
    },
    Worker(String),
}

impl fmt::Display for ExportWizardPanelSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ActionDisabled { action, reason } => {
                write!(f, "export wizard action {:?} is disabled: {reason}", action)
            }
            Self::JobAlreadyActive { job_id } => {
                write!(f, "export wizard job {job_id} is already active")
            }
            Self::NoActiveJob { job_id } => {
                write!(f, "export wizard job {job_id} is not active")
            }
            Self::Worker(error) => f.write_str(error),
        }
    }
}

impl std::error::Error for ExportWizardPanelSessionError {}

pub struct ExportWizardPanelSession {
    job_id: String,
    plan: ExportWizardPipelinePlan,
    view_model: ExportWizardPanelViewModel,
    controller: Option<ExportWizardJobController>,
}

impl ExportWizardPanelSession {
    pub fn new(job_id: impl Into<String>, plan: ExportWizardPipelinePlan) -> Self {
        let job_id = job_id.into();
        let view_model = ExportWizardPanelViewModel::from_plan(job_id.clone(), &plan);
        Self {
            job_id,
            plan,
            view_model,
            controller: None,
        }
    }

    pub fn from_options(job_id: impl Into<String>, options: ExportWizardPipelineOptions) -> Self {
        Self::new(job_id, export_wizard_pipeline_plan(options))
    }

    pub fn regenerate_plan(
        &mut self,
        job_id: impl Into<String>,
        options: ExportWizardPipelineOptions,
    ) -> Result<(), ExportWizardPanelSessionError> {
        self.replace_plan(job_id, export_wizard_pipeline_plan(options))
    }

    pub fn replace_plan(
        &mut self,
        job_id: impl Into<String>,
        plan: ExportWizardPipelinePlan,
    ) -> Result<(), ExportWizardPanelSessionError> {
        self.reject_when_active(ExportWizardPanelAction::GeneratePlan)?;
        self.job_id = job_id.into();
        self.view_model = ExportWizardPanelViewModel::from_plan(self.job_id.clone(), &plan);
        self.plan = plan;
        Ok(())
    }

    pub fn handle_request(
        &mut self,
        request: ExportWizardPanelRequest,
    ) -> Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError> {
        let action = request.action();
        let events_drained = match request {
            ExportWizardPanelRequest::GeneratePlan { job_id, options } => {
                self.regenerate_plan(job_id, options)?;
                0
            }
            ExportWizardPanelRequest::Start => {
                self.start()?;
                0
            }
            ExportWizardPanelRequest::Cancel => {
                self.request_cancel()?;
                self.view_model.mark_cancel_requested();
                self.poll_events_and_finish_terminal()?
            }
            ExportWizardPanelRequest::Poll => self.poll_events_and_finish_terminal()?,
        };
        Ok(self.update_for_action(action, events_drained))
    }

    pub fn handle_start_request_with_runner(
        &mut self,
        runner: impl ExportWizardCommandRunner + Send + 'static,
    ) -> Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError> {
        self.start_with_runner(runner)?;
        Ok(self.update_for_action(ExportWizardPanelAction::Start, 0))
    }

    pub fn handle_action_call(
        &mut self,
        call: &UiBindingCall,
    ) -> Result<Option<ExportWizardPanelUpdate>, ExportWizardPanelSessionError> {
        let Some(action) = ExportWizardPanelAction::from_call(call) else {
            return Ok(None);
        };
        let request = ExportWizardPanelRequest::from_action(action).ok_or(
            ExportWizardPanelSessionError::ActionDisabled {
                action,
                reason: "generate_plan requires explicit pipeline options",
            },
        )?;
        self.handle_request(request).map(Some)
    }

    pub fn start(&mut self) -> Result<(), ExportWizardPanelSessionError> {
        self.start_with_runner(ProcessCommandRunner)
    }

    pub fn start_with_runner(
        &mut self,
        runner: impl ExportWizardCommandRunner + Send + 'static,
    ) -> Result<(), ExportWizardPanelSessionError> {
        self.reject_when_active(ExportWizardPanelAction::Start)?;
        if !self.view_model.controls().can_start {
            return Err(ExportWizardPanelSessionError::ActionDisabled {
                action: ExportWizardPanelAction::Start,
                reason: "plan is not ready",
            });
        }
        self.controller = Some(ExportWizardJobController::spawn(
            self.job_id.clone(),
            self.plan.clone(),
            runner,
        ));
        self.view_model.mark_job_started();
        Ok(())
    }

    pub fn request_cancel(&self) -> Result<(), ExportWizardPanelSessionError> {
        let Some(controller) = self.controller.as_ref() else {
            return Err(ExportWizardPanelSessionError::NoActiveJob {
                job_id: self.job_id.clone(),
            });
        };
        controller.request_cancel();
        Ok(())
    }

    pub fn poll_events(&mut self) -> usize {
        let Some(controller) = self.controller.as_ref() else {
            return 0;
        };
        self.view_model.drain_events(controller.events())
    }

    pub fn poll_events_and_finish_terminal(
        &mut self,
    ) -> Result<usize, ExportWizardPanelSessionError> {
        let drained = self.poll_events();
        if self.controller.is_some() && self.view_model.snapshot().is_terminal() {
            let _ = self.finish_job()?;
        }
        Ok(drained)
    }

    pub fn finish_job(
        &mut self,
    ) -> Result<Option<ExportWizardJobSnapshot>, ExportWizardPanelSessionError> {
        let Some(controller) = self.controller.take() else {
            return Ok(None);
        };
        let snapshot = controller
            .finish()
            .map_err(ExportWizardPanelSessionError::Worker)?;
        self.view_model.mark_job_finished(&snapshot);
        Ok(Some(snapshot))
    }

    pub fn active_job_id(&self) -> Option<&str> {
        self.controller
            .as_ref()
            .map(|controller| controller.handle().job_id.as_str())
    }

    pub fn view_model(&self) -> &ExportWizardPanelViewModel {
        &self.view_model
    }

    pub fn plan(&self) -> &ExportWizardPipelinePlan {
        &self.plan
    }

    fn update_for_action(
        &self,
        action: ExportWizardPanelAction,
        events_drained: usize,
    ) -> ExportWizardPanelUpdate {
        ExportWizardPanelUpdate {
            action,
            events_drained,
            active_job_id: self.active_job_id().map(str::to_string),
            latest_event_kind: self.view_model.latest_event_kind(),
            snapshot: self.view_model.snapshot().clone(),
        }
    }

    fn reject_when_active(
        &self,
        action: ExportWizardPanelAction,
    ) -> Result<(), ExportWizardPanelSessionError> {
        if self.controller.is_some() {
            return Err(ExportWizardPanelSessionError::JobAlreadyActive {
                job_id: self.job_id.clone(),
            });
        }
        if matches!(action, ExportWizardPanelAction::GeneratePlan) {
            return Ok(());
        }
        Ok(())
    }
}
