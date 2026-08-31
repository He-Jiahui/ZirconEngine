use crate::core::framework::scene::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderParticleSpriteIdentity {
    pub entity: EntityId,
    pub stable_sprite_key: u64,
}

impl RenderParticleSpriteIdentity {
    pub const fn new(entity: EntityId, stable_sprite_key: u64) -> Self {
        Self {
            entity,
            stable_sprite_key,
        }
    }
}
