use super::super::*;

#[path = "f8_child_owners/budgets.rs"]
mod budgets;
#[path = "f8_child_owners/delegation.rs"]
mod delegation;
#[path = "f8_child_owners/route_ownership.rs"]
mod route_ownership;
#[path = "f8_child_owners/status_mirrors.rs"]
mod status_mirrors;

pub(super) const STRUCTURE_GUARD_OWNER: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners.rs";
pub(super) const SLICE: &str = "Runtime 15 M3 F8 API convergence review guard child-owner split";
pub(super) const STATUS: &str =
    "runtime_15_f8_api_convergence_review_guard_child_owner_split_static_passed_cargo_deferred";
pub(super) const DATE: &str = "2026-06-30";
pub(super) const GUARD: &str = "runtime_15_f8_api_convergence_review_guards_are_child_owners";
pub(super) const FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 F8 child-owner structure guard folder-backed split";
pub(super) const FOLDER_BACKED_STATUS: &str =
    "runtime_15_f8_child_owner_structure_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const FOLDER_BACKED_DATE: &str = "2026-07-03";
pub(super) const FOLDER_BACKED_GUARD: &str =
    "runtime_15_f8_child_owner_structure_guard_is_folder_backed";
pub(super) const FOLDER_BACKED_STATUS_GUARD: &str =
    "runtime_15_f8_child_owner_structure_guard_folder_backed_status_is_current";
pub(super) const BUDGET_GUARD: &str =
    "runtime_15_f8_child_owner_structure_guard_budgets_are_focused";

pub(super) const PARENT: &str =
    "tests/runtime_absorption/code_review_findings/f8_api_convergence.rs";
pub(super) const TEXTURE_IMPORT_SETTINGS: &str =
    "tests/runtime_absorption/code_review_findings/f8_api_convergence/texture_import_settings.rs";
pub(super) const DESCRIPTOR_BUILDER: &str =
    "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder.rs";
pub(super) const DESCRIPTOR_BUILDER_SCAFFOLD: &str = "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/scaffold.rs";
pub(super) const DESCRIPTOR_BUILDER_FIRST_PARTY: &str = "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/first_party_descriptors.rs";
pub(super) const DESCRIPTOR_BUILDER_TEST_FIXTURES: &str = "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/test_fixtures.rs";
pub(super) const DESCRIPTOR_PRIVACY: &str =
    "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy.rs";
pub(super) const DESCRIPTOR_PRIVACY_PRIVATE_FIELDS: &str = "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/private_fields.rs";
pub(super) const DESCRIPTOR_PRIVACY_CONSTRUCTOR_RETIREMENT: &str = "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/constructor_retirement.rs";
pub(super) const DESCRIPTOR_PRIVACY_STATUS_MIRRORS: &str = "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/status_mirrors.rs";
pub(super) const STRUCTURE_GUARD_ROWS: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows/root_and_children/f8_child_owner.rs";
pub(super) const REVIEW_GUARD_STATUS_MAP: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs";
pub(super) const REVIEW_GUARD_DATE_MAP: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs";

pub(super) const REVIEW_GUARDS: &[&str] = &[
    "review_f8_texture_import_settings_use_fallible_apply_not_with",
    "review_f8_runtime_plugin_descriptor_exposes_builder_scaffold",
    "review_f8_first_party_runtime_plugin_descriptors_use_builder",
    "review_f8_runtime_plugin_descriptor_test_fixtures_use_builder",
    "review_f8_runtime_plugin_descriptor_fields_are_private_with_accessors",
    "review_f8_runtime_plugin_descriptor_public_constructor_is_retired",
    "review_f8_runtime_plugin_descriptor_status_mirrors_do_not_claim_public_field_pending",
];

pub(super) const FOLDER_BACKED_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/delegation.rs",
        FOLDER_BACKED_GUARD,
    ),
    (
        "route_ownership",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/route_ownership.rs",
        GUARD,
    ),
    (
        "status_mirrors",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/status_mirrors.rs",
        FOLDER_BACKED_STATUS_GUARD,
    ),
    (
        "budgets",
        "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_child_owners/budgets.rs",
        BUDGET_GUARD,
    ),
];

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
