use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) enum ModulePluginLiveHostCommand {
    Unload,
    HotReload,
}

impl ModulePluginLiveHostCommand {
    pub(in crate::ui::retained_host::app) fn label(self) -> &'static str {
        match self {
            Self::Unload => "unload",
            Self::HotReload => "hot reload",
        }
    }

    pub(super) fn past_tense(self) -> &'static str {
        match self {
            Self::Unload => "unloaded",
            Self::HotReload => "hot reloaded",
        }
    }
}

/// Exact active project session bound to a live plugin operation or watch completion.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::ui::retained_host::app) struct ModulePluginLiveHostProject {
    root: PathBuf,
    instance_id: String,
    session_generation: u64,
}

impl ModulePluginLiveHostProject {
    pub(in crate::ui::retained_host::app) fn new(
        root: PathBuf,
        instance_id: String,
        session_generation: u64,
    ) -> Self {
        Self {
            root,
            instance_id,
            session_generation,
        }
    }

    pub(in crate::ui::retained_host::app) fn root(&self) -> &Path {
        &self.root
    }

    pub(in crate::ui::retained_host::app) fn session_target(&self) -> (PathBuf, String, u64) {
        (
            self.root.clone(),
            self.instance_id.clone(),
            self.session_generation,
        )
    }

    pub(super) fn with_root(mut self, root: PathBuf) -> Self {
        self.root = root;
        self
    }
}

/// A terminal watch result. The host applies it only to its matching active session.
#[derive(Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct ModulePluginLiveHostCompletion {
    pub(in crate::ui::retained_host::app) project: ModulePluginLiveHostProject,
    pub(in crate::ui::retained_host::app) plugin_id: String,
    pub(in crate::ui::retained_host::app) result: Result<String, String>,
}

impl ModulePluginLiveHostCompletion {
    pub(in crate::ui::retained_host::app) fn matches_project(
        &self,
        active: Option<&ModulePluginLiveHostProject>,
    ) -> bool {
        active == Some(&self.project)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct ModulePluginLiveHostOutcome {
    pub(in crate::ui::retained_host::app) plugin_id: String,
    pub(in crate::ui::retained_host::app) command: ModulePluginLiveHostCommand,
    pub(in crate::ui::retained_host::app) diagnostics: Vec<String>,
}

pub(in crate::ui::retained_host::app) struct ModulePluginLiveHostRequest<'a> {
    pub(in crate::ui::retained_host::app) plugin_id: &'a str,
    pub(in crate::ui::retained_host::app) command: ModulePluginLiveHostCommand,
    pub(in crate::ui::retained_host::app) project: &'a ModulePluginLiveHostProject,
}

pub(in crate::ui::retained_host::app) trait ModulePluginLiveHostBackend {
    fn execute(
        &self,
        request: ModulePluginLiveHostRequest<'_>,
    ) -> Result<ModulePluginLiveHostOutcome, String>;

    fn poll_development_watches(
        &self,
        project: Option<&ModulePluginLiveHostProject>,
    ) -> ModulePluginDevelopmentWatchPoll;

    fn loaded_editor_plugin_ids(&self) -> Result<Vec<String>, String>;
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct ModulePluginDevelopmentWatchPoll {
    diagnostics: Vec<String>,
    completions: Vec<ModulePluginLiveHostCompletion>,
    next_deadline: Option<std::time::Instant>,
}

impl ModulePluginDevelopmentWatchPoll {
    pub(super) fn push_diagnostic(&mut self, diagnostic: String) {
        self.diagnostics.push(diagnostic);
    }

    pub(super) fn push_completion(&mut self, completion: ModulePluginLiveHostCompletion) {
        self.completions.push(completion);
    }

    pub(super) fn include_deadline(&mut self, deadline: Option<std::time::Instant>) {
        let Some(deadline) = deadline else {
            return;
        };
        self.next_deadline = Some(
            self.next_deadline
                .map_or(deadline, |current| current.min(deadline)),
        );
    }

    pub(in crate::ui::retained_host::app) fn into_parts(
        self,
    ) -> (
        Vec<String>,
        Vec<ModulePluginLiveHostCompletion>,
        Option<std::time::Instant>,
    ) {
        (self.diagnostics, self.completions, self.next_deadline)
    }
}

#[cfg(test)]
#[path = "tests/types.rs"]
mod tests;
