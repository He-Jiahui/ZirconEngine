use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ResourceHandle, TextureMarker};

use super::super::super::RenderLayerSet;
use super::RenderParticleSpriteIdentity;

#[derive(Clone, Debug, PartialEq)]
pub struct RenderParticleSpriteSnapshot {
    pub entity: EntityId,
    pub stable_sprite_key: u64,
    pub position: Vec3,
    pub size: Real,
    pub aspect_ratio: Real,
    pub billboard_offset: Vec2,
    pub rotation: Real,
    pub sort_order: i32,
    pub color: Vec4,
    pub intensity: Real,
    pub depth_test: bool,
    pub render_layer_mask: RenderLayerSet,
    pub material: Option<ResourceHandle<MaterialMarker>>,
    pub texture: Option<ResourceHandle<TextureMarker>>,
}

impl RenderParticleSpriteSnapshot {
    pub const fn identity(&self) -> RenderParticleSpriteIdentity {
        RenderParticleSpriteIdentity::new(self.entity, self.stable_sprite_key)
    }
}

impl Default for RenderParticleSpriteSnapshot {
    fn default() -> Self {
        Self {
            entity: 0,
            stable_sprite_key: 0,
            position: Vec3::ZERO,
            size: 0.0,
            aspect_ratio: 1.0,
            billboard_offset: Vec2::ZERO,
            rotation: 0.0,
            sort_order: 0,
            color: Vec4::ZERO,
            intensity: 0.0,
            depth_test: true,
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            material: None,
            texture: None,
        }
    }
}
