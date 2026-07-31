#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayBackendStartReport {
    pub diagnostics: Vec<String>,
    pub attachable: bool,
}

impl Default for PlayBackendStartReport {
    fn default() -> Self {
        Self {
            diagnostics: Vec::new(),
            attachable: true,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayBackendStopReport {
    pub diagnostics: Vec<String>,
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
