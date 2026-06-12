use super::super::super::super::*;

use super::effect::sidechain_compressor;

#[test]
fn effect_update_accepts_pre_effect_sidechain_track_reference() {
    let sound = DefaultSoundManager::default();
    let key = SoundTrackId::new(2);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(key, "Key"))
        .unwrap();

    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            test_effect(sidechain_compressor(key, true)),
        )
        .unwrap();
}
