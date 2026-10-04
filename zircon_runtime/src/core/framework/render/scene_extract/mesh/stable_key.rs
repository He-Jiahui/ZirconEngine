use crate::core::framework::scene::EntityId;

pub const RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS: u32 = 16;
pub const RENDER_MESH_STABLE_KEY_MAX_PRIMITIVE_ORDINAL: u32 =
    (1_u32 << RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS) - 1;

// BUG: [CR-RUNTIME-SCENE-EXTRACT-0001] 世界提取未限制 primitive 序号；超过 16 位时 release 构建会生成与其他 primitive 相同的键。
/// 为跨帧可见性、拾取和绘制缓存生成实体内稳定的 primitive 键。
/// 调用方必须保证序号不超过 RENDER_MESH_STABLE_KEY_MAX_PRIMITIVE_ORDINAL。
pub fn render_mesh_stable_instance_key(entity: EntityId, primitive_ordinal: u32) -> u64 {
    debug_assert!(
        primitive_ordinal <= RENDER_MESH_STABLE_KEY_MAX_PRIMITIVE_ORDINAL,
        "render mesh primitive ordinal exceeds stable instance key packing range"
    );
    assert!(
        entity <= (u64::MAX >> RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS),
        "entity id exceeds stable instance key packing range"
    );
    (entity << RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS) | u64::from(primitive_ordinal)
}
