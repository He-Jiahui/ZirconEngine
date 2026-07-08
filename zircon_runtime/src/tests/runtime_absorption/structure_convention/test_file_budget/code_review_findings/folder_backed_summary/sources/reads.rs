use super::super::super::super::*;
use super::model::CodeReviewFindingsSources;

pub(super) fn code_review_findings_sources() -> CodeReviewFindingsSources {
    CodeReviewFindingsSources {
        parent: read_runtime_src("tests/runtime_absorption/code_review_findings.rs"),
        f8_api_convergence: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/f8_api_convergence.rs",
        ),
        f8_texture_import_settings: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/texture_import_settings.rs",
        ),
        f8_descriptor_builder: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder.rs",
        ),
        f8_descriptor_builder_scaffold: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/scaffold.rs",
        ),
        f8_descriptor_builder_first_party: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/first_party_descriptors.rs",
        ),
        f8_descriptor_builder_test_fixtures: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/test_fixtures.rs",
        ),
        f8_descriptor_privacy: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy.rs",
        ),
        f8_descriptor_privacy_private_fields: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/private_fields.rs",
        ),
        f8_descriptor_privacy_constructor_retirement: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/constructor_retirement.rs",
        ),
        f8_descriptor_privacy_status_mirrors: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_privacy/status_mirrors.rs",
        ),
        late_api_cleanup: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/late_api_cleanup.rs",
        ),
        p0_robustness: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/p0_robustness.rs",
        ),
        p0_native_host_callbacks: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_host_callbacks.rs",
        ),
        p0_lock_poison: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/p0_robustness/lock_poison.rs",
        ),
        p0_render_submit: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/p0_robustness/render_submit.rs",
        ),
        p0_native_fixture: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture.rs",
        ),
        p0_native_fixture_sdk_macro: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/sdk_macro_manifest.rs",
        ),
        p0_native_fixture_importer: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/importer_manifest.rs",
        ),
        p0_priority_recommendation: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/p0_robustness/priority_recommendation.rs",
        ),
        render_structure: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/render_structure.rs",
        ),
        f12_dead_code: read_runtime_src(
            "tests/runtime_absorption/code_review_findings/f12_dead_code.rs",
        ),
    }
}
