use super::super::super::super::*;

use super::super::source_inventory::CodeReviewFindingsSources;

const DIRECT_REVIEW_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
const F8_DIRECT_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8.rs";
const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

pub(super) fn assert_f8_direct_sources_are_folder_backed(sources: &CodeReviewFindingsSources) {
    assert_contains_all(
        "F8 API convergence parent only mounts focused child review guard owners",
        &sources.f8_api_convergence,
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
        sources.f8_api_convergence.matches("#[test]").count(),
        0,
        "f8_api_convergence.rs should only mount child review guard owners"
    );
    for child_owned_test in [
        "fn review_f8_texture_import_settings_use_fallible_apply_not_with",
        "fn review_f8_runtime_plugin_descriptor_exposes_builder_scaffold",
        "fn review_f8_first_party_runtime_plugin_descriptors_use_builder",
        "fn review_f8_runtime_plugin_descriptor_test_fixtures_use_builder",
        "fn review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors",
        "fn review_f8_runtime_plugin_descriptor_public_constructor_is_retired",
        "fn review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending",
    ] {
        assert!(
            !sources.f8_api_convergence.contains(child_owned_test),
            "child-owned F8 review guard `{child_owned_test}` should not return to f8_api_convergence.rs"
        );
    }
    assert_contains_all(
        "F8 texture import settings child owns texture apply review guard",
        &sources.f8_texture_import_settings,
        &[
            "fn review_f8_texture_import_settings_use_fallible_apply_not_with",
            "apply_import_settings",
            "TextureDescriptorError",
            "runtime_15_texture_descriptor_typed_errors_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "F8 descriptor builder child mounts builder and fixture migration leaf owners",
        &sources.f8_descriptor_builder,
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
        sources.f8_descriptor_builder.matches("#[test]").count(),
        0,
        "descriptor_builder.rs should only mount builder review guard owners"
    );
    assert_contains_all(
        "F8 descriptor builder leaf owners keep builder migration review guards",
        &[
            sources.f8_descriptor_builder_scaffold.as_str(),
            sources.f8_descriptor_builder_first_party.as_str(),
            sources.f8_descriptor_builder_test_fixtures.as_str(),
        ]
        .join("\n"),
        &[
            "fn review_f8_runtime_plugin_descriptor_exposes_builder_scaffold",
            "fn review_f8_first_party_runtime_plugin_descriptors_use_builder",
            "fn review_f8_runtime_plugin_descriptor_test_fixtures_use_builder",
            "RuntimePluginDescriptorBuilder",
            "RuntimePluginDescriptor::builder(",
        ],
    );
    assert_contains_all(
        "F8 descriptor privacy child mounts private-field and constructor leaf owners",
        &sources.f8_descriptor_privacy,
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
        sources.f8_descriptor_privacy.matches("#[test]").count(),
        0,
        "descriptor_privacy.rs should only mount privacy review guard owners"
    );
    assert_contains_all(
        "F8 descriptor privacy leaf owners keep private-field and constructor review guards",
        &[
            sources.f8_descriptor_privacy_private_fields.as_str(),
            sources.f8_descriptor_privacy_constructor_retirement.as_str(),
            sources.f8_descriptor_privacy_status_mirrors.as_str(),
        ]
        .join("\n"),
        &[
            "fn review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors",
            "fn review_f8_runtime_plugin_descriptor_public_constructor_is_retired",
            "fn review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending",
            "RuntimePluginDescriptor private fields 15/15",
            "RuntimePluginDescriptor::new retired",
        ],
    );
}

#[test]
fn runtime_15_code_review_findings_f8_direct_assertions_are_child_owner() {
    let parent = read_runtime_src(DIRECT_REVIEW_ASSERTIONS_CHILD);
    let child = read_runtime_src(F8_DIRECT_ASSERTIONS_CHILD);
    let sources = super::super::source_inventory::code_review_findings_sources();

    assert_contains_all(
        "direct-review assertion child delegates F8 assertions to child owner",
        &parent,
        &[
            "#[path = \"direct_review_assertions/f8.rs\"]",
            "mod f8;",
            "f8::assert_f8_direct_sources_are_folder_backed",
        ],
    );
    for moved_guard in [
        concat!(
            "F8 API convergence parent only mounts focused child ",
            "review guard owners"
        ),
        concat!(
            "F8 descriptor privacy leaf owners keep private-field ",
            "and constructor review guards"
        ),
        concat!(
            "review_f8_runtime_plugin_descriptor_public_",
            "constructor_is_retired"
        ),
    ] {
        assert!(
            !parent.contains(moved_guard),
            "F8 direct assertion `{moved_guard}` should stay in {F8_DIRECT_ASSERTIONS_CHILD}"
        );
    }
    assert_contains_all(
        "F8 direct assertion child owns F8 source checks",
        &child,
        &[
            "pub(super) fn assert_f8_direct_sources_are_folder_backed",
            "F8 API convergence parent only mounts focused child review guard owners",
            "F8 texture import settings child owns texture apply review guard",
            "F8 descriptor builder child mounts builder and fixture migration leaf owners",
            "F8 descriptor builder leaf owners keep builder migration review guards",
            "F8 descriptor privacy child mounts private-field and constructor leaf owners",
            "F8 descriptor privacy leaf owners keep private-field and constructor review guards",
            "review_f8_runtime_plugin_descriptor_public_constructor_is_retired",
        ],
    );

    assert_f8_direct_sources_are_folder_backed(&sources);

    for (path, source) in [
        (DIRECT_REVIEW_ASSERTIONS_CHILD, parent.as_str()),
        (F8_DIRECT_ASSERTIONS_CHILD, child.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < CODE_REVIEW_FINDINGS_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
