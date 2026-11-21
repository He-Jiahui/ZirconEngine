//! Drive the real Kira mixer without an OS audio device; read the production SoundManager API.
use kira::{sound::PlaybackState, AudioManagerSettings};
use zircon_runtime::asset::{AssetUri, SoundAsset};
use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundError, SoundManager, SoundPlaybackId, SoundPlaybackSettings,
};

use super::DefaultSoundManager;
use crate::kira_bridge::OwnerCpalBackend;
use crate::poison_recovery::lock_recover;
use crate::SoundConfig;

const SAMPLE_RATE: u32 = 8_000;
const BLOCK_FRAMES: usize = 64;
const START_FRAME: usize = 4_000;
const END_FRAME: usize = 20_000;
const SEEK_FRAME: usize = 12_000;
const SEEK_SECONDS: f32 = 1.5;

#[test]
fn paused_clip_seek_status_reports_target_without_resume() {
    let (manager, clip, id) = playing_slice();
    let sound: &dyn SoundManager = &manager;
    process_blocks(&manager, 8);
    let playing = sound.playback_status(id).unwrap();
    assert!(!playing.paused);
    assert!((START_FRAME + 1..SEEK_FRAME).contains(&playing.cursor_frame));
    sound.pause_playback(id).unwrap();
    process_blocks(&manager, 8);
    assert_eq!(backend_state(&manager, id), PlaybackState::Paused);
    let frozen = sound.playback_status(id).unwrap();
    assert!(frozen.paused && frozen.cursor_frame >= playing.cursor_frame);
    assert!(frozen.cursor_frame < SEEK_FRAME);
    assert_eq!(frozen.range_start_frame, START_FRAME);
    assert_eq!(frozen.range_end_frame, Some(END_FRAME));
    process_blocks(&manager, 2);
    assert_eq!(sound.playback_status(id).unwrap(), frozen);

    sound.seek_playback_seconds(id, SEEK_SECONDS).unwrap();
    process_blocks(&manager, 3);
    assert_eq!(backend_state(&manager, id), PlaybackState::Paused);
    let first = sound.playback_status(id).unwrap();
    process_blocks(&manager, 3);
    assert_eq!(backend_state(&manager, id), PlaybackState::Paused);
    let second = sound.playback_status(id).unwrap();
    assert!(first.paused && second.paused);
    assert_eq!((first.playback, second.playback), (id, id));
    assert_eq!((first.clip, second.clip), (clip, clip));
    assert!(!sound.playback_empty(id).unwrap());

    // No second seek: after resume, the real mixer must progress from the new transport region.
    // Read this even on the expected RED, so the failure is specifically paused readback.
    sound.resume_playback(id).unwrap();
    process_blocks(&manager, 8);
    assert_eq!(backend_state(&manager, id), PlaybackState::Playing);
    let resumed = sound.playback_status(id).unwrap();
    assert!(!resumed.paused);
    assert!((SEEK_FRAME + 1..SEEK_FRAME + 1_000).contains(&resumed.cursor_frame));
    process_blocks(&manager, 2);
    assert!(sound.playback_status(id).unwrap().cursor_frame > resumed.cursor_frame);
    sound.stop_playback(id).unwrap();

    assert_eq!(
        ((first.cursor_frame, first.cursor_seconds), (second.cursor_frame, second.cursor_seconds)),
        ((SEEK_FRAME, SEEK_SECONDS), (SEEK_FRAME, SEEK_SECONDS)),
        "successful seek after completed mixer callbacks must change the paused cursor; frozen frame={}, resumed frame={}",
        frozen.cursor_frame,
        resumed.cursor_frame,
    );
}

// Port of the freeze/continue invariants from Harmonix RuntimeMusicSource.TransportStates.
// ZE owns a clip handle, not a beat clock: stop retires the ID instead of resetting its time.
#[test]
fn transport_states_freeze_then_continue_without_a_seek() {
    let (manager, _, id) = playing_slice();
    let sound: &dyn SoundManager = &manager;
    process_blocks(&manager, 8);
    sound.pause_playback(id).unwrap();
    process_blocks(&manager, 8);
    let frozen = sound.playback_status(id).unwrap();
    assert!(frozen.paused && frozen.cursor_frame > START_FRAME);
    sound.pause_playback(id).unwrap();
    process_blocks(&manager, 4);
    assert_eq!(sound.playback_status(id).unwrap(), frozen);
    sound.resume_playback(id).unwrap();
    process_blocks(&manager, 8);
    let resumed = sound.playback_status(id).unwrap();
    assert!(!resumed.paused && resumed.cursor_frame > frozen.cursor_frame);
    sound.stop_playback(id).unwrap();
    assert!(matches!(
        sound.playback_status(id),
        Err(SoundError::UnknownPlayback { .. })
    ));
    assert!(sound.playback_empty(id).unwrap());
}

#[test]
fn latest_paused_seek_survives_rejection_and_the_first_resume_callback() {
    // Opposite directions: accepting a numerically larger cursor is not proof of freshness.
    for (seconds, target, toggle) in [(1.5, SEEK_FRAME, false), (0.0, START_FRAME, true)] {
        let (manager, _, id) = playing_slice();
        let sound: &dyn SoundManager = &manager;
        process_blocks(&manager, 8);
        sound.pause_playback(id).unwrap();
        process_blocks(&manager, 8);
        assert_eq!(backend_state(&manager, id), PlaybackState::Paused);
        let old_sample = sound.playback_status(id).unwrap().cursor_frame;
        assert_ne!(old_sample, target);
        sound.seek_playback_seconds(id, 1.0).unwrap();
        sound.seek_playback_seconds(id, seconds).unwrap();
        let accepted = sound.playback_status(id).unwrap();
        assert!(accepted.paused);
        assert_eq!(accepted.cursor_frame, target);
        assert_eq!(accepted.cursor_seconds, target as f32 / SAMPLE_RATE as f32);
        for invalid in [f32::NAN, -0.5, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(matches!(
                sound.seek_playback_seconds(id, invalid),
                Err(SoundError::InvalidParameter(_))
            ));
            assert_eq!(sound.playback_status(id).unwrap(), accepted);
        }
        process_blocks(&manager, 2);
        assert_eq!(sound.playback_status(id).unwrap(), accepted);
        if toggle {
            sound.toggle_playback(id).unwrap();
        } else {
            sound.resume_playback(id).unwrap();
        }
        for blocks in [0, 1] {
            process_blocks(&manager, blocks);
            let status = sound.playback_status(id).unwrap();
            assert!(!status.paused);
            assert_eq!(
                (status.cursor_frame, status.cursor_seconds),
                (accepted.cursor_frame, accepted.cursor_seconds)
            );
            // Verify that this really is stale feedback from the concrete Kira handle.
            let raw = lock_recover(&manager.state)
                .kira
                .playback_position(id)
                .unwrap();
            assert_eq!(
                START_FRAME + (raw * SAMPLE_RATE as f64).round() as usize,
                old_sample
            );
        }
        process_blocks(&manager, 1);
        let published = sound.playback_status(id).unwrap().cursor_frame;
        assert!((target + 1..=target + BLOCK_FRAMES).contains(&published));
        process_blocks(&manager, 2);
        assert!(sound.playback_status(id).unwrap().cursor_frame > published);
    }
}

#[test]
fn feedback_for_one_resume_does_not_release_a_later_handle() {
    let (manager, clip, first) = playing_slice();
    let sound: &dyn SoundManager = &manager;
    let later = sound.play_clip(clip, SoundPlaybackSettings::ONCE).unwrap();
    process_blocks(&manager, 8);
    for id in [first, later] {
        sound.pause_playback(id).unwrap();
    }
    process_blocks(&manager, 8);
    sound.seek_playback_seconds(first, SEEK_SECONDS).unwrap();
    sound.seek_playback_seconds(later, 2.0).unwrap();
    sound.resume_playback(first).unwrap();
    process_blocks(&manager, 1);
    sound.resume_playback(later).unwrap();
    process_blocks(&manager, 1);
    assert!(sound.playback_status(first).unwrap().cursor_frame > SEEK_FRAME);
    assert_eq!(sound.playback_status(later).unwrap().cursor_frame, 16_000);
    process_blocks(&manager, 1);
    assert!(sound.playback_status(later).unwrap().cursor_frame > 16_000);
}

#[test]
fn paused_clip_cursor_is_not_the_automation_sequence_clock() {
    use zircon_runtime::core::framework::sound::{
        SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationCurve,
        SoundAutomationKeyframe, SoundAutomationTarget, SoundParameterId,
        SoundTimelineAutomationTrack, SoundTimelineSequence, SoundTimelineSequenceId,
    };
    let (manager, _, id) = playing_slice();
    let sound: &dyn SoundManager = &manager;
    let binding = SoundAutomationBindingId::new(1);
    let parameter = SoundParameterId::new("test.seek.level");
    sound
        .bind_automation(SoundAutomationBinding {
            id: binding,
            timeline_track_path: "Timeline/Gain:sound.gain".into(),
            target: SoundAutomationTarget::SynthParameter(parameter.clone()),
            parameter: SoundParameterId::new("value"),
        })
        .unwrap();
    sound
        .schedule_timeline_sequence(SoundTimelineSequence::new(
            SoundTimelineSequenceId::new("independent"),
            1.0,
            false,
            vec![SoundTimelineAutomationTrack {
                binding,
                curve: SoundAutomationCurve::from_keyframes([
                    SoundAutomationKeyframe::linear(0.0, 0.0),
                    SoundAutomationKeyframe::linear(1.0, 1.0),
                ]),
            }],
        ))
        .unwrap();
    assert_eq!(
        sound.advance_timeline_sequences(0.25).unwrap()[0].time_seconds,
        0.25
    );
    sound.pause_playback(id).unwrap();
    process_blocks(&manager, 8);
    sound.seek_playback_seconds(id, SEEK_SECONDS).unwrap();
    process_blocks(&manager, 4);
    let report = sound.advance_timeline_sequences(0.25).unwrap();
    assert_eq!(report[0].time_seconds, 0.5);
    assert_eq!(sound.parameter_value(&parameter).unwrap(), 0.5);
    let status = sound.playback_status(id).unwrap();
    assert!(status.paused);
    assert_eq!(status.cursor_frame, SEEK_FRAME);
}

fn playing_slice() -> (DefaultSoundManager, SoundClipId, SoundPlaybackId) {
    let manager = DefaultSoundManager::with_config(
        None,
        SoundConfig {
            sample_rate_hz: SAMPLE_RATE,
            block_size_frames: BLOCK_FRAMES,
            ..SoundConfig::default()
        },
    );
    {
        let mut state = lock_recover(&manager.state);
        OwnerCpalBackend::with_mock_output_for_test(SAMPLE_RATE, || {
            state.kira.activate(AudioManagerSettings {
                internal_buffer_size: BLOCK_FRAMES,
                ..AudioManagerSettings::default()
            })
        })
        .expect("device-free test backend must activate the production Kira manager");
    }
    let uri = AssetUri::parse("res://audio/seek.wav").unwrap();
    let asset = SoundAsset::from_wav_bytes(&uri, &pcm16_wav()).expect("valid mono PCM16 WAV");
    let clip = manager.insert_clip_for_test(asset);
    let sound: &dyn SoundManager = &manager;
    let clip_info = sound.clip_info(clip).unwrap();
    assert_eq!(clip_info.sample_rate_hz, SAMPLE_RATE);
    assert_eq!(clip_info.frame_count, 32_000);
    let id = sound
        .play_clip(
            clip,
            SoundPlaybackSettings::ONCE
                .with_start_seconds(0.5)
                .with_duration_seconds(2.0),
        )
        .unwrap();
    (manager, clip, id)
}

fn process_blocks(manager: &DefaultSoundManager, count: usize) {
    lock_recover(&manager.state)
        .kira
        .with_backend_mut(|backend| {
            for _ in 0..count {
                backend.process_mock_block_for_test();
            }
        })
        .unwrap();
}

fn backend_state(manager: &DefaultSoundManager, id: SoundPlaybackId) -> PlaybackState {
    lock_recover(&manager.state)
        .kira
        .playback_state(id)
        .unwrap()
}

fn pcm16_wav() -> Vec<u8> {
    let frames = SAMPLE_RATE * 4;
    let data_len = frames * 2;
    let mut bytes = Vec::with_capacity((44 + data_len) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1u16.to_le_bytes()); // Mono
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_len.to_le_bytes());
    for frame in 0..frames {
        let sample: i16 = if frame < SAMPLE_RATE { 2_048 } else { -2_048 };
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}
