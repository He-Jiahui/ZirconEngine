use uuid::Uuid;

/// Version of the byte-level algorithm used by stable resource and asset identities.
pub const STABLE_UUID_ALGORITHM_VERSION: u32 = 1;

const STABLE_UUID_DERIVE_KEY_CONTEXT: &str = "zircon stable identity UUID";

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
mod tests {
    use super::stable_uuid_from_components;

    #[test]
    fn stable_uuid_frames_component_boundaries_without_delimiter_aliases() {
        let separate_components = stable_uuid_from_components("namespace", &["a", "b"]);
        let embedded_delimiter = stable_uuid_from_components("namespace", &["a\u{1f}b"]);

        assert_ne!(separate_components, embedded_delimiter);
    }

    #[test]
    fn stable_uuid_declares_custom_uuid_version_and_rfc_variant() {
        let uuid = stable_uuid_from_components("namespace", &["component"]);
        let bytes = uuid.as_bytes();

        assert_eq!(bytes[6] >> 4, 8);
        assert_eq!(bytes[8] & 0b1100_0000, 0b1000_0000);
    }

    #[test]
    fn stable_uuid_v1_matches_fixed_cross_platform_vector() {
        let uuid =
            stable_uuid_from_components("zircon-asset-uuid", &["res://materials/hero.zmaterial"]);

        assert_eq!(uuid.to_string(), "189d05ad-e595-8f2b-94c0-615f977daa11");
    }
}
