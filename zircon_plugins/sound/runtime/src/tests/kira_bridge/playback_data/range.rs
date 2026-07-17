use kira::sound::PlaybackPosition;
use zircon_runtime::core::framework::sound::SoundPlaybackSettings;

use crate::engine::LoadedClip;
use crate::kira_bridge::static_sound_data;
use crate::tests::test_clip_with_rate;

#[test]
fn non_loop_duration_is_encoded_as_kira_slice() {
    let loaded = LoadedClip::new(test_clip_with_rate(
        "res://sound/non-loop-slice.wav",
        10,
        &[0.0, 0.1, 0.2, 0.3, 0.4],
    ))
    .unwrap();
    let settings = SoundPlaybackSettings {
        start_seconds: Some(0.1),
        duration_seconds: Some(0.2),
        ..SoundPlaybackSettings::default()
    };

    let data = static_sound_data(&loaded.kira_data, &settings, 1, Some(3));

    assert_eq!(data.slice, Some((1, 3)));
    assert_eq!(data.settings.start_position, PlaybackPosition::Samples(0));
    assert_eq!(data.settings.loop_region, None);
    assert_eq!(data.num_frames(), 2);
}
