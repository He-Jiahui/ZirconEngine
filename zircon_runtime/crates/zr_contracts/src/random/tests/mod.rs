mod algorithm;
mod checkpoint;
mod state;

use super::{RandomEntityKey, RandomPurposeKey, RandomStreamKey, RandomSystemKey, RandomWorldKey};

fn key(id: u64) -> RandomStreamKey {
    RandomStreamKey::for_entity(
        RandomWorldKey::new(7, 3),
        RandomEntityKey::new(id, 2),
        RandomSystemKey::new(91),
        RandomPurposeKey::new(5),
        0x5eed,
    )
}
