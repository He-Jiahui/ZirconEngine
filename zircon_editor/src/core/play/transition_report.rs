use super::{PendingEditDecisionPrompt, PlayModeKind, PluginBridgeActivationReport};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayTransitionReport {
    pub changed: bool,
    pub mode: PlayModeKind,
    pub activation: PluginBridgeActivationReport,
    pub backend_diagnostics: Vec<String>,
    pub backend_attachable: bool,
    pub cause: PlayTransitionCause,
    pub pending_edit_prompt: Option<PendingEditDecisionPrompt>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PlayTransitionCause {
    #[default]
    Unchanged,
    Started,
    Stopped,
    BuildFailed,
    Crashed {
        exit_code: Option<i32>,
    },
}

impl PlayTransitionReport {
    pub(super) fn unchanged(mode: PlayModeKind) -> Self {
        Self {
            changed: false,
            mode,
            activation: PluginBridgeActivationReport::default(),
            backend_diagnostics: Vec::new(),
            backend_attachable: false,
            cause: PlayTransitionCause::Unchanged,
            pending_edit_prompt: None,
        }
    }

    pub(super) fn unchanged_with_backend(
        mode: PlayModeKind,
        backend_diagnostics: Vec<String>,
    ) -> Self {
        Self {
            changed: false,
            mode,
            activation: PluginBridgeActivationReport::default(),
            backend_diagnostics,
            backend_attachable: false,
            cause: PlayTransitionCause::Unchanged,
            pending_edit_prompt: None,
        }
    }

    pub(super) fn changed(
        mode: PlayModeKind,
        activation: PluginBridgeActivationReport,
        backend_diagnostics: Vec<String>,
        backend_attachable: bool,
        cause: PlayTransitionCause,
    ) -> Self {
        Self {
            changed: true,
            mode,
            activation,
            backend_diagnostics,
            backend_attachable,
            cause,
            pending_edit_prompt: None,
        }
    }

    pub(super) fn with_pending_edit_prompt(
        mut self,
        prompt: Option<PendingEditDecisionPrompt>,
    ) -> Self {
        self.pending_edit_prompt = prompt;
        self
    }
}
