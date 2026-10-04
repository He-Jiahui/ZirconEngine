//! Lifecycle declarations for editor-plugin SDK consumers.

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
/// 编辑器插件阶段与外部事实通知的统一词汇；激活和禁用由管理器独占，Play/文档等事件走消息桥。
pub enum EditorPluginLifecycleStage {
    Loaded,
    Enabled,
    Disabled,
    Unloaded,
    HotReloaded,
    EnteredPlayMode,
    ExitedPlayMode,
    SceneChanged,
    AssetChanged,
    UiMessage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginLifecycleEvent {
    stage: EditorPluginLifecycleStage,
    subject: Option<String>,
}

impl EditorPluginLifecycleEvent {
    pub fn new(stage: EditorPluginLifecycleStage) -> Self {
        Self {
            stage,
            subject: None,
        }
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn stage(&self) -> &EditorPluginLifecycleStage {
        &self.stage
    }

    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginLifecycleRecord {
    package_id: String,
    event: EditorPluginLifecycleEvent,
}

impl EditorPluginLifecycleRecord {
    pub fn new(package_id: impl Into<String>, event: EditorPluginLifecycleEvent) -> Self {
        Self {
            package_id: package_id.into(),
            event,
        }
    }

    pub fn package_id(&self) -> &str {
        &self.package_id
    }

    pub fn event(&self) -> &EditorPluginLifecycleEvent {
        &self.event
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// 一次或多次插件回调的有序记录与诊断；任一诊断使该报告失败，但已执行的回调没有自动回滚。
pub struct EditorPluginLifecycleReport {
    records: Vec<EditorPluginLifecycleRecord>,
    diagnostics: Vec<String>,
}

impl EditorPluginLifecycleReport {
    pub fn record(&mut self, record: EditorPluginLifecycleRecord) {
        self.records.push(record);
    }

    pub fn extend(&mut self, report: EditorPluginLifecycleReport) {
        append_or_adopt(&mut self.records, report.records);
        append_or_adopt(&mut self.diagnostics, report.diagnostics);
    }

    pub fn push_diagnostic(&mut self, diagnostic: impl Into<String>) {
        self.diagnostics.push(diagnostic.into());
    }

    pub fn records(&self) -> &[EditorPluginLifecycleRecord] {
        &self.records
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

fn append_or_adopt<T>(target: &mut Vec<T>, mut incoming: Vec<T>) {
    if target.is_empty() && target.capacity() == 0 {
        *target = incoming;
    } else {
        target.append(&mut incoming);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorPluginLifecycleError {
    stage: EditorPluginLifecycleStage,
    message: String,
}

impl EditorPluginLifecycleError {
    pub fn new(stage: EditorPluginLifecycleStage, message: impl Into<String>) -> Self {
        Self {
            stage,
            message: message.into(),
        }
    }

    pub fn stage(&self) -> &EditorPluginLifecycleStage {
        &self.stage
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for EditorPluginLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "editor plugin lifecycle {:?} failed: {}",
            self.stage, self.message
        )
    }
}

impl std::error::Error for EditorPluginLifecycleError {}

#[cfg(test)]
#[path = "tests/lifecycle_optimization_tests.rs"]
mod optimization_tests;
