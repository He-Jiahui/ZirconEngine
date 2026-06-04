use super::super::super::*;
use super::super::support::test_hrtf_profile;

#[test]
fn hrtf_profiles_can_be_loaded_listed_and_removed() {
    let sound = DefaultSoundManager::default();
    sound
        .load_hrtf_profile(test_hrtf_profile("profile.b"))
        .unwrap();
    sound
        .load_hrtf_profile(test_hrtf_profile("profile.a"))
        .unwrap();

    let profiles = sound.hrtf_profiles().unwrap();
    assert_eq!(profiles.len(), 2);
    assert_eq!(profiles[0].profile_id, "profile.a");
    assert_eq!(profiles[1].profile_id, "profile.b");

    sound.remove_hrtf_profile("profile.a").unwrap();
    let profiles = sound.hrtf_profiles().unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].profile_id, "profile.b");
}
