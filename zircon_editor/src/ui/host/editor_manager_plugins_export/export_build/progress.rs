#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorExportBuildProgress {
    pub stage: String,
    pub percent: u8,
    pub message: String,
}

impl EditorExportBuildProgress {
    pub(super) fn new(stage: impl Into<String>, percent: u8, message: impl Into<String>) -> Self {
        Self {
            stage: stage.into(),
            percent: percent.min(100),
            message: message.into(),
        }
    }
}
