use super::super::*;

#[test]
fn global_volume_setting_updates_the_kira_runtime_contract_and_rejects_invalid_values() {
    let sound = DefaultSoundManager::default();

    assert_eq!(sound.global_volume_gain().unwrap(), 1.0);
    sound.set_global_volume_gain(0.25).unwrap();
    assert_eq!(sound.global_volume_gain().unwrap(), 0.25);

    assert!(sound.set_global_volume_gain(f32::NAN).is_err());
    assert!(sound.set_global_volume_gain(-0.1).is_err());
    assert_eq!(sound.global_volume_gain().unwrap(), 0.25);
}
