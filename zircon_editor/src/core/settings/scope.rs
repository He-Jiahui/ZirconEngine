/// Persistence layer for a setting value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettingsScope {
    User,
    Project,
    Session,
}

impl SettingsScope {
    pub(crate) const fn is_persistent(self) -> bool {
        matches!(self, Self::User | Self::Project)
    }

    pub(crate) const fn allows_write(self, requested: Self) -> bool {
        match self {
            Self::User => matches!(requested, Self::User | Self::Session),
            Self::Project => matches!(requested, Self::User | Self::Project | Self::Session),
            Self::Session => matches!(requested, Self::Session),
        }
    }
}
