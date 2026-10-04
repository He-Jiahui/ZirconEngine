use semver::{Version, VersionReq};

use super::{caret_range, classify_incompatible_requirement};
use crate::project::engine_compatibility::ProjectEngineCompatibilityDisposition;

fn classify(requirement: &str, running: &str) -> ProjectEngineCompatibilityDisposition {
    let requirement = VersionReq::parse(requirement).expect("test requirement must parse");
    let running = Version::parse(running).expect("test engine version must parse");
    classify_incompatible_requirement(&requirement, &running)
}

#[test]
fn caret_with_nonzero_major_has_a_next_major_upper_bound() {
    assert_eq!(
        classify("^1.2", "2.0.0"),
        ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine
    );
}

#[test]
fn caret_with_omitted_minor_still_has_a_next_major_upper_bound() {
    assert_eq!(
        classify("^1", "2.0.0"),
        ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine
    );
}

#[test]
fn caret_with_zero_major_and_nonzero_minor_has_a_next_minor_upper_bound() {
    assert_eq!(
        classify("^0.2", "0.3.0"),
        ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine
    );
}

#[test]
fn caret_with_zero_major_minor_and_patch_has_a_next_patch_upper_bound() {
    assert_eq!(
        classify("^0.0.3", "0.0.4"),
        ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine
    );
}

#[test]
fn caret_with_zero_major_minor_and_omitted_patch_has_a_next_minor_upper_bound() {
    assert_eq!(
        classify("^0.0", "0.1.0"),
        ProjectEngineCompatibilityDisposition::ProjectRequiresOlderEngine
    );
}

#[test]
fn caret_with_maximum_major_is_indeterminate_when_its_upper_bound_overflows() {
    let base = Version::new(u64::MAX, 2, 0);

    assert!(caret_range(&base, Some(2), None).is_none());
}

#[test]
fn caret_with_maximum_zero_major_minor_is_indeterminate_on_overflow() {
    let base = Version::new(0, u64::MAX, 0);

    assert!(caret_range(&base, Some(u64::MAX), None).is_none());
}

#[test]
fn caret_with_maximum_zero_major_patch_is_indeterminate_on_overflow() {
    let base = Version::new(0, 0, u64::MAX);

    assert!(caret_range(&base, Some(0), Some(u64::MAX)).is_none());
}
