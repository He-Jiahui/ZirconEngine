use super::*;

#[test]
fn runtime_defaults_bound_every_job_category() {
    let limits = EditorJobLimits::resolved(4, []);

    for category in JobCategory::ALL {
        assert!(
            limits.limit(category) < usize::MAX,
            "{category:?} must not bypass admission with an unbounded default"
        );
    }
}

#[test]
fn interactive_save_has_an_explicit_finite_default() {
    let limits = EditorJobLimits::resolved(4, []);

    assert_eq!(
        limits.limit(JobCategory::InteractiveSave),
        DEFAULT_INTERACTIVE_SAVE_LIMIT
    );
}

#[test]
fn play_default_does_not_alias_the_export_default_path() {
    assert_eq!(
        user_configurable_default_limit(JobCategory::Play),
        Some(DEFAULT_PLAY_LIMIT)
    );
    assert_eq!(
        user_configurable_default_limit(JobCategory::Export),
        Some(DEFAULT_EXPORT_LIMIT)
    );
}
