use super::super::super::*;

#[test]
fn playback_settings_presets_match_bevy_playback_modes() {
    assert_eq!(
        SoundPlaybackSettings::default(),
        SoundPlaybackSettings::ONCE
    );
    assert!(!SoundPlaybackSettings::ONCE.looped);
    assert_eq!(
        SoundPlaybackSettings::ONCE.completion_action,
        SoundPlaybackCompletionAction::None
    );
    assert!(SoundPlaybackSettings::LOOP.looped);
    assert_eq!(
        SoundPlaybackSettings::LOOP.completion_action,
        SoundPlaybackCompletionAction::None
    );
    assert!(!SoundPlaybackSettings::DESPAWN.looped);
    assert_eq!(
        SoundPlaybackSettings::DESPAWN.completion_action,
        SoundPlaybackCompletionAction::DespawnEntity
    );
    assert!(!SoundPlaybackSettings::REMOVE.looped);
    assert_eq!(
        SoundPlaybackSettings::REMOVE.completion_action,
        SoundPlaybackCompletionAction::RemoveAudioComponents
    );

    let customized = SoundPlaybackSettings::LOOP
        .paused()
        .muted()
        .with_gain(0.5)
        .with_speed(2.0)
        .with_pan(-0.25)
        .with_start_seconds(0.1)
        .with_duration_seconds(0.2)
        .with_completion_action(SoundPlaybackCompletionAction::RemoveAudioComponents)
        .with_looped(false);
    assert!(customized.paused);
    assert!(customized.muted);
    assert_eq!(customized.gain, 0.5);
    assert_eq!(customized.speed, 2.0);
    assert_eq!(customized.pan, -0.25);
    assert_eq!(customized.start_seconds, Some(0.1));
    assert_eq!(customized.duration_seconds, Some(0.2));
    assert_eq!(
        customized.completion_action,
        SoundPlaybackCompletionAction::RemoveAudioComponents
    );
    assert!(!customized.looped);
}
