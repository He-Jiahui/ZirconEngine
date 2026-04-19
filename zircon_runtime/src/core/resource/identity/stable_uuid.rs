use std::hash::{Hash, Hasher};
use uuid::Uuid;

pub(crate) fn stable_uuid_from_components(namespace: &str, components: &[&str]) -> Uuid {
    fn hash_with(namespace: &str, value: &str) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        namespace.hash(&mut hasher);
        value.hash(&mut hasher);
        hasher.finish()
    }

    let mut joined = namespace.to_string();
    for component in components {
        joined.push('\x1f');
        joined.push_str(component);
    }

    let high = hash_with("zircon-stable-uuid/high", &joined).to_be_bytes();
    let low = hash_with("zircon-stable-uuid/low", &joined).to_be_bytes();
    let mut bytes = [0_u8; 16];
    bytes[..8].copy_from_slice(&high);
    bytes[8..].copy_from_slice(&low);
    Uuid::from_bytes(bytes)
}
