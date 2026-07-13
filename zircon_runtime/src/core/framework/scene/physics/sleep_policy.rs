use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhysicsSleepPolicy {
    #[default]
    Allow,
    Never,
}

impl PhysicsSleepPolicy {
    pub const fn allows_sleep(self) -> bool {
        matches!(self, Self::Allow)
    }
}
