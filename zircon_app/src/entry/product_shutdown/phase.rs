/// Monotonic product-host phase from composition through terminal teardown.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ProductHostPhase {
    #[default]
    Composing,
    Running,
    Quiescing,
    Draining,
    ReleasingPlatform,
    DestroyingRuntime,
    DeactivatingModules,
    FlushingDiagnostics,
    Exited,
}

impl ProductHostPhase {
    pub(crate) const COUNT: usize = 9;

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Composing => "composing",
            Self::Running => "running",
            Self::Quiescing => "quiescing",
            Self::Draining => "draining",
            Self::ReleasingPlatform => "releasing_platform",
            Self::DestroyingRuntime => "destroying_runtime",
            Self::DeactivatingModules => "deactivating_modules",
            Self::FlushingDiagnostics => "flushing_diagnostics",
            Self::Exited => "exited",
        }
    }

    pub(super) const fn next_shutdown_phase(self) -> Option<Self> {
        match self {
            Self::Composing | Self::Running => Some(Self::Quiescing),
            Self::Quiescing => Some(Self::Draining),
            Self::Draining => Some(Self::ReleasingPlatform),
            Self::ReleasingPlatform => Some(Self::DestroyingRuntime),
            Self::DestroyingRuntime => Some(Self::DeactivatingModules),
            Self::DeactivatingModules => Some(Self::FlushingDiagnostics),
            Self::FlushingDiagnostics => Some(Self::Exited),
            Self::Exited => None,
        }
    }
}
