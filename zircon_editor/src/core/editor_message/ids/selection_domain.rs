use serde::{Deserialize, Serialize};

use crate::core::play::{PlayInstanceId, WorldDomain};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 选择消息的权威域，同时也是最新态合并键；Play 场景保留实例身份，避免重启后的选择覆盖旧实例。
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
