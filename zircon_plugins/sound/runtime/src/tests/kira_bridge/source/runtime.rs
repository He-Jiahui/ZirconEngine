use std::collections::HashMap;

use kira::{backend::mock::MockBackend, sound::PlaybackState};
use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundClipId, SoundParameterId, SoundSourceDescriptor,
    SoundSourceInput,
};

use crate::engine::SourceVoice;
use crate::kira_bridge::KiraEngine;
use crate::service_types::{
    mute_bound_source, pause_bound_source, resume_bound_source, seek_bound_source,
    set_bound_source_gain, set_bound_source_speed, stop_bound_source, sync_source_voice,
};

use super::support::MockSourceRuntime;

#[test]
fn inactive_clip_source_preconfigures_then_binds_after_kira_activation() {
    let mut runtime = MockSourceRuntime::inactive_clip();

    sync_source_voice(
        &mut runtime.engine,
        &mut runtime.next_playback_id,
        &runtime.clips,
        &mut runtime.voice,
    )
    .unwrap();
    assert_eq!(runtime.voice.kira_playback, None);
    assert!(runtime.voice.descriptor.playing);

    runtime.activate();
    sync_source_voice(
        &mut runtime.engine,
        &mut runtime.next_playback_id,
        &runtime.clips,
        &mut runtime.voice,
    )
    .unwrap();

    let playback = runtime.voice.kira_playback.unwrap();
    assert!(runtime.engine.contains_playback(playback));
}

#[test]
fn clip_source_controls_drive_the_bound_kira_handle() {
    let mut runtime = MockSourceRuntime::inactive_clip();
    runtime.activate();
    sync_source_voice(
        &mut runtime.engine,
        &mut runtime.next_playback_id,
        &runtime.clips,
        &mut runtime.voice,
    )
    .unwrap();
    let playback = runtime.voice.kira_playback.unwrap();

    pause_bound_source(&mut runtime.engine, &mut runtime.voice).unwrap();
    runtime
        .engine
        .with_backend_mut(|backend| {
            backend.on_start_processing();
            backend.process();
        })
        .unwrap();
    assert_eq!(
        runtime.engine.playback_state(playback).unwrap(),
        PlaybackState::Paused
    );
    resume_bound_source(&mut runtime.engine, &mut runtime.voice).unwrap();
    set_bound_source_gain(&mut runtime.engine, &mut runtime.voice, 0.25).unwrap();
    set_bound_source_speed(&mut runtime.engine, &mut runtime.voice, 1.5).unwrap();
    mute_bound_source(&mut runtime.engine, &mut runtime.voice, true).unwrap();
    seek_bound_source(&mut runtime.engine, &mut runtime.voice, 0.2, 2).unwrap();

    assert!(runtime.engine.contains_playback(playback));
    assert!(runtime.voice.descriptor.playing);
    assert_eq!(runtime.voice.descriptor.gain, 0.25);
    assert_eq!(runtime.voice.descriptor.speed, 1.5);
    assert!(runtime.voice.descriptor.muted);
    assert_eq!(runtime.voice.cursor_frame, 2);

    stop_bound_source(&mut runtime.engine, &mut runtime.voice).unwrap();
    assert!(!runtime.engine.contains_playback(playback));
    assert_eq!(runtime.voice.kira_playback, None);
}

#[test]
fn unsupported_external_and_synth_sources_return_typed_errors() {
    let mut engine = KiraEngine::<MockBackend>::inactive();
    let mut next_playback_id = 0;
    let clips = HashMap::new();
    let mut external = SourceVoice::new(SoundSourceDescriptor {
        input: SoundSourceInput::External(ExternalAudioSourceHandle::new("external.unsupported")),
        ..SoundSourceDescriptor::clip(SoundClipId::new(1))
    });
    let mut synth = SourceVoice::new(SoundSourceDescriptor {
        input: SoundSourceInput::SynthParameter {
            parameter: SoundParameterId::new("synth.unsupported"),
            default_value: 0.0,
        },
        ..SoundSourceDescriptor::clip(SoundClipId::new(1))
    });

    let external_error =
        sync_source_voice(&mut engine, &mut next_playback_id, &clips, &mut external).unwrap_err();
    let synth_error =
        sync_source_voice(&mut engine, &mut next_playback_id, &clips, &mut synth).unwrap_err();

    assert!(external_error.to_string().contains("external source"));
    assert!(synth_error.to_string().contains("synth source"));
    assert_eq!(external.kira_playback, None);
    assert_eq!(synth.kira_playback, None);
}
