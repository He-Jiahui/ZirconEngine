use serde::{Deserialize, Serialize};

/// Stable World identity used for deterministic random-stream derivation.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct RandomWorldKey {
    id: u64,
    generation: u64,
}

impl RandomWorldKey {
    pub const fn new(id: u64, generation: u64) -> Self {
        Self { id, generation }
    }

    pub const fn id(self) -> u64 {
        self.id
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

/// Stable entity identity used for deterministic random-stream derivation.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct RandomEntityKey {
    id: u64,
    generation: u64,
}

impl RandomEntityKey {
    pub const fn new(id: u64, generation: u64) -> Self {
        Self { id, generation }
    }

    pub const fn id(self) -> u64 {
        self.id
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

/// Stable compiled-system identity used for deterministic random-stream derivation.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct RandomSystemKey(u64);

impl RandomSystemKey {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Stable call-site or effect-purpose identity within one system.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct RandomPurposeKey(u64);

impl RandomPurposeKey {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Complete stable owner key for one deterministic random stream.
///
/// The key deliberately has no frame counter, wall time, pointer, or execution
/// order field. Callers that need distinct streams must provide a stable purpose.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct RandomStreamKey {
    world: RandomWorldKey,
    entity: Option<RandomEntityKey>,
    system: RandomSystemKey,
    purpose: RandomPurposeKey,
    authoring_seed: u64,
}

impl RandomStreamKey {
    pub const fn for_world(
        world: RandomWorldKey,
        system: RandomSystemKey,
        purpose: RandomPurposeKey,
        authoring_seed: u64,
    ) -> Self {
        Self {
            world,
            entity: None,
            system,
            purpose,
            authoring_seed,
        }
    }

    pub const fn for_entity(
        world: RandomWorldKey,
        entity: RandomEntityKey,
        system: RandomSystemKey,
        purpose: RandomPurposeKey,
        authoring_seed: u64,
    ) -> Self {
        Self {
            world,
            entity: Some(entity),
            system,
            purpose,
            authoring_seed,
        }
    }

    pub const fn world(self) -> RandomWorldKey {
        self.world
    }

    pub const fn entity(self) -> Option<RandomEntityKey> {
        self.entity
    }

    pub const fn system(self) -> RandomSystemKey {
        self.system
    }

    pub const fn purpose(self) -> RandomPurposeKey {
        self.purpose
    }

    pub const fn authoring_seed(self) -> u64 {
        self.authoring_seed
    }
}
