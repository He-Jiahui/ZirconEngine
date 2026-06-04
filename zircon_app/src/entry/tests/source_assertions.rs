pub(crate) fn assert_source_order(source: &str, needles: &[&str], message: &str) {
    let mut offset = 0;
    for needle in needles {
        let Some(index) = source[offset..].find(needle) else {
            panic!("{message}: missing `{needle}`");
        };
        offset += index + needle.len();
    }
}

const FIRST_PARTY_RUNTIME_PLUGIN_CRATES: &[&str] = &[
    "zircon_plugin_ai_runtime",
    "zircon_plugin_animation_runtime",
    "zircon_plugin_hybrid_gi_runtime",
    "zircon_plugin_navigation_runtime",
    "zircon_plugin_net_runtime",
    "zircon_plugin_particles_runtime",
    "zircon_plugin_rendering_runtime",
    "zircon_plugin_solari_runtime",
    "zircon_plugin_sound_runtime",
    "zircon_plugin_texture_runtime",
    "zircon_plugin_virtual_geometry_runtime",
];

#[test]
fn app_manifest_depends_on_first_party_catalog_instead_of_plugin_crate_fan_out() {
    let manifest = include_str!("../../../Cargo.toml");

    assert!(
        manifest.contains("zircon_first_party_runtime_catalog"),
        "zircon_app should depend on the first-party provider catalog boundary"
    );
    for crate_name in FIRST_PARTY_RUNTIME_PLUGIN_CRATES {
        assert!(
            !manifest.contains(&format!("dep:{crate_name}")),
            "zircon_app features should not name individual runtime plugin crate `{crate_name}`"
        );
        assert!(
            !manifest.contains(&format!("{crate_name} = {{")),
            "zircon_app dependencies should not directly list individual runtime plugin crate `{crate_name}`"
        );
    }
}

#[test]
fn first_party_runtime_provider_collection_delegates_to_catalog() {
    let source = include_str!("../first_party_runtime_plugins.rs");

    assert!(
        source.contains(
            "zircon_first_party_runtime_catalog::first_party_runtime_plugin_registrations_for_manifest"
        ),
        "app first-party provider collection should delegate to the provider catalog"
    );
    for crate_name in FIRST_PARTY_RUNTIME_PLUGIN_CRATES {
        assert!(
            !source.contains(&format!("{crate_name}::plugin_registration()")),
            "zircon_app provider collection should not call `{crate_name}::plugin_registration()` directly"
        );
    }
}
