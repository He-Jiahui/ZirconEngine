use super::*;

pub(super) struct F8ReviewSources {
    pub(super) parent: String,
    pub(super) texture_import_settings: String,
    pub(super) descriptor_builder: String,
    pub(super) descriptor_builder_scaffold: String,
    pub(super) descriptor_builder_first_party: String,
    pub(super) descriptor_builder_test_fixtures: String,
    pub(super) descriptor_privacy: String,
    pub(super) descriptor_privacy_private_fields: String,
    pub(super) descriptor_privacy_constructor_retirement: String,
    pub(super) descriptor_privacy_status_mirrors: String,
}

impl F8ReviewSources {
    pub(super) fn all_sources(&self) -> [(&'static str, &str); 10] {
        [
            (PARENT, self.parent.as_str()),
            (
                TEXTURE_IMPORT_SETTINGS,
                self.texture_import_settings.as_str(),
            ),
            (DESCRIPTOR_BUILDER, self.descriptor_builder.as_str()),
            (
                DESCRIPTOR_BUILDER_SCAFFOLD,
                self.descriptor_builder_scaffold.as_str(),
            ),
            (
                DESCRIPTOR_BUILDER_FIRST_PARTY,
                self.descriptor_builder_first_party.as_str(),
            ),
            (
                DESCRIPTOR_BUILDER_TEST_FIXTURES,
                self.descriptor_builder_test_fixtures.as_str(),
            ),
            (DESCRIPTOR_PRIVACY, self.descriptor_privacy.as_str()),
            (
                DESCRIPTOR_PRIVACY_PRIVATE_FIELDS,
                self.descriptor_privacy_private_fields.as_str(),
            ),
            (
                DESCRIPTOR_PRIVACY_CONSTRUCTOR_RETIREMENT,
                self.descriptor_privacy_constructor_retirement.as_str(),
            ),
            (
                DESCRIPTOR_PRIVACY_STATUS_MIRRORS,
                self.descriptor_privacy_status_mirrors.as_str(),
            ),
        ]
    }
}

pub(super) fn read_f8_review_sources() -> F8ReviewSources {
    F8ReviewSources {
        parent: read_runtime_src(PARENT),
        texture_import_settings: read_runtime_src(TEXTURE_IMPORT_SETTINGS),
        descriptor_builder: read_runtime_src(DESCRIPTOR_BUILDER),
        descriptor_builder_scaffold: read_runtime_src(DESCRIPTOR_BUILDER_SCAFFOLD),
        descriptor_builder_first_party: read_runtime_src(DESCRIPTOR_BUILDER_FIRST_PARTY),
        descriptor_builder_test_fixtures: read_runtime_src(DESCRIPTOR_BUILDER_TEST_FIXTURES),
        descriptor_privacy: read_runtime_src(DESCRIPTOR_PRIVACY),
        descriptor_privacy_private_fields: read_runtime_src(DESCRIPTOR_PRIVACY_PRIVATE_FIELDS),
        descriptor_privacy_constructor_retirement: read_runtime_src(
            DESCRIPTOR_PRIVACY_CONSTRUCTOR_RETIREMENT,
        ),
        descriptor_privacy_status_mirrors: read_runtime_src(DESCRIPTOR_PRIVACY_STATUS_MIRRORS),
    }
}

pub(super) fn folder_backed_child_sources() -> Vec<(&'static str, String)> {
    FOLDER_BACKED_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn folder_backed_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, child_source) in folder_backed_child_sources() {
        blob.push_str(&child_source);
        blob.push('\n');
    }
    blob
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn f8_structure_guard_child_source_blob(
) -> String {
    let mut blob = folder_backed_child_source_blob();
    blob.push('\n');
    let review_sources = read_f8_review_sources();
    for (path, source) in review_sources.all_sources() {
        blob.push_str(path);
        blob.push('\n');
        blob.push_str(source);
        blob.push('\n');
    }
    blob
}
