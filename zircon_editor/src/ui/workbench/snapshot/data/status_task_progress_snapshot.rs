#[derive(Clone, Debug, PartialEq, Eq)]
/// 状态条主要任务的只读进度；task_id保持身份，percent缺席表示不可确定。
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

    /// 接受host计算比例并限为0..100；完成状态不能从显示文案推断。
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
/// 状态条显示语气，不作为job完成或取消的权威事实。
pub enum StatusTaskProgressTone {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}
