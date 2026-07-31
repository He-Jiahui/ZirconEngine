#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PreferenceStorageBackendKind {
    Unavailable,
    AtomicFile,
    HostProvided,
}

impl PreferenceStorageBackendKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::AtomicFile => "atomic_file",
            Self::HostProvided => "host_provided",
        }
    }

    pub const fn is_persistent(self) -> bool {
        !matches!(self, Self::Unavailable)
    }
}
