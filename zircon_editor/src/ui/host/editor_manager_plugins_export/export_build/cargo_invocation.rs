#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorExportCargoInvocation {
    pub command: Vec<String>,
    pub status_code: Option<i32>,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}
