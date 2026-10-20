use crate::core::framework::scene::EntityId;

pub const RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS: u32 = 16;
pub const RENDER_MESH_STABLE_KEY_MAX_PRIMITIVE_ORDINAL: u32 =
    (1_u32 << RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS) - 1;

/// 为跨帧可见性、拾取和绘制缓存生成实体内稳定的 primitive 键。
/// 序号或实体超出编码域时在所有构建配置中 panic；正常键保持原 64 位编码。
pub fn render_mesh_stable_instance_key(entity: EntityId, primitive_ordinal: u32) -> u64 {
    assert!(
        primitive_ordinal <= RENDER_MESH_STABLE_KEY_MAX_PRIMITIVE_ORDINAL,
        "render mesh primitive ordinal exceeds stable instance key packing range"
    );
    assert!(
        entity <= (u64::MAX >> RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS),
        "entity id exceeds stable instance key packing range"
    );
    (entity << RENDER_MESH_STABLE_KEY_PRIMITIVE_BITS) | u64::from(primitive_ordinal)
}
