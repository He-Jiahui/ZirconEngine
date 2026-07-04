use super::super::*;
use super::*;

pub(super) fn assert_f8_review_leaf_owners_are_child_owned(sources: &F8ReviewSources) {
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
