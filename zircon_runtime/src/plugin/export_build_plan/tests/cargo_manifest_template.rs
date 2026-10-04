use super::{cargo_manifest_template, ExportLinkedRuntimeCrate, ExportProfile};

#[test]
fn streaming_cargo_manifest_preserves_contract() {
    let mut profile = ExportProfile::default();
    profile.output_name = "Demo Game++".to_string();
    let linked_crate = ExportLinkedRuntimeCrate::runtime_plugin(
        "zircon_plugin_test_runtime".to_string(),
        "test".to_string(),
    );

    assert_eq!(
        cargo_manifest_template(&profile, &[linked_crate]),
        concat!(
            "[package]\n",
            "name = \"zircon_export_demo_game__\"\n",
            "version = \"0.1.0\"\n",
            "edition = \"2021\"\n",
            "\n[dependencies]\n",
            "zircon_app = { path = \"../../zircon_app\", default-features = false, features = [\"target-client\"] }\n",
            "zircon_runtime = { path = \"../../zircon_runtime\", default-features = false }\n",
            "zircon_plugin_test_runtime = { path = \"../../zircon_plugins/test\" }\n",
        )
    );
}
