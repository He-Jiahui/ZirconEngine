use super::interface_import_key;

#[test]
fn exact_interface_import_key_preserves_identity() {
    assert_eq!(
        interface_import_key("weather.runtime", "zr.weather.v1"),
        "weather.runtime=>zr.weather.v1"
    );
}
