//! Drive the real Kira mixer without an OS audio device; read the production SoundManager API.
use kira::{sound::PlaybackState, AudioManagerSettings};
use zircon_runtime::asset::{AssetUri, SoundAsset};
use zircon_runtime::core::framework::sound::{
    SoundManager, SoundPlaybackId, SoundPlaybackSettings,
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
