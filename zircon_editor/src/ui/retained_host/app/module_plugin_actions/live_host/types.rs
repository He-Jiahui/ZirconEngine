#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) enum ModulePluginLiveHostCommand {
    Unload,
    HotReload,
}

impl ModulePluginLiveHostCommand {
    #[cfg(test)]
    pub(super) fn label(self) -> &'static str {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct ModulePluginLiveHostOutcome {
    pub(in crate::ui::retained_host::app) plugin_id: String,
    pub(in crate::ui::retained_host::app) command: ModulePluginLiveHostCommand,
    pub(in crate::ui::retained_host::app) diagnostics: Vec<String>,
}

pub(in crate::ui::retained_host::app) struct ModulePluginLiveHostRequest<'a> {
    pub(in crate::ui::retained_host::app) plugin_id: &'a str,
    pub(in crate::ui::retained_host::app) command: ModulePluginLiveHostCommand,
    pub(in crate::ui::retained_host::app) project_root: &'a std::path::Path,
}

pub(in crate::ui::retained_host::app) trait ModulePluginLiveHostBackend {
    fn execute(
        &self,
        request: ModulePluginLiveHostRequest<'_>,
    ) -> Result<ModulePluginLiveHostOutcome, String>;

    fn poll_development_watches(&self) -> ModulePluginDevelopmentWatchPoll;
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct ModulePluginDevelopmentWatchPoll {
    diagnostics: Vec<String>,
    next_deadline: Option<std::time::Instant>,
}

impl ModulePluginDevelopmentWatchPoll {
    pub(super) fn push_diagnostic(&mut self, diagnostic: String) {
        self.diagnostics.push(diagnostic);
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
    ) -> (Vec<String>, Option<std::time::Instant>) {
        (self.diagnostics, self.next_deadline)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::ModulePluginDevelopmentWatchPoll;

    #[test]
    fn development_watch_poll_keeps_the_earliest_host_wake_deadline() {
        let now = Instant::now();
        let mut poll = ModulePluginDevelopmentWatchPoll::default();

        poll.include_deadline(Some(now + Duration::from_secs(2)));
        poll.include_deadline(None);
        poll.include_deadline(Some(now + Duration::from_secs(1)));

        let (diagnostics, deadline) = poll.into_parts();
        assert!(diagnostics.is_empty());
        assert_eq!(deadline, Some(now + Duration::from_secs(1)));
    }
}
