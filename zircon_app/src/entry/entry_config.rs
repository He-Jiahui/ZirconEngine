use super::entry_profile::EntryProfile;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryConfig {
    pub profile: EntryProfile,
}

impl EntryConfig {
    pub const fn new(profile: EntryProfile) -> Self {
        Self { profile }
    }
}
