use crate::core::framework::scene::EntityId;

/// 粒子跨帧身份由实体与发射流稳定键共同组成；匿名同实体多精灵需避免误配。
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
