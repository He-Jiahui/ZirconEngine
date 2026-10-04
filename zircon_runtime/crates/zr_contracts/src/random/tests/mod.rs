mod algorithm;
mod checkpoint;
mod state;

use super::{RandomEntityKey, RandomPurposeKey, RandomStreamKey, RandomSystemKey, RandomWorldKey};

// checkpoint 测试固定世界、系统、用途和种子，只用实体 ID 区分流键，便于核对顺序与代际约束。
fn key(id: u64) -> RandomStreamKey {
    RandomStreamKey::for_entity(
        RandomWorldKey::new(7, 3),
        RandomEntityKey::new(id, 2),
        RandomSystemKey::new(91),
        RandomPurposeKey::new(5),
        0x5eed,
    )
}
