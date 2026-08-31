use std::io::{Cursor, ErrorKind};
use std::path::Path;

use symphonia::core::audio::{AudioBufferRef, Channels, SampleBuffer};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, ImportedAsset, SoundAsset,
};
use zircon_runtime::core::framework::audio::{AudioChannelLayout, AudioSpeakerChannel};

mod capability;
mod plugin;

pub use capability::{
    AUDIO_IMPORTER_DECLARATION, CODEC_IMPORTER_CAPABILITY, MODULE_NAME, NATIVE_PLUGIN_ID,
    NATIVE_REQUESTED_CAPABILITIES, NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST,
    PLUGIN_ID, RUNTIME_CAPABILITY, RUNTIME_CRATE_NAME, WAV_IMPORTER_CAPABILITY,
};
pub use plugin::{
    asset_importer_descriptors, dist_module_manifest, module_descriptor, package_manifest,
    plugin_registration, runtime_capabilities, runtime_module_manifest, runtime_plugin,
    runtime_plugin_descriptor, runtime_selection, supported_platforms, supported_targets,
    AudioImporterRuntimePlugin, AUDIO_IMPORTER_DIST_CRATE_NAME, AUDIO_IMPORTER_DIST_RUNTIME_ENTRY,
};

// Limit metadata-driven reservation until the resident/streaming budget contract lands.
const MAX_AUDIO_SAMPLE_PREALLOCATION: usize = 4 * 1024 * 1024 / std::mem::size_of::<f32>();

struct InterleavedSampleAccumulator {
    samples: Vec<f32>,
    scratch: Option<SampleBuffer<f32>>,
    #[cfg(test)]
    scratch_allocations: usize,
}

impl InterleavedSampleAccumulator {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
            scratch: None,
            #[cfg(test)]
            scratch_allocations: 0,
        }
    }

    fn append_decoded(&mut self, decoded: AudioBufferRef<'_>) -> Result<(), String> {
        let spec = *decoded.spec();
        let required_samples = decoded
            .capacity()
            .checked_mul(spec.channels.count())
            .ok_or_else(|| "decoded audio packet sample capacity overflowed usize".to_string())?;
        let needs_larger_scratch = self
            .scratch
            .as_ref()
            .is_none_or(|scratch| scratch.capacity() < required_samples);
        if needs_larger_scratch {
            let frame_capacity = u64::try_from(decoded.capacity())
                .map_err(|_| "decoded audio packet frame capacity exceeded u64".to_string())?;
            self.scratch = Some(SampleBuffer::<f32>::new(frame_capacity, spec));
            #[cfg(test)]
            {
                self.scratch_allocations += 1;
            }
        }

        let scratch = self
            .scratch
            .as_mut()
            .expect("scratch buffer is initialized");
        scratch.copy_interleaved_ref(decoded);
        self.samples.extend_from_slice(scratch.samples());
        Ok(())
    }

    fn into_samples(self) -> Vec<f32> {
        self.samples
    }

    #[cfg(test)]
    fn samples(&self) -> &[f32] {
        &self.samples
    }

    #[cfg(test)]
    fn scratch_allocations(&self) -> usize {
        self.scratch_allocations
    }
}

fn bounded_sample_preallocation(frame_count: Option<u64>, channel_count: Option<usize>) -> usize {
    let (Some(frame_count), Some(channel_count)) = (frame_count, channel_count) else {
        return 0;
    };
    usize::try_from(frame_count)
        .ok()
        .and_then(|frame_count| frame_count.checked_mul(channel_count))
        .unwrap_or(MAX_AUDIO_SAMPLE_PREALLOCATION)
        .min(MAX_AUDIO_SAMPLE_PREALLOCATION)
}

pub fn import_wav(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
    let asset =
        SoundAsset::from_wav_bytes(&context.uri, &context.source_bytes).map_err(|error| {
            AssetImportError::Parse(format!(
                "decode wav {}: {error}",
                context.source_path.display()
            ))
        })?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Sound(asset),
    ))
}

pub fn import_symphonia_audio(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let asset = decode_symphonia_audio(
        &context.uri,
        &context.source_path,
        context.source_bytes.clone(),
    )
    .map_err(|error| {
        AssetImportError::Parse(format!(
            "decode audio {}: {error}",
            context.source_path.display()
        ))
    })?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Sound(asset),
    ))
}

fn decode_symphonia_audio(
    uri: &zircon_runtime::asset::AssetUri,
    source_path: &Path,
    source_bytes: Vec<u8>,
) -> Result<SoundAsset, String> {
    let mut hint = Hint::new();
    if let Some(extension) = source_path
        .extension()
        .and_then(|extension| extension.to_str())
    {
        hint.with_extension(extension);
    }
    let stream = MediaSourceStream::new(Box::new(Cursor::new(source_bytes)), Default::default());
    let probed = get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|error| format!("probe audio container: {error}"))?;
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|track| track.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "audio container has no decodable track".to_string())?;
    let track_id = track.id;
    let initial_sample_capacity = bounded_sample_preallocation(
        track.codec_params.n_frames,
        track.codec_params.channels.map(|channels| channels.count()),
    );
    let mut decoder = get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|error| format!("create audio decoder: {error}"))?;

    let mut sample_rate_hz = None;
    let mut channel_count = None;
    let mut channel_layout = None;
    let mut samples = InterleavedSampleAccumulator::with_capacity(initial_sample_capacity);
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(error)) if error.kind() == ErrorKind::UnexpectedEof => {
                break;
            }
            Err(error) => return Err(format!("read audio packet: {error}")),
        };
        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(error)) if error.kind() == ErrorKind::UnexpectedEof => {
                break;
            }
            Err(error) => return Err(format!("decode audio packet: {error}")),
        };
        let spec = *decoded.spec();
        if spec.rate == 0 {
            return Err("decoded audio declared zero sample rate".to_string());
        }
        let decoded_channels = spec.channels.count();
        if decoded_channels == 0 {
            return Err("decoded audio declared zero channels".to_string());
        }
        match sample_rate_hz {
            Some(existing) if existing != spec.rate => {
                return Err(format!(
                    "decoded audio changed sample rate from {existing} to {}",
                    spec.rate
                ));
            }
            None => sample_rate_hz = Some(spec.rate),
            _ => {}
        }
        let decoded_layout = sound_channel_layout_from_symphonia_channels(spec.channels);
        match channel_count {
            Some(existing) if existing != decoded_layout.channel_count as usize => {
                return Err(format!(
                    "decoded audio changed channel count from {existing} to {}",
                    decoded_layout.channel_count
                ));
            }
            Some(_) if channel_layout.as_ref() != Some(&decoded_layout) => {
                return Err("decoded audio changed channel layout".to_string());
            }
            None => {
                channel_count = Some(decoded_layout.channel_count as usize);
                channel_layout = Some(decoded_layout);
            }
            _ => {}
        }

        samples.append_decoded(decoded)?;
    }

    let sample_rate_hz =
        sample_rate_hz.ok_or_else(|| "audio file produced no decoded samples".to_string())?;
    let channel_count =
        channel_count.ok_or_else(|| "audio file produced no decoded channels".to_string())?;
    let channel_layout = channel_layout
        .ok_or_else(|| "audio file produced no decoded channel layout".to_string())?;
    if channel_count > u16::MAX as usize {
        return Err(format!(
            "decoded audio channel count {channel_count} exceeds u16"
        ));
    }
    Ok(SoundAsset {
        uri: uri.clone(),
        sample_rate_hz,
        channel_count: channel_count as u16,
        channel_layout,
        samples: samples.into_samples(),
    })
}

fn sound_channel_layout_from_symphonia_channels(channels: Channels) -> AudioChannelLayout {
    let channel_count = channels.count() as u16;
    if channel_count == 1 {
        return AudioChannelLayout::mono();
    }
    let supported_mask = Channels::FRONT_LEFT
        | Channels::FRONT_RIGHT
        | Channels::FRONT_CENTRE
        | Channels::LFE1
        | Channels::REAR_LEFT
        | Channels::REAR_RIGHT
        | Channels::SIDE_LEFT
        | Channels::SIDE_RIGHT;
    if !supported_mask.contains(channels) {
        return AudioChannelLayout::discrete(channel_count);
    }

    let mut speakers = Vec::with_capacity(channel_count as usize);
    for (channel, speaker) in [
        (Channels::FRONT_LEFT, AudioSpeakerChannel::FrontLeft),
        (Channels::FRONT_RIGHT, AudioSpeakerChannel::FrontRight),
        (Channels::FRONT_CENTRE, AudioSpeakerChannel::FrontCenter),
        (Channels::LFE1, AudioSpeakerChannel::LowFrequency),
        (Channels::REAR_LEFT, AudioSpeakerChannel::BackLeft),
        (Channels::REAR_RIGHT, AudioSpeakerChannel::BackRight),
        (Channels::SIDE_LEFT, AudioSpeakerChannel::SideLeft),
        (Channels::SIDE_RIGHT, AudioSpeakerChannel::SideRight),
    ] {
        if channels.contains(channel) {
            speakers.push(speaker);
        }
    }
    sound_channel_layout_from_speakers(channel_count, speakers)
}

fn sound_channel_layout_from_speakers(
    channel_count: u16,
    speakers: Vec<AudioSpeakerChannel>,
) -> AudioChannelLayout {
    [
        AudioChannelLayout::mono(),
        AudioChannelLayout::stereo(),
        AudioChannelLayout::quad(),
        AudioChannelLayout::surround_5_0(),
        AudioChannelLayout::surround_5_1(),
        AudioChannelLayout::surround_5_1_side(),
        AudioChannelLayout::surround_7_0(),
        AudioChannelLayout::surround_7_1(),
    ]
    .into_iter()
    .find(|layout| layout.channel_count == channel_count && layout.speakers == speakers)
    .unwrap_or(AudioChannelLayout {
        name: format!("codec_channels_{channel_count}"),
        channel_count,
        speakers,
    })
}

#[cfg(test)]
mod tests {
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
        assert!(manifest
            .asset_importers
            .iter()
            .any(|importer| importer.id == "audio_importer.opus"));
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
        assert_eq!(report.extensions.asset_importers().descriptors().len(), 3);
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
            include_bytes!("../../../../dev/bevy/assets/sounds/breakout_collision.ogg").to_vec(),
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
}
