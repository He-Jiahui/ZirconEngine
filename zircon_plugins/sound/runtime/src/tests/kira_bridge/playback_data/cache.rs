use std::sync::Arc;

use zircon_runtime::core::framework::sound::SoundPlaybackSettings;

use crate::engine::LoadedClip;
use crate::kira_bridge::static_sound_data;
use crate::tests::test_clip_with_rate;

#[test]
fn loaded_clip_reuses_arc_frames_across_playback_data() {
    let loaded = LoadedClip::new(test_clip_with_rate(
        "res://sound/cached-kira-data.wav",
        10,
        &[0.1, 0.2, 0.3, 0.4],
    ))
    .unwrap();

    let first = static_sound_data(
        &loaded.kira_data,
        &SoundPlaybackSettings::default(),
        0,
        None,
    );
    let second = static_sound_data(
        &loaded.kira_data,
        &SoundPlaybackSettings::default(),
        0,
        None,
    );

    assert!(Arc::ptr_eq(&loaded.kira_data.frames, &first.frames));
    assert!(Arc::ptr_eq(&first.frames, &second.frames));
}
