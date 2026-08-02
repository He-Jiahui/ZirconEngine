#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDiagnostic {
    pub code: String,
    pub message: String,
    pub hint: String,
}

impl PluginDiagnostic {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        hint: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            hint: hint.into(),
        }
    }
}
