use super::super::super::super::*;
use super::support::reconfigure_preview_output;

#[test]
fn reconfigure_updates_runtime_and_mixer_format() {
    let sound = DefaultSoundManager::default();
    reconfigure_preview_output(&sound);

    assert_eq!(sound.backend_status().sample_rate_hz, 24_000);
    assert_eq!(sound.backend_status().channel_count, 1);
    assert_eq!(
        sound.backend_status().channel_layout,
        AudioChannelLayout::mono()
    );
    let snapshot = sound.mixer_snapshot().unwrap();
    assert_eq!(snapshot.graph.sample_rate_hz, 24_000);
    assert_eq!(snapshot.graph.channel_count, 1);
    assert_eq!(snapshot.graph.channel_layout, AudioChannelLayout::mono());
}
