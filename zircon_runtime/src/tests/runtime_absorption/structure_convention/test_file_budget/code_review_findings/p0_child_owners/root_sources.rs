use super::*;

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings)
struct P0RobustnessSources
{
    pub(super) parent: String,
    pub(super) native_host_callbacks: String,
    pub(super) lock_poison: String,
    pub(super) render_submit: String,
    pub(super) native_fixture: String,
    pub(super) native_fixture_sdk_macro: String,
    pub(super) native_fixture_importer: String,
    pub(super) priority_recommendation: String,
}

impl P0RobustnessSources {
    pub(super) fn all_sources(&self) -> [(&'static str, &str); 8] {
        [
            (PARENT, self.parent.as_str()),
            (NATIVE_HOST_CALLBACKS, self.native_host_callbacks.as_str()),
            (LOCK_POISON, self.lock_poison.as_str()),
            (RENDER_SUBMIT, self.render_submit.as_str()),
            (NATIVE_FIXTURE, self.native_fixture.as_str()),
            (
                NATIVE_FIXTURE_SDK_MACRO,
                self.native_fixture_sdk_macro.as_str(),
            ),
            (
                NATIVE_FIXTURE_IMPORTER,
                self.native_fixture_importer.as_str(),
            ),
            (
                PRIORITY_RECOMMENDATION,
                self.priority_recommendation.as_str(),
            ),
        ]
    }
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn read_p0_robustness_sources(
) -> P0RobustnessSources {
    P0RobustnessSources {
        parent: read_runtime_src(PARENT),
        native_host_callbacks: read_runtime_src(NATIVE_HOST_CALLBACKS),
        lock_poison: read_runtime_src(LOCK_POISON),
        render_submit: read_runtime_src(RENDER_SUBMIT),
        native_fixture: read_runtime_src(NATIVE_FIXTURE),
        native_fixture_sdk_macro: read_runtime_src(NATIVE_FIXTURE_SDK_MACRO),
        native_fixture_importer: read_runtime_src(NATIVE_FIXTURE_IMPORTER),
        priority_recommendation: read_runtime_src(PRIORITY_RECOMMENDATION),
    }
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn folder_backed_child_sources(
) -> Vec<(&'static str, String)> {
    FOLDER_BACKED_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn folder_backed_child_source_blob(
) -> String {
    let mut blob = String::new();
    for (_, child_source) in folder_backed_child_sources() {
        blob.push_str(&child_source);
        blob.push('\n');
    }
    blob
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn p0_robustness_review_guard_count(
) -> usize {
    let review_sources = read_p0_robustness_sources();
    review_sources
        .all_sources()
        .iter()
        .map(|(_, source)| source.matches("#[test]").count())
        .sum()
}

pub(in crate::tests::runtime_absorption::structure_convention::test_file_budget::code_review_findings) fn p0_robustness_status_row_source(
) -> String {
    format!(
        "{}\n{}",
        read_runtime_src(STRUCTURE_GUARD_ROW_PARENT),
        read_runtime_src(STRUCTURE_GUARD_ROWS),
    )
}
