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

// The container reports its normal end-of-input from `next_packet`; a decoder failure instead
// means the selected packet could not produce the complete imported asset. This importer has no
// resynchronization receipt or packet-loss policy, so fail closed rather than silently truncating
// the resident clip.
fn decoder_packet_error(packet_timestamp: u64, error: SymphoniaError) -> String {
    format!("decode audio packet at timestamp {packet_timestamp}: {error}")
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

        let decoded = decoder
            .decode(&packet)
            .map_err(|error| decoder_packet_error(packet.ts(), error))?;
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
#[path = "tests/lib.rs"]
mod tests;
