#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum EditorStartupStage {
    #[default]
    RuntimeLoad,
    GpuInit,
    ProjectOpen,
    Ready,
}

impl EditorStartupStage {
    pub(super) const ALL: [Self; 4] = [
        Self::RuntimeLoad,
        Self::GpuInit,
        Self::ProjectOpen,
        Self::Ready,
    ];

    pub(super) fn wire(self) -> &'static str {
        match self {
            Self::RuntimeLoad => "runtime",
            Self::GpuInit => "gpu",
            Self::ProjectOpen => "project",
            Self::Ready => "ready",
        }
    }

    pub(super) fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|stage| stage.wire() == value)
    }

    pub(super) fn text_key(self, project_requested: bool) -> &'static str {
        match self {
            Self::RuntimeLoad => "editor.startup.runtime",
            Self::GpuInit => "editor.startup.gpu",
            Self::ProjectOpen if project_requested => "editor.startup.project",
            Self::ProjectOpen => "editor.startup.welcome",
            Self::Ready => "editor.startup.ready",
        }
    }
}

#[cfg(test)]
#[path = "tests/stage.rs"]
mod tests;
