#[path = "algorithm.rs"]
mod algorithm;
#[path = "bounded.rs"]
mod bounded;
#[path = "registry.rs"]
mod registry;
#[path = "retention.rs"]
mod retention;
#[path = "service.rs"]
mod service;

use zr_contracts::random::{
    RandomEntityKey, RandomPurposeKey, RandomStreamKey, RandomSystemKey, RandomWorldKey,
};

fn key() -> RandomStreamKey {
    keyed(44)
}

fn keyed(entity_id: u64) -> RandomStreamKey {
    RandomStreamKey::for_entity(
        RandomWorldKey::new(7, 3),
        RandomEntityKey::new(entity_id, 2),
        RandomSystemKey::new(91),
        RandomPurposeKey::new(5),
        0x5eed,
    )
}
