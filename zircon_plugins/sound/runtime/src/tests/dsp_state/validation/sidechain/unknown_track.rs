use super::super::super::super::*;

use super::effect::sidechain_compressor;

#[test]
fn effect_update_rejects_unknown_sidechain_track_reference() {
    let sound = DefaultSoundManager::default();

    assert!(matches!(
        sound
            .add_or_update_effect(
                SoundTrackId::master(),
                test_effect(sidechain_compressor(SoundTrackId::new(999), true)),
            )
            .unwrap_err(),
        SoundError::UnknownTrack { .. }
    ));
}
