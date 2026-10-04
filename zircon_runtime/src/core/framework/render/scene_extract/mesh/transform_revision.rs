use crate::core::math::Transform;

/// 以变换内容生成跨帧比较用指纹，供网格和阴影缓存发现姿态变化。
/// 它不是递增代数，也不能替代几何或材质资源的修订。
pub fn render_mesh_transform_revision(transform: &Transform) -> u64 {
    let mut revision = FNV_OFFSET_BASIS;
    for lane in transform.translation.to_array() {
        revision = fnv1a_u32(revision, lane.to_bits());
    }
    for lane in transform.rotation.to_array() {
        revision = fnv1a_u32(revision, lane.to_bits());
    }
    for lane in transform.scale.to_array() {
        revision = fnv1a_u32(revision, lane.to_bits());
    }
    revision
}

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

const fn fnv1a_u32(mut hash: u64, value: u32) -> u64 {
    let bytes = value.to_le_bytes();
    let mut index = 0;
    while index < bytes.len() {
        hash ^= bytes[index] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        index += 1;
    }
    hash
}
