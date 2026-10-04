use super::{PluginPackageManifest, PluginPackageRole};

#[test]
fn exact_package_coordinate_id_preserves_qualified_and_fallback_identity() {
    let qualified = PluginPackageManifest::new("legacy_weather", "Weather")
        .with_package_identity("org", "zircon", "weather");
    assert_eq!(qualified.package_id(), "org.zircon.weather");

    let fallback = PluginPackageManifest::new("legacy_weather", "Weather")
        .with_package_identity("", "zircon", "weather");
    assert_eq!(fallback.package_id(), "legacy_weather");
}

#[test]
fn package_role_defaults_to_production_and_round_trips_carrier_roles() {
    let legacy: PluginPackageManifest =
        toml::from_str("id = \"legacy\"\nversion = \"0.1.0\"\ndisplay_name = \"Legacy\"\n")
            .expect("legacy package manifest should deserialize");
    assert_eq!(legacy.package_role, PluginPackageRole::Production);

    let fixture = PluginPackageManifest::new("fixture", "Fixture")
        .with_package_role(PluginPackageRole::TestFixture);
    let encoded = toml::to_string(&fixture).expect("fixture package manifest should serialize");
    assert!(encoded.contains("package_role = \"test_fixture\""));
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("fixture package manifest should deserialize");
    assert_eq!(decoded.package_role, PluginPackageRole::TestFixture);
}
