#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskStatus {
    pub label: String,
    pub detail: String,
    pub running: bool,
}

impl TaskStatus {
    pub fn idle() -> Self {
        Self {
            label: "Ready".to_string(),
            detail: "Hub is ready".to_string(),
            running: false,
        }
    }

    pub fn running(label: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            detail: detail.into(),
            running: true,
        }
    }
}
