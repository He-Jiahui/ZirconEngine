use std::sync::{Arc, OnceLock};

use zircon_runtime::plugin::{EditorCoreProfile, RuntimePluginRegistrationReport};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionProfileKind {
    Runtime,
    Editor,
    Dev,
    Minimal,
    Headless,
}

impl SessionProfileKind {
    pub fn from_profile_bytes(profile: &[u8]) -> Option<Self> {
        match profile {
            [] | b"runtime" => Some(Self::Runtime),
            b"editor" => Some(Self::Editor),
            b"dev" => Some(Self::Dev),
            b"minimal" => Some(Self::Minimal),
            b"headless" => Some(Self::Headless),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PluginActivationState {
    Active,
    Disabled,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSummaryEntry {
    id: String,
    version: String,
    activation: PluginActivationState,
}

impl PluginSummaryEntry {
    pub fn new(
        id: impl Into<String>,
        version: impl Into<String>,
        activation: PluginActivationState,
    ) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            activation,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn activation(&self) -> PluginActivationState {
        self.activation
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeCapabilities {
    session_profile: SessionProfileKind,
    core_capabilities: Vec<String>,
    plugin_summary: Vec<PluginSummaryEntry>,
}

impl RuntimeCapabilities {
    pub fn new<I, S, P>(
        session_profile: SessionProfileKind,
        core_capabilities: I,
        plugin_summary: P,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
        P: IntoIterator<Item = PluginSummaryEntry>,
    {
        let mut core_capabilities = core_capabilities
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        core_capabilities.sort();
        core_capabilities.dedup();

        let mut plugin_summary = plugin_summary.into_iter().collect::<Vec<_>>();
        plugin_summary.sort_by(|left, right| {
            (&left.id, &left.version, left.activation).cmp(&(
                &right.id,
                &right.version,
                right.activation,
            ))
        });
        plugin_summary.dedup();

        Self {
            session_profile,
            core_capabilities,
            plugin_summary,
        }
    }

    pub fn editor_default() -> Self {
        Self::new(
            SessionProfileKind::Editor,
            EditorCoreProfile::minimal().required_capabilities,
            [],
        )
    }

    pub fn from_runtime_plugin_registrations(
        session_profile: SessionProfileKind,
        registrations: &[RuntimePluginRegistrationReport],
    ) -> Self {
        let plugin_summary = registrations.iter().map(|registration| {
            let activation = if !registration.project_selection.enabled {
                PluginActivationState::Disabled
            } else if registration.is_success() {
                PluginActivationState::Active
            } else {
                PluginActivationState::Rejected
            };
            PluginSummaryEntry::new(
                &registration.package_manifest.id,
                &registration.package_manifest.version,
                activation,
            )
        });
        Self::new(
            session_profile,
            EditorCoreProfile::minimal().required_capabilities,
            plugin_summary,
        )
    }

    pub fn session_profile(&self) -> SessionProfileKind {
        self.session_profile
    }

    pub fn core_capabilities(&self) -> &[String] {
        &self.core_capabilities
    }

    pub fn plugin_summary(&self) -> &[PluginSummaryEntry] {
        &self.plugin_summary
    }

    pub(crate) fn unavailable() -> Arc<Self> {
        static CAPABILITIES: OnceLock<Arc<RuntimeCapabilities>> = OnceLock::new();
        CAPABILITIES
            .get_or_init(|| {
                Arc::new(Self::new(
                    SessionProfileKind::Minimal,
                    Vec::<String>::new(),
                    [],
                ))
            })
            .clone()
    }
}
