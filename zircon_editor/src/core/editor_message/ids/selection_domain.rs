use serde::{Deserialize, Serialize};

use crate::core::play::{PlayInstanceId, WorldDomain};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SelectionDomain {
    Scene(WorldDomain),
    Asset,
}

impl SelectionDomain {
    pub const fn edit_scene() -> Self {
        Self::Scene(WorldDomain::Edit)
    }

    pub const fn play_scene(instance: PlayInstanceId) -> Self {
        Self::Scene(WorldDomain::Play(instance))
    }
}
