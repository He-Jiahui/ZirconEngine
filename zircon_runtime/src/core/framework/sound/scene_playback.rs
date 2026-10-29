use crate::core::framework::scene::{EntityId, SceneResource};

/// A validated sound URI requested by gameplay for an entity in the owning world.
#[derive(Debug)]
pub struct SoundPlayRequest {
    pub entity: EntityId,
    /// Resolve again when consumed: clip IDs belong to a sound manager session.
    pub uri: String,
}

/// Runtime-only world inbox consumed by the sound plugin after world transforms.
/// Dropping/replacing the world also discards requests that have not played yet.
#[derive(Default, Debug)]
pub struct SoundPlayRequests(pub Vec<SoundPlayRequest>);

impl SceneResource for SoundPlayRequests {}
