#[derive(Debug)]
pub(in super::super) struct CodeReviewFindingsSources {
    pub(in super::super) parent: String,
    pub(in super::super) f8_api_convergence: String,
    pub(in super::super) f8_texture_import_settings: String,
    pub(in super::super) f8_descriptor_builder: String,
    pub(in super::super) f8_descriptor_builder_scaffold: String,
    pub(in super::super) f8_descriptor_builder_first_party: String,
    pub(in super::super) f8_descriptor_builder_test_fixtures: String,
    pub(in super::super) f8_descriptor_privacy: String,
    pub(in super::super) f8_descriptor_privacy_private_fields: String,
    pub(in super::super) f8_descriptor_privacy_constructor_retirement: String,
    pub(in super::super) f8_descriptor_privacy_status_mirrors: String,
    pub(in super::super) late_api_cleanup: String,
    pub(in super::super) p0_robustness: String,
    pub(in super::super) p0_native_host_callbacks: String,
    pub(in super::super) p0_lock_poison: String,
    pub(in super::super) p0_render_submit: String,
    pub(in super::super) p0_native_fixture: String,
    pub(in super::super) p0_native_fixture_sdk_macro: String,
    pub(in super::super) p0_native_fixture_importer: String,
    pub(in super::super) p0_priority_recommendation: String,
    pub(in super::super) render_structure: String,
    pub(in super::super) f12_dead_code: String,
}

impl CodeReviewFindingsSources {
    pub(in super::super) fn direct_review_guard_count(&self) -> usize {
        [
            self.f8_api_convergence.as_str(),
            self.f8_texture_import_settings.as_str(),
            self.f8_descriptor_builder.as_str(),
            self.f8_descriptor_builder_scaffold.as_str(),
            self.f8_descriptor_builder_first_party.as_str(),
            self.f8_descriptor_builder_test_fixtures.as_str(),
            self.f8_descriptor_privacy.as_str(),
            self.f8_descriptor_privacy_private_fields.as_str(),
            self.f8_descriptor_privacy_constructor_retirement.as_str(),
            self.f8_descriptor_privacy_status_mirrors.as_str(),
            self.p0_robustness.as_str(),
            self.p0_native_host_callbacks.as_str(),
            self.p0_lock_poison.as_str(),
            self.p0_render_submit.as_str(),
            self.p0_native_fixture.as_str(),
            self.p0_native_fixture_sdk_macro.as_str(),
            self.p0_native_fixture_importer.as_str(),
            self.p0_priority_recommendation.as_str(),
            self.render_structure.as_str(),
            self.f12_dead_code.as_str(),
        ]
        .into_iter()
        .map(|source| source.matches("#[test]").count())
        .sum()
    }
}
