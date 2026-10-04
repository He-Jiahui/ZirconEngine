use super::*;
use std::hint::black_box;
use std::time::{Duration, Instant};

use symphonia::core::audio::{AsAudioBufferRef, AudioBuffer, Signal, SignalSpec};

#[test]
fn package_declares_audio_importers() {
    let manifest = package_manifest();

    assert_eq!(manifest.id, PLUGIN_ID);
    assert!(manifest
        .capabilities
        .contains(&RUNTIME_CAPABILITY.to_string()));
    assert!(manifest
        .asset_importers
        .iter()
        .any(|importer| importer.source_extensions.contains(&"flac".to_string())));
    assert!(!manifest
        .asset_importers
        .iter()
        .any(|importer| importer.source_extensions.contains(&"opus".to_string())));
}

#[test]
fn declaration_projects_audio_package_metadata() {
    let descriptor = runtime_plugin_descriptor();
    let manifest = package_manifest();

    assert_eq!(descriptor.package_id(), AUDIO_IMPORTER_DECLARATION.id());
    assert_eq!(descriptor.category(), AUDIO_IMPORTER_DECLARATION.category());
    assert_eq!(
        descriptor.target_modes(),
        AUDIO_IMPORTER_DECLARATION.target_modes()
    );
    assert_eq!(
        descriptor.capabilities(),
        runtime_capabilities()
            .iter()
            .map(|capability| capability.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        manifest.supported_platforms.as_slice(),
        AUDIO_IMPORTER_DECLARATION.supported_platforms()
    );
    assert_eq!(
        manifest.default_packaging.as_slice(),
        AUDIO_IMPORTER_DECLARATION.default_packaging()
    );
}

#[test]
fn package_manifest_declares_audio_importer_dist_contract() {
    let manifest = package_manifest();
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("audio importer package exposes dist metadata");

    assert!(manifest.default_packaging.contains(
        &zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic
    ));
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::core::framework::project::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.dist_crate, AUDIO_IMPORTER_DIST_CRATE_NAME);
    assert_eq!(
        distribution.runtime_entry,
        AUDIO_IMPORTER_DIST_RUNTIME_ENTRY
    );

    let dist_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "audio_importer.dist")
        .expect("audio importer package includes native dist module");
    assert_eq!(
        dist_module.kind,
        zircon_runtime::plugin::PluginModuleKind::Native
    );
    assert_eq!(dist_module.crate_name, AUDIO_IMPORTER_DIST_CRATE_NAME);
    assert!(dist_module.target_modes.contains(
        &zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime
    ));
    assert!(dist_module
        .target_modes
        .contains(&zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost));
    assert!(dist_module
        .capabilities
        .contains(&WAV_IMPORTER_CAPABILITY.to_string()));
    assert!(dist_module
        .capabilities
        .contains(&CODEC_IMPORTER_CAPABILITY.to_string()));
}

#[test]
fn registration_contributes_module_and_importers() {
    let report = plugin_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == MODULE_NAME));
    assert_eq!(report.extensions.asset_importers().descriptors().len(), 2);
}

#[test]
fn codec_channel_masks_preserve_named_sound_layouts_when_supported() {
    assert_eq!(
        sound_channel_layout_from_symphonia_channels(
            Channels::FRONT_LEFT
                | Channels::FRONT_RIGHT
                | Channels::FRONT_CENTRE
                | Channels::LFE1
                | Channels::SIDE_LEFT
                | Channels::SIDE_RIGHT
        ),
        AudioChannelLayout::surround_5_1_side()
    );
    assert_eq!(
        sound_channel_layout_from_symphonia_channels(
            Channels::FRONT_LEFT | Channels::FRONT_RIGHT | Channels::TOP_CENTRE
        ),
        AudioChannelLayout::discrete(3)
    );
}

#[test]
fn wav_importer_decodes_sound_asset() {
    let report = plugin_registration();
    let importer = report
        .extensions
        .asset_importers()
        .select(std::path::Path::new("tone.wav"))
        .unwrap();
    let context = zircon_runtime::asset::AssetImportContext::new(
        "tone.wav".into(),
        zircon_runtime::asset::AssetUri::parse("res://audio/tone.wav").unwrap(),
        tiny_wav_bytes(),
        Default::default(),
    );

    let outcome = importer.import(&context).unwrap();
    let imported = &outcome.root_entry().expect("root sound asset entry").asset;

    match imported {
        zircon_runtime::asset::ImportedAsset::Sound(sound) => {
            assert_eq!(sound.sample_rate_hz, 8_000);
            assert_eq!(sound.channel_count, 1);
            assert_eq!(sound.channel_layout, AudioChannelLayout::mono());
            assert_eq!(sound.frame_count(), 2);
            assert_eq!(sound.duration_seconds(), 2.0 / 8_000.0);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn wav_importer_rejects_partial_multichannel_frame() {
    let report = plugin_registration();
    let importer = report
        .extensions
        .asset_importers()
        .select(std::path::Path::new("partial.wav"))
        .unwrap();
    let context = zircon_runtime::asset::AssetImportContext::new(
        "partial.wav".into(),
        zircon_runtime::asset::AssetUri::parse("res://audio/partial.wav").unwrap(),
        partial_stereo_wav_bytes(),
        Default::default(),
    );

    let error = importer.import(&context).unwrap_err();

    assert!(error
        .to_string()
        .contains("wav data chunk did not align to whole audio frames"));
}

#[test]
fn decoder_packet_failures_are_terminal_and_include_the_packet_timestamp() {
    let malformed_packet =
        decoder_packet_error(42, SymphoniaError::DecodeError("corrupt codec packet"));
    assert_eq!(
        malformed_packet,
        "decode audio packet at timestamp 42: malformed stream: corrupt codec packet"
    );

    let truncated_packet = decoder_packet_error(
        99,
        SymphoniaError::IoError(std::io::Error::from(ErrorKind::UnexpectedEof)),
    );
    assert!(truncated_packet.contains("decode audio packet at timestamp 99"));
    assert!(truncated_packet.contains("unexpected end of file"));
}

#[test]
fn codec_importer_decodes_ogg_sound_asset() {
    let report = plugin_registration();
    let importer = report
        .extensions
        .asset_importers()
        .select(std::path::Path::new("collision.ogg"))
        .unwrap();
    let context = zircon_runtime::asset::AssetImportContext::new(
        "collision.ogg".into(),
        zircon_runtime::asset::AssetUri::parse("res://audio/collision.ogg").unwrap(),
        include_bytes!("../../../../../dev/bevy/assets/sounds/breakout_collision.ogg").to_vec(),
        Default::default(),
    );

    let outcome = importer.import(&context).unwrap();
    let imported = &outcome.root_entry().expect("root sound asset entry").asset;

    match imported {
        zircon_runtime::asset::ImportedAsset::Sound(sound) => {
            assert!(sound.sample_rate_hz > 0);
            assert!(sound.channel_count > 0);
            assert!(sound
                .channel_layout
                .matches_channel_count(sound.channel_count));
            assert!(sound.frame_count() > 0);
            assert_eq!(sound.samples.len() % sound.channel_count as usize, 0);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn audio_hotpath_interleaved_accumulator_reuses_scratch_and_preserves_samples() {
    let first = decoded_stereo_buffer(4, 0.0);
    let second = decoded_stereo_buffer(2, 10.0);
    let larger = decoded_stereo_buffer(8, 20.0);
    let mut accumulator = InterleavedSampleAccumulator::with_capacity(0);

    accumulator
        .append_decoded(first.as_audio_buffer_ref())
        .unwrap();
    accumulator
        .append_decoded(second.as_audio_buffer_ref())
        .unwrap();

    assert_eq!(accumulator.scratch_allocations(), 1);
    assert_eq!(
        accumulator.samples(),
        &[0.0, 100.0, 1.0, 101.0, 2.0, 102.0, 3.0, 103.0, 10.0, 110.0, 11.0, 111.0,]
    );

    accumulator
        .append_decoded(larger.as_audio_buffer_ref())
        .unwrap();
    assert_eq!(accumulator.scratch_allocations(), 2);
}

#[test]
fn audio_hotpath_resident_preallocation_is_checked_and_bounded() {
    assert_eq!(bounded_sample_preallocation(Some(512), Some(2)), 1_024);
    assert_eq!(
        bounded_sample_preallocation(Some(u64::MAX), Some(usize::MAX)),
        MAX_AUDIO_SAMPLE_PREALLOCATION
    );
    assert_eq!(bounded_sample_preallocation(None, Some(2)), 0);
    assert_eq!(bounded_sample_preallocation(Some(512), None), 0);
}

#[test]
#[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
fn audio_hotpath_release_packet_scratch_reuse_p95_gate() {
    const SAMPLE_PAIRS: usize = 21;
    const PACKETS: usize = 1_024;
    const FRAMES_PER_PACKET: usize = 256;
    let buffer = decoded_stereo_buffer(FRAMES_PER_PACKET, 0.0);
    let (legacy_samples, optimized_samples) = alternating_audio_samples(
        SAMPLE_PAIRS,
        || measure_legacy_packet_scratch(&buffer, PACKETS),
        || measure_reused_packet_scratch(&buffer, PACKETS),
    );

    assert_audio_performance_gate(
        "plugins07_audio_packet_scratch_reuse",
        &legacy_samples,
        &optimized_samples,
        20,
        &format!(
            "packets={PACKETS} frames_per_packet={FRAMES_PER_PACKET} channels=2 legacy_scratch_allocations_per_sample={PACKETS} optimized_scratch_allocations_per_sample=1"
        ),
    );
}

#[test]
#[ignore = "release performance gate; run through the Plugins07 coordinator validator"]
fn audio_hotpath_release_sample_preallocation_p95_gate() {
    const SAMPLE_PAIRS: usize = 21;
    const PACKETS: usize = 8_192;
    const SAMPLES_PER_PACKET: usize = 128;
    let packet = vec![0.25_f32; SAMPLES_PER_PACKET];
    let total_samples = PACKETS * SAMPLES_PER_PACKET;
    let (legacy_samples, optimized_samples) = alternating_audio_samples(
        SAMPLE_PAIRS,
        || measure_sample_appends(&packet, PACKETS, 0),
        || measure_sample_appends(&packet, PACKETS, total_samples),
    );

    assert_audio_performance_gate(
        "plugins07_audio_sample_preallocation",
        &legacy_samples,
        &optimized_samples,
        20,
        &format!(
            "packets={PACKETS} samples_per_packet={SAMPLES_PER_PACKET} total_samples={total_samples} legacy_initial_capacity=0 optimized_initial_capacity={total_samples} max_preallocation_samples={MAX_AUDIO_SAMPLE_PREALLOCATION}"
        ),
    );
}

fn decoded_stereo_buffer(frames: usize, offset: f32) -> AudioBuffer<f32> {
    let spec = SignalSpec::new(48_000, Channels::FRONT_LEFT | Channels::FRONT_RIGHT);
    let mut buffer = AudioBuffer::<f32>::new(frames as u64, spec);
    buffer.render_reserved(None);
    for (index, sample) in buffer.chan_mut(0).iter_mut().enumerate() {
        *sample = offset + index as f32;
    }
    for (index, sample) in buffer.chan_mut(1).iter_mut().enumerate() {
        *sample = offset + 100.0 + index as f32;
    }
    buffer
}

fn measure_legacy_packet_scratch(buffer: &AudioBuffer<f32>, packet_count: usize) -> Duration {
    let total_samples = buffer.frames() * buffer.spec().channels.count() * packet_count;
    let mut samples = Vec::with_capacity(total_samples);
    let started = Instant::now();
    for _ in 0..packet_count {
        let decoded = black_box(buffer).as_audio_buffer_ref();
        let spec = *decoded.spec();
        let mut scratch = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        scratch.copy_interleaved_ref(decoded);
        samples.extend_from_slice(scratch.samples());
    }
    black_box(&samples);
    started.elapsed()
}

fn measure_reused_packet_scratch(buffer: &AudioBuffer<f32>, packet_count: usize) -> Duration {
    let total_samples = buffer.frames() * buffer.spec().channels.count() * packet_count;
    let mut accumulator = InterleavedSampleAccumulator::with_capacity(total_samples);
    let started = Instant::now();
    for _ in 0..packet_count {
        accumulator
            .append_decoded(black_box(buffer).as_audio_buffer_ref())
            .unwrap();
    }
    black_box(accumulator.samples());
    started.elapsed()
}

fn measure_sample_appends(
    packet: &[f32],
    packet_count: usize,
    initial_capacity: usize,
) -> Duration {
    let started = Instant::now();
    let mut samples = Vec::with_capacity(black_box(initial_capacity));
    for _ in 0..packet_count {
        samples.extend_from_slice(black_box(packet));
    }
    black_box(&samples);
    started.elapsed()
}

fn alternating_audio_samples(
    sample_pairs: usize,
    mut legacy: impl FnMut() -> Duration,
    mut optimized: impl FnMut() -> Duration,
) -> (Vec<Duration>, Vec<Duration>) {
    let mut legacy_samples = Vec::with_capacity(sample_pairs);
    let mut optimized_samples = Vec::with_capacity(sample_pairs);
    for pair in 0..sample_pairs {
        if pair % 2 == 0 {
            legacy_samples.push(legacy());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            legacy_samples.push(legacy());
        }
    }
    (legacy_samples, optimized_samples)
}

fn nearest_rank_audio_p95(samples: &[Duration]) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn audio_durations_csv(samples: &[Duration]) -> String {
    samples
        .iter()
        .map(|sample| sample.as_nanos().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn assert_audio_performance_gate(
    marker: &str,
    legacy_samples: &[Duration],
    optimized_samples: &[Duration],
    threshold_percent: u128,
    workload: &str,
) {
    let legacy_p95 = nearest_rank_audio_p95(legacy_samples).as_nanos();
    let optimized_p95 = nearest_rank_audio_p95(optimized_samples).as_nanos();
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT {marker} sample_pairs=21 order=alternating_legacy_first_even {workload} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={threshold_percent}",
        audio_durations_csv(legacy_samples),
        audio_durations_csv(optimized_samples),
    );
    assert!(
        improvement_percent >= threshold_percent,
        "{marker} must improve P95 by at least {threshold_percent}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
    );
}

fn tiny_wav_bytes() -> Vec<u8> {
    wav_bytes(1, 8_000, 16, &[0, 0, 0, 64])
}

fn partial_stereo_wav_bytes() -> Vec<u8> {
    wav_bytes(2, 8_000, 16, &[0, 0])
}

fn wav_bytes(
    channel_count: u16,
    sample_rate_hz: u32,
    bits_per_sample: u16,
    data: &[u8],
) -> Vec<u8> {
    let bytes_per_sample = bits_per_sample / 8;
    let block_align = channel_count * bytes_per_sample;
    let byte_rate = sample_rate_hz * block_align as u32;
    let riff_size = 36 + data.len() as u32;

    let mut bytes = Vec::new();
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
    bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(data);
    bytes
}
