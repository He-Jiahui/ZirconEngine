use super::*;

#[test]
fn runtime_15_f8_api_convergence_review_guards_are_child_owners() {
    let sources = read_f8_review_sources();

    assert_contains_all(
        "F8 API convergence parent mounts focused child owners",
        &sources.parent,
        &[
            "#[path = \"f8_api_convergence/texture_import_settings.rs\"]",
            "mod texture_import_settings;",
            "#[path = \"f8_api_convergence/descriptor_builder.rs\"]",
            "mod descriptor_builder;",
            "#[path = \"f8_api_convergence/descriptor_privacy.rs\"]",
            "mod descriptor_privacy;",
        ],
    );
    assert_eq!(
        sources.parent.matches("#[test]").count(),
        0,
        "f8_api_convergence.rs should only mount child review guard owners"
    );
    for child_owned_test in REVIEW_GUARDS {
        assert!(
            !sources.parent.contains(&format!("fn {child_owned_test}")),
            "child-owned F8 review guard `{child_owned_test}` should not return to f8_api_convergence.rs"
        );
    }

    assert_contains_all(
        "F8 descriptor builder route mounts focused child owners",
        &sources.descriptor_builder,
        &[
            "#[path = \"descriptor_builder/first_party_descriptors.rs\"]",
            "mod first_party_descriptors;",
            "#[path = \"descriptor_builder/scaffold.rs\"]",
            "mod scaffold;",
            "#[path = \"descriptor_builder/test_fixtures.rs\"]",
            "mod test_fixtures;",
        ],
    );
    assert_eq!(
        sources.descriptor_builder.matches("#[test]").count(),
        0,
        "descriptor_builder.rs should only mount descriptor builder review guard owners"
    );
    for child_owned_test in &REVIEW_GUARDS[1..4] {
        assert!(
            !sources
                .descriptor_builder
                .contains(&format!("fn {child_owned_test}")),
            "child-owned F8 descriptor builder guard `{child_owned_test}` should not return to descriptor_builder.rs"
        );
    }

    assert_contains_all(
        "F8 descriptor privacy route mounts focused child owners",
        &sources.descriptor_privacy,
        &[
            "#[path = \"descriptor_privacy/constructor_retirement.rs\"]",
            "mod constructor_retirement;",
            "#[path = \"descriptor_privacy/private_fields.rs\"]",
            "mod private_fields;",
            "#[path = \"descriptor_privacy/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );
    assert_eq!(
        sources.descriptor_privacy.matches("#[test]").count(),
        0,
        "descriptor_privacy.rs should only mount descriptor privacy review guard owners"
    );
    for child_owned_test in &REVIEW_GUARDS[4..7] {
        assert!(
            !sources
                .descriptor_privacy
                .contains(&format!("fn {child_owned_test}")),
            "child-owned F8 descriptor privacy guard `{child_owned_test}` should not return to descriptor_privacy.rs"
        );
    }

    assert_contains_all(
        "F8 texture child owns texture apply review guard",
        &sources.texture_import_settings,
        &[
            REVIEW_GUARDS[0],
            "apply_import_settings",
            "TextureDescriptorError",
            "runtime_15_texture_descriptor_typed_errors_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "F8 descriptor scaffold child owns builder scaffold review guard",
        &sources.descriptor_builder_scaffold,
        &[
            REVIEW_GUARDS[1],
            "RuntimePluginDescriptorBuilder",
            "RuntimePluginDescriptor::builder(",
        ],
    );
    assert_contains_all(
        "F8 descriptor first-party child owns production builder migration guard",
        &sources.descriptor_builder_first_party,
        &[
            REVIEW_GUARDS[2],
            "first-party runtime plugin descriptor production files 16/16",
            "RuntimePluginDescriptor::builder(",
        ],
    );
    assert_contains_all(
        "F8 descriptor fixture child owns test fixture builder migration guard",
        &sources.descriptor_builder_test_fixtures,
        &[
            REVIEW_GUARDS[3],
            "plugin extension RuntimePluginDescriptor test fixtures 14/14",
            "RuntimePluginDescriptor::builder(",
        ],
    );
    assert_contains_all(
        "F8 descriptor private-fields child owns privacy review guard",
        &sources.descriptor_privacy_private_fields,
        &[
            REVIEW_GUARDS[4],
            "RuntimePluginDescriptor private fields 15/15",
            "pub fn package_id(&self) -> &str",
        ],
    );
    assert_contains_all(
        "F8 descriptor constructor child owns constructor retirement guard",
        &sources.descriptor_privacy_constructor_retirement,
        &[
            REVIEW_GUARDS[5],
            "RuntimePluginDescriptor::new retired",
            "descriptor/builder/construction.rs retired",
        ],
    );
    assert_contains_all(
        "F8 descriptor status child owns status mirror cleanup guard",
        &sources.descriptor_privacy_status_mirrors,
        &[
            REVIEW_GUARDS[6],
            "Runtime 15 F8 RuntimePluginDescriptor status mirror cleanup",
            "f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred",
        ],
    );
}
