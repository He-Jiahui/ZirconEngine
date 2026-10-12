use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec2, Vec3};

use super::{
    RenderParticleBillboardBasisSnapshot, RenderParticleSpriteIdentity,
    RenderParticleSpriteSnapshot,
};

/// 成功提交的上一帧粒子状态，用复合身份与当前帧配对以构造运动向量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderParticlePreviousSpriteSnapshot {
    pub entity: EntityId,
    pub stable_sprite_key: u64,
    pub position: Vec3,
    pub size: Real,
    pub aspect_ratio: Real,
    pub billboard_offset: Vec2,
    pub rotation: Real,
    pub billboard_basis: Option<RenderParticleBillboardBasisSnapshot>,
}

impl RenderParticlePreviousSpriteSnapshot {
    pub fn from_current(sprite: &RenderParticleSpriteSnapshot) -> Self {
        Self {
            entity: sprite.entity,
            stable_sprite_key: sprite.stable_sprite_key,
            position: sprite.position,
            size: sprite.size,
            aspect_ratio: sprite.aspect_ratio,
            billboard_offset: sprite.billboard_offset,
            rotation: sprite.rotation,
            billboard_basis: None,
        }
    }

    /// 供提交成功后的历史更新调用，连同该帧相机基向量保存可复现的旧公告板。
    pub fn from_current_with_billboard_basis(
        sprite: &RenderParticleSpriteSnapshot,
        right: Vec3,
        up: Vec3,
    ) -> Self {
        let mut previous = Self::from_current(sprite);
        previous.billboard_basis = Some(RenderParticleBillboardBasisSnapshot::new(right, up));
        previous
    }

    pub const fn identity(&self) -> RenderParticleSpriteIdentity {
        RenderParticleSpriteIdentity::new(self.entity, self.stable_sprite_key)
    }
}
