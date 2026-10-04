use uuid::Uuid;

/// Version of the byte-level algorithm used by stable resource and asset identities.
pub const STABLE_UUID_ALGORITHM_VERSION: u32 = 1;

const STABLE_UUID_DERIVE_KEY_CONTEXT: &str = "zircon stable identity UUID";

// 版本、命名空间和每个组件都按长度分帧，避免不同分组拼接成同一哈希输入。
// AssetUuid 与 ResourceId 使用不同命名空间，保持跨平台持久身份的域隔离。
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

    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

#[cfg(test)]
#[path = "tests/stable_uuid.rs"]
mod tests;
