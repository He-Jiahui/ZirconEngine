use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::{resolve_asset_manager, AssetUri, ProjectManifest};
use crate::core::framework::sound::SoundPlaybackSettings;
use crate::core::manager::resolve_sound_manager;
use crate::core::CoreRuntime;

#[test]
fn sound_manager_loads_project_wav_and_mixes_playback_to_stereo() {
    let project_root = unique_temp_project_root("sound-manager-wav-mix");
    write_manifest(&project_root);
    write_pcm16_wav(
        project_root.join("assets/audio/ping.wav"),
        48_000,
        1,
        &[0, 16_384, -16_384, 32_767],
    );

    let runtime = CoreRuntime::new();
    runtime
        .register_module(crate::asset::module_descriptor())
        .unwrap();
    runtime.register_module(super::module_descriptor()).unwrap();
    runtime
        .activate_module(crate::asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime.activate_module(super::SOUND_MODULE_NAME).unwrap();

    let asset_manager = resolve_asset_manager(&runtime.handle()).unwrap();
    asset_manager
        .open_project(project_root.to_str().unwrap())
        .unwrap();

    let sound = resolve_sound_manager(&runtime.handle()).unwrap();
    let clip = sound.load_clip("res://audio/ping.wav").unwrap();
    let info = sound.clip_info(clip).unwrap();

    assert_eq!(info.sample_rate_hz, 48_000);
    assert_eq!(info.channel_count, 1);
    assert_eq!(info.frame_count, 4);

    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let first_mix = sound.render_mix(2).unwrap();
    assert_eq!(first_mix.sample_rate_hz, 48_000);
    assert_eq!(first_mix.channel_count, 2);
    assert_samples_close(&first_mix.samples, &[0.0, 0.0, 0.5, 0.5]);

    let second_mix = sound.render_mix(3).unwrap();
    assert_samples_close(&second_mix.samples, &[-0.5, -0.5, 1.0, 1.0, 0.0, 0.0]);

    fs::remove_dir_all(project_root).ok();
}

#[test]
fn sound_manager_stop_playback_removes_it_from_future_mixes() {
    let (runtime, project_root) =
        runtime_with_sound_project("sound-manager-stop-playback", &[16_384, 16_384]);
    let sound = resolve_sound_manager(&runtime.handle()).unwrap();
    let clip = sound.load_clip("res://audio/ping.wav").unwrap();
    let playback = sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    sound.stop_playback(playback).unwrap();

    let mix = sound.render_mix(2).unwrap();
    assert_samples_close(&mix.samples, &[0.0, 0.0, 0.0, 0.0]);
    assert_eq!(
        sound.stop_playback(playback),
        Err(crate::core::framework::sound::SoundError::UnknownPlayback { playback })
    );

    fs::remove_dir_all(project_root).ok();
}

#[test]
fn sound_manager_looped_playback_wraps_across_clip_end() {
    let (runtime, project_root) =
        runtime_with_sound_project("sound-manager-looped-playback", &[16_384, -16_384]);
    let sound = resolve_sound_manager(&runtime.handle()).unwrap();
    let clip = sound.load_clip("res://audio/ping.wav").unwrap();
    sound
        .play_clip(
            clip,
            SoundPlaybackSettings {
                looped: true,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();

    let mix = sound.render_mix(5).unwrap();
    assert_samples_close(
        &mix.samples,
        &[0.5, 0.5, -0.5, -0.5, 0.5, 0.5, -0.5, -0.5, 0.5, 0.5],
    );

    fs::remove_dir_all(project_root).ok();
}

fn runtime_with_sound_project(label: &str, samples: &[i16]) -> (CoreRuntime, PathBuf) {
    let project_root = unique_temp_project_root(label);
    write_manifest(&project_root);
    write_pcm16_wav(
        project_root.join("assets/audio/ping.wav"),
        48_000,
        1,
        samples,
    );

    let runtime = CoreRuntime::new();
    runtime
        .register_module(crate::asset::module_descriptor())
        .unwrap();
    runtime.register_module(super::module_descriptor()).unwrap();
    runtime
        .activate_module(crate::asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime.activate_module(super::SOUND_MODULE_NAME).unwrap();

    let asset_manager = resolve_asset_manager(&runtime.handle()).unwrap();
    asset_manager
        .open_project(project_root.to_str().unwrap())
        .unwrap();

    (runtime, project_root)
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("zircon-{label}-{nanos}"));
    fs::create_dir_all(root.join("assets/audio")).unwrap();
    root
}

fn write_manifest(project_root: &Path) {
    ProjectManifest::new(
        "Sound Test Project",
        AssetUri::parse("res://audio/ping.wav").unwrap(),
        1,
    )
    .save(project_root.join("zircon-project.toml"))
    .unwrap();
}

fn write_pcm16_wav(path: PathBuf, sample_rate_hz: u32, channel_count: u16, samples: &[i16]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }

    let bits_per_sample = 16_u16;
    let block_align = channel_count * (bits_per_sample / 8);
    let byte_rate = sample_rate_hz * block_align as u32;
    let data_size = (samples.len() * std::mem::size_of::<i16>()) as u32;
    let riff_size = 36 + data_size;

    let mut bytes = Vec::with_capacity((riff_size + 8) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&channel_count.to_le_bytes());
    bytes.extend_from_slice(&sample_rate_hz.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }

    fs::write(path, bytes).unwrap();
}

fn assert_samples_close(actual: &[f32], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected.iter()).enumerate() {
        assert!(
            (actual - expected).abs() <= 1.0e-4,
            "sample {index} mismatch: expected {expected}, got {actual}"
        );
    }
}
