use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
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
pub struct EditorPluginLifecycleReport {
    records: Vec<EditorPluginLifecycleRecord>,
    diagnostics: Vec<String>,
}

impl EditorPluginLifecycleReport {
    pub fn record(&mut self, record: EditorPluginLifecycleRecord) {
        self.records.push(record);
    }

    pub fn extend(&mut self, report: EditorPluginLifecycleReport) {
        self.records.extend(report.records);
        self.diagnostics.extend(report.diagnostics);
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
