use crate::core::gateway::SharedEditorRuntimeGateway;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayBackendStartFailure {
    message: String,
    retirement_pending: bool,
}

pub struct PlayBackendStartReport {
    pub diagnostics: Vec<String>,
    gateway: Option<SharedEditorRuntimeGateway>,
}

impl PlayBackendStartFailure {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retirement_pending: false,
        }
    }

    pub fn retirement_pending(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            retirement_pending: true,
        }
    }

    pub fn into_parts(self) -> (String, bool) {
        (self.message, self.retirement_pending)
    }
}

impl std::fmt::Display for PlayBackendStartFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for PlayBackendStartFailure {}

impl Default for PlayBackendStartReport {
    fn default() -> Self {
        Self {
            diagnostics: Vec::new(),
            gateway: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayBackendStopReport {
    pub diagnostics: Vec<String>,
    pub retirement_pending: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayBackendRetireReport {
    pub diagnostics: Vec<String>,
}

impl PlayBackendStartReport {
    pub fn with_diagnostics(mut self, diagnostics: Vec<String>) -> Self {
        self.diagnostics = diagnostics;
        self
    }

    pub fn with_gateway(diagnostics: Vec<String>, gateway: SharedEditorRuntimeGateway) -> Self {
        Self {
            diagnostics,
            gateway: Some(gateway),
        }
    }

    pub fn attachable(&self) -> bool {
        self.gateway.is_some()
    }

    pub(crate) fn take_gateway(&mut self) -> Option<SharedEditorRuntimeGateway> {
        self.gateway.take()
    }
}

impl std::fmt::Debug for PlayBackendStartReport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PlayBackendStartReport")
            .field("diagnostics", &self.diagnostics)
            .field(
                "gateway_identity",
                &self
                    .gateway
                    .as_ref()
                    .map(|gateway| gateway.session_identity()),
            )
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlayBackendPoll {
    Running {
        diagnostics: Vec<String>,
    },
    Exited {
        exit_code: Option<i32>,
        diagnostics: Vec<String>,
    },
}
