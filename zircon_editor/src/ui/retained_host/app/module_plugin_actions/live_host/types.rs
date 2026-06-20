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
}
