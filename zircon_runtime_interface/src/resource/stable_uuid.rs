use uuid::Uuid;

/// Version of the byte-level algorithm used by stable resource and asset identities.
/// 固定资产与资源持久身份的字节算法版本；版本或编码改变会改变派生 UUID，需要明确迁移。
///
/// # 参考
///
/// [`crate::resource::AssetUuid::from_stable_label`]、[`crate::resource::ResourceId::from_stable_label`]、[`crate::resource::ResourceId::from_asset_uuid`]。
pub const STABLE_UUID_ALGORITHM_VERSION: u32 = 1;

const STABLE_UUID_DERIVE_KEY_CONTEXT: &str = "zircon stable identity UUID";

// 版本、命名空间和每个组件都按长度分帧，避免不同分组拼接成同一哈希输入。
// AssetUuid 与 ResourceId 使用不同命名空间，保持跨平台持久身份的域隔离。
/// 按版本化的字节编码派生稳定 UUID，供资产和资源身份用不同命名空间共享同一算法。
///
/// # 参数
///
/// - `namespace`：区分身份用途的原始 UTF-8 命名空间。
/// - `components`：有序的 UTF-8 组件借用；顺序、数量和每项字节内容参与身份。
///
/// # 实际字节布局
///
/// 版本按固定大端 u32 写入；命名空间与各组件才使用固定大端 u128 字节长度加内容的分帧，组件数另以固定大端 u128 写入。不同组件分组因此有明确编码边界。
///
/// # 返回与生命周期
///
/// 返回 BLAKE3 派生摘要前 16 字节设置自定义版本 8 与 RFC 变体后的 UUID。相同版本、上下文和输入得到相同值；输入只在调用期间借用，结果不借用它们。
///
/// # 参考
///
/// [`crate::resource::STABLE_UUID_ALGORITHM_VERSION`]、[`crate::resource::AssetUuid::from_stable_label`]、[`crate::resource::ResourceId::from_asset_uuid`]。
pub(crate) fn stable_uuid_from_components(namespace: &str, components: &[&str]) -> Uuid {
    fn update_framed(hasher: &mut blake3::Hasher, bytes: &[u8]) {
        hasher.update(&(bytes.len() as u128).to_be_bytes());
        hasher.update(bytes);
    }

    let mut hasher = blake3::Hasher::new_derive_key(STABLE_UUID_DERIVE_KEY_CONTEXT);
    hasher.update(&STABLE_UUID_ALGORITHM_VERSION.to_be_bytes());
    update_framed(&mut hasher, namespace.as_bytes());
    hasher.update(&(components.len() as u128).to_be_bytes());
    for component in components {
        update_framed(&mut hasher, component.as_bytes());
    }

    // 取前 16 字节后设置自定义版本 8 与 RFC 变体位；这些位操作属于持久算法，应与固定向量一起维护。
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

#[cfg(test)]
#[path = "tests/stable_uuid.rs"]
mod tests;
