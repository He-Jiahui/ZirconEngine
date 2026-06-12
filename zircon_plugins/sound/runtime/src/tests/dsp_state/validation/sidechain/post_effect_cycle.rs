use super::super::super::super::*;

use super::effect::sidechain_compressor;

#[test]
fn effect_update_rejects_post_effect_sidechain_cycle() {
    let sound = DefaultSoundManager::default();
    let key = SoundTrackId::new(2);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(key, "Key"))
        .unwrap();

    assert!(sound
        .add_or_update_effect(key, test_effect(sidechain_compressor(key, false)))
        .unwrap_err()
        .to_string()
        .contains("post-effect sidechain"));
}
