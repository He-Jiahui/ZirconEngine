#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusTaskProgressSnapshot {
    pub task_id: String,
    pub label: String,
    pub detail: String,
    pub percent: Option<u8>,
    pub tone: StatusTaskProgressTone,
}

impl StatusTaskProgressSnapshot {
    pub fn new(task_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            label: label.into(),
            detail: String::new(),
            percent: None,
            tone: StatusTaskProgressTone::Info,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }

    pub fn with_percent(mut self, percent: impl Into<Option<u8>>) -> Self {
        self.percent = percent.into().map(|percent| percent.min(100));
        self
    }

    pub fn with_tone(mut self, tone: StatusTaskProgressTone) -> Self {
        self.tone = tone;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatusTaskProgressTone {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}
