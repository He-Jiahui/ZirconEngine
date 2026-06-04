use super::super::super::*;
use super::super::support::test_hrtf_profile;

#[test]
fn hrtf_profile_validation_and_missing_remove_are_typed() {
    let sound = DefaultSoundManager::default();
    let mut invalid = test_hrtf_profile("bad");
    invalid.left_kernel = vec![f32::NAN];
    assert!(sound
        .load_hrtf_profile(invalid)
        .unwrap_err()
        .to_string()
        .contains("finite"));

    let mut silent = test_hrtf_profile("silent");
    silent.left_kernel = vec![0.0];
    silent.right_kernel = vec![0.0];
    assert!(sound
        .load_hrtf_profile(silent)
        .unwrap_err()
        .to_string()
        .contains("non-zero"));

    assert!(sound
        .remove_hrtf_profile("missing")
        .unwrap_err()
        .to_string()
        .contains("unknown HRTF profile"));
}
