// 经公共声管理器加载、排序列出并移除 HRTF 描述符，核对目录身份及剩余集合；不执行空间混音。
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
