use super::join_string_parts;

#[test]
fn exact_runtime_descriptor_metadata_preserves_identity_and_description() {
    assert_eq!(
        join_string_parts(&["weather", ".runtime"]),
        "weather.runtime"
    );
    assert_eq!(
        join_string_parts(&["Runtime plugin module for ", "Weather"]),
        "Runtime plugin module for Weather"
    );
}
