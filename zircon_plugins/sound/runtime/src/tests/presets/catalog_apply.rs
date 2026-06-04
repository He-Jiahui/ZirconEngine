use zircon_runtime::core::framework::sound::{SoundMixerGraphManager, SoundTrackId};

use super::super::DefaultSoundManager;

#[test]
fn built_in_mixer_presets_are_discoverable_and_apply() {
    let sound = DefaultSoundManager::default();

    let presets = sound.available_mixer_presets().unwrap();
    assert!(presets
        .iter()
        .any(|preset| preset.locator == "sound://mixer/default"));
    assert!(presets
        .iter()
        .any(|preset| preset.locator == "sound://mixer/music_sfx"));
    assert!(presets
        .iter()
        .any(|preset| preset.locator == "sound://mixer/spatial_room"));

    sound
        .apply_mixer_preset("sound://mixer/spatial_room")
        .unwrap();
    let snapshot = sound.mixer_snapshot().unwrap();

    assert!(snapshot
        .graph
        .tracks
        .iter()
        .any(|track| track.id == SoundTrackId::master() && track.display_name == "Master"));
    assert!(snapshot
        .graph
        .tracks
        .iter()
        .any(|track| track.id == SoundTrackId::new(5) && track.display_name == "Room Reverb"));
    let sfx = snapshot
        .graph
        .tracks
        .iter()
        .find(|track| track.id == SoundTrackId::new(3))
        .unwrap();
    assert!(sfx
        .sends
        .iter()
        .any(|send| send.target == SoundTrackId::new(5) && send.gain > 0.0));
}
