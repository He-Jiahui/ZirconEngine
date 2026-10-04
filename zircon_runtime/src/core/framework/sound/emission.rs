use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{EntityId, WorldHandle};

pub const SOUND_GAMEPLAY_EMISSION_CAPACITY: usize = 1_024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundGameplayEmitter {
    pub world: WorldHandle,
    pub entity: EntityId,
}

/// A sound-owned, runtime-neutral gameplay emission consumed by optional systems such as AI.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundGameplayEmission {
    pub sequence: u64,
    pub world: WorldHandle,
    pub source: EntityId,
    pub position: [f32; 3],
    pub strength: f32,
    pub max_range: Option<f32>,
    pub emitted_at_seconds: f64,
}

/// 每个世界独立的有界日志读取结果；next_sequence 是下次游标，missed_events 表示已覆盖的历史。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SoundGameplayEmissionRead {
    pub events: Vec<SoundGameplayEmission>,
    pub next_sequence: u64,
    pub missed_events: u64,
}
