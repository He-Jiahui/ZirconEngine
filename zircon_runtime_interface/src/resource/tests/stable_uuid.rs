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
