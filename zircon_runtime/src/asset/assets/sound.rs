use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::asset::AssetUri;
use crate::core::framework::audio::{AudioChannelLayout, AudioSpeakerChannel};

const PCM_FORMAT: u16 = 1;
const IEEE_FLOAT_FORMAT: u16 = 3;
const WAVE_FORMAT_EXTENSIBLE: u16 = 0xfffe;
const PCM_SUBFORMAT_GUID: [u8; 16] = [
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71,
];
const IEEE_FLOAT_SUBFORMAT_GUID: [u8; 16] = [
    0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71,
];
const SPEAKER_FRONT_LEFT: u32 = 0x0000_0001;
const SPEAKER_FRONT_RIGHT: u32 = 0x0000_0002;
const SPEAKER_FRONT_CENTER: u32 = 0x0000_0004;
const SPEAKER_LOW_FREQUENCY: u32 = 0x0000_0008;
const SPEAKER_BACK_LEFT: u32 = 0x0000_0010;
const SPEAKER_BACK_RIGHT: u32 = 0x0000_0020;
const SPEAKER_SIDE_LEFT: u32 = 0x0000_0200;
const SPEAKER_SIDE_RIGHT: u32 = 0x0000_0400;
const SUPPORTED_WAV_SPEAKER_MASK: u32 = SPEAKER_FRONT_LEFT
    | SPEAKER_FRONT_RIGHT
    | SPEAKER_FRONT_CENTER
    | SPEAKER_LOW_FREQUENCY
    | SPEAKER_BACK_LEFT
    | SPEAKER_BACK_RIGHT
    | SPEAKER_SIDE_LEFT
    | SPEAKER_SIDE_RIGHT;
const MONO_SPEAKER_MASK: u32 = SPEAKER_FRONT_CENTER;
const STEREO_SPEAKER_MASK: u32 = SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT;
const QUAD_SPEAKER_MASK: u32 =
    SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_BACK_LEFT | SPEAKER_BACK_RIGHT;
const SURROUND_5_0_SPEAKER_MASK: u32 = SPEAKER_FRONT_LEFT
    | SPEAKER_FRONT_RIGHT
    | SPEAKER_FRONT_CENTER
    | SPEAKER_BACK_LEFT
    | SPEAKER_BACK_RIGHT;
const SURROUND_5_1_SPEAKER_MASK: u32 = SURROUND_5_0_SPEAKER_MASK | SPEAKER_LOW_FREQUENCY;
const SURROUND_5_1_SIDE_SPEAKER_MASK: u32 = SPEAKER_FRONT_LEFT
    | SPEAKER_FRONT_RIGHT
    | SPEAKER_FRONT_CENTER
    | SPEAKER_LOW_FREQUENCY
    | SPEAKER_SIDE_LEFT
    | SPEAKER_SIDE_RIGHT;
const SURROUND_7_0_SPEAKER_MASK: u32 =
    SURROUND_5_0_SPEAKER_MASK | SPEAKER_SIDE_LEFT | SPEAKER_SIDE_RIGHT;
const SURROUND_7_1_SPEAKER_MASK: u32 = SURROUND_7_0_SPEAKER_MASK | SPEAKER_LOW_FREQUENCY;

pub type SoundAssetResult<T> = std::result::Result<T, SoundAssetError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum SoundAssetError {
    #[error("wav file is too small")]
    WavFileTooSmall,
    #[error("wav file is missing RIFF/WAVE header")]
    MissingRiffWaveHeader,
    #[error("wav chunk extends beyond end of file")]
    WavChunkExtendsBeyondFile,
    #[error("wav file is missing fmt chunk")]
    MissingFormatChunk,
    #[error("wav fmt chunk declared zero channels")]
    ZeroChannels,
    #[error("wav fmt chunk declared zero sample rate")]
    ZeroSampleRate,
    #[error("wav file is missing data chunk")]
    MissingDataChunk,
    #[error("wav fmt chunk is too small")]
    FormatChunkTooSmall,
    #[error("wav extensible fmt chunk is too small")]
    ExtensibleFormatChunkTooSmall,
    #[error("wav extensible fmt chunk extension is too small")]
    ExtensibleFormatExtensionTooSmall,
    #[error(
        "unsupported wav extensible valid bits per sample {valid_bits_per_sample} for container bits {container_bits}"
    )]
    UnsupportedExtensibleValidBits {
        valid_bits_per_sample: u16,
        container_bits: u16,
    },
    #[error("wav extensible subformat read overflow")]
    ExtensibleSubformatReadOverflow,
    #[error("unsupported wav extensible subformat")]
    UnsupportedExtensibleSubformat,
    #[error("unsupported wav bits per sample: {bits_per_sample}")]
    UnsupportedBitsPerSample { bits_per_sample: u16 },
    #[error(
        "wav block align {block_align} did not match channel_count {channel_count} * bytes_per_sample {bytes_per_sample}"
    )]
    BlockAlignMismatch {
        block_align: u16,
        channel_count: u16,
        bytes_per_sample: usize,
    },
    #[error("wav data chunk did not align to whole audio frames")]
    DataFrameAlignment,
    #[error("wav data chunk did not align to sample width")]
    DataSampleWidthAlignment,
    #[error("unsupported wav format {audio_format} / {bits_per_sample}-bit")]
    UnsupportedFormat {
        audio_format: u16,
        bits_per_sample: u16,
    },
    #[error(
        "wav extensible channel mask {channel_mask:#010x} did not match channel count {channel_count}"
    )]
    ChannelMaskCountMismatch {
        channel_mask: u32,
        channel_count: u16,
    },
    #[error(
        "wav extensible channel mask {channel_mask:#010x} uses unsupported speaker bits {unsupported:#010x}"
    )]
    UnsupportedSpeakerMaskBits { channel_mask: u32, unsupported: u32 },
    #[error("wav header read overflow")]
    HeaderReadOverflow,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundAsset {
    pub uri: AssetUri,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    /// Speaker layout for each interleaved frame in `samples`.
    pub channel_layout: AudioChannelLayout,
    pub samples: Vec<f32>,
}

impl SoundAsset {
    pub fn from_wav_bytes(uri: &AssetUri, bytes: &[u8]) -> SoundAssetResult<Self> {
        if bytes.len() < 12 {
            return Err(SoundAssetError::WavFileTooSmall);
        }
        if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return Err(SoundAssetError::MissingRiffWaveHeader);
        }

        let mut cursor = 12;
        let mut format = None;
        let mut data = None;
        while cursor + 8 <= bytes.len() {
            let chunk_id = &bytes[cursor..cursor + 4];
            let chunk_size = read_u32(bytes, cursor + 4)? as usize;
            let chunk_start = cursor + 8;
            let chunk_end = chunk_start
                .checked_add(chunk_size)
                .ok_or(SoundAssetError::WavChunkExtendsBeyondFile)?;
            if chunk_end > bytes.len() {
                return Err(SoundAssetError::WavChunkExtendsBeyondFile);
            }

            match chunk_id {
                b"fmt " => format = Some(parse_format_chunk(&bytes[chunk_start..chunk_end])?),
                b"data" => data = Some(&bytes[chunk_start..chunk_end]),
                _ => {}
            }
            cursor = chunk_end + (chunk_size % 2);
        }

        let format = format.ok_or(SoundAssetError::MissingFormatChunk)?;
        if format.channel_count == 0 {
            return Err(SoundAssetError::ZeroChannels);
        }
        if format.sample_rate_hz == 0 {
            return Err(SoundAssetError::ZeroSampleRate);
        }

        Ok(Self {
            uri: uri.clone(),
            sample_rate_hz: format.sample_rate_hz,
            channel_count: format.channel_count,
            channel_layout: format.channel_layout()?,
            samples: decode_samples(&format, data.ok_or(SoundAssetError::MissingDataChunk)?)?,
        })
    }

    pub fn frame_count(&self) -> usize {
        let channel_count = self.channel_count as usize;
        if channel_count == 0 {
            return 0;
        }
        self.samples.len() / channel_count
    }

    pub fn duration_seconds(&self) -> f32 {
        if self.sample_rate_hz == 0 {
            return 0.0;
        }
        self.frame_count() as f32 / self.sample_rate_hz as f32
    }
}

#[derive(Clone, Copy, Debug)]
struct WavFormat {
    audio_format: u16,
    channel_count: u16,
    sample_rate_hz: u32,
    block_align: u16,
    bits_per_sample: u16,
    channel_mask: Option<u32>,
}

impl WavFormat {
    fn channel_layout(&self) -> SoundAssetResult<AudioChannelLayout> {
        match self.channel_mask {
            Some(mask) => channel_layout_from_wav_mask(mask, self.channel_count),
            None => Ok(AudioChannelLayout::for_channel_count(self.channel_count)),
        }
    }
}

fn parse_format_chunk(bytes: &[u8]) -> SoundAssetResult<WavFormat> {
    if bytes.len() < 16 {
        return Err(SoundAssetError::FormatChunkTooSmall);
    }

    let mut format = WavFormat {
        audio_format: read_u16(bytes, 0)?,
        channel_count: read_u16(bytes, 2)?,
        sample_rate_hz: read_u32(bytes, 4)?,
        block_align: read_u16(bytes, 12)?,
        bits_per_sample: read_u16(bytes, 14)?,
        channel_mask: None,
    };
    if format.audio_format == WAVE_FORMAT_EXTENSIBLE {
        parse_extensible_format_chunk(bytes, &mut format)?;
    }
    Ok(format)
}

fn parse_extensible_format_chunk(bytes: &[u8], format: &mut WavFormat) -> SoundAssetResult<()> {
    if bytes.len() < 40 {
        return Err(SoundAssetError::ExtensibleFormatChunkTooSmall);
    }
    let extension_size = read_u16(bytes, 16)?;
    if extension_size < 22 {
        return Err(SoundAssetError::ExtensibleFormatExtensionTooSmall);
    }
    let valid_bits_per_sample = read_u16(bytes, 18)?;
    if valid_bits_per_sample != 0 && valid_bits_per_sample != format.bits_per_sample {
        return Err(SoundAssetError::UnsupportedExtensibleValidBits {
            valid_bits_per_sample,
            container_bits: format.bits_per_sample,
        });
    }
    let subformat = bytes
        .get(24..40)
        .ok_or(SoundAssetError::ExtensibleSubformatReadOverflow)?;
    format.audio_format = if subformat == PCM_SUBFORMAT_GUID {
        PCM_FORMAT
    } else if subformat == IEEE_FLOAT_SUBFORMAT_GUID {
        IEEE_FLOAT_FORMAT
    } else {
        return Err(SoundAssetError::UnsupportedExtensibleSubformat);
    };
    let channel_mask = read_u32(bytes, 20)?;
    format.channel_mask = (channel_mask != 0).then_some(channel_mask);
    Ok(())
}

fn decode_samples(format: &WavFormat, data: &[u8]) -> SoundAssetResult<Vec<f32>> {
    let bytes_per_sample = match format.bits_per_sample {
        8 => 1,
        16 => 2,
        24 => 3,
        32 => 4,
        bits_per_sample => {
            return Err(SoundAssetError::UnsupportedBitsPerSample { bits_per_sample });
        }
    };
    let expected_block_align = format.channel_count as usize * bytes_per_sample;
    if format.block_align as usize != expected_block_align {
        return Err(SoundAssetError::BlockAlignMismatch {
            block_align: format.block_align,
            channel_count: format.channel_count,
            bytes_per_sample,
        });
    }
    if data.len() % format.block_align as usize != 0 {
        return Err(SoundAssetError::DataFrameAlignment);
    }
    if data.len() % bytes_per_sample != 0 {
        return Err(SoundAssetError::DataSampleWidthAlignment);
    }

    match (format.audio_format, format.bits_per_sample) {
        (PCM_FORMAT, 8) => Ok(data
            .iter()
            .map(|sample| (*sample as f32 - 128.0) / 128.0)
            .collect()),
        (PCM_FORMAT, 16) => Ok(data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0)
            .collect()),
        (PCM_FORMAT, 24) => Ok(data
            .chunks_exact(3)
            .map(|chunk| {
                let value =
                    ((chunk[2] as i32) << 24 >> 8) | ((chunk[1] as i32) << 8) | (chunk[0] as i32);
                value as f32 / 8_388_608.0
            })
            .collect()),
        (PCM_FORMAT, 32) => Ok(data
            .chunks_exact(4)
            .map(|chunk| {
                i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f32
                    / 2_147_483_648.0
            })
            .collect()),
        (IEEE_FLOAT_FORMAT, 32) => Ok(data
            .chunks_exact(4)
            .map(|chunk| {
                f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]).clamp(-1.0, 1.0)
            })
            .collect()),
        (audio_format, bits_per_sample) => Err(SoundAssetError::UnsupportedFormat {
            audio_format,
            bits_per_sample,
        }),
    }
}

fn channel_layout_from_wav_mask(
    channel_mask: u32,
    channel_count: u16,
) -> SoundAssetResult<AudioChannelLayout> {
    if channel_mask.count_ones() != channel_count as u32 {
        return Err(SoundAssetError::ChannelMaskCountMismatch {
            channel_mask,
            channel_count,
        });
    }
    let unsupported = channel_mask & !SUPPORTED_WAV_SPEAKER_MASK;
    if unsupported != 0 {
        return Err(SoundAssetError::UnsupportedSpeakerMaskBits {
            channel_mask,
            unsupported,
        });
    }

    let named_layout = match channel_mask {
        MONO_SPEAKER_MASK => Some(AudioChannelLayout::mono()),
        STEREO_SPEAKER_MASK => Some(AudioChannelLayout::stereo()),
        QUAD_SPEAKER_MASK => Some(AudioChannelLayout::quad()),
        SURROUND_5_0_SPEAKER_MASK => Some(AudioChannelLayout::surround_5_0()),
        SURROUND_5_1_SPEAKER_MASK => Some(AudioChannelLayout::surround_5_1()),
        SURROUND_5_1_SIDE_SPEAKER_MASK => Some(AudioChannelLayout::surround_5_1_side()),
        SURROUND_7_0_SPEAKER_MASK => Some(AudioChannelLayout::surround_7_0()),
        SURROUND_7_1_SPEAKER_MASK => Some(AudioChannelLayout::surround_7_1()),
        _ => None,
    };
    if let Some(layout) = named_layout {
        return Ok(layout);
    }

    let mut speakers = Vec::with_capacity(channel_count as usize);
    for (bit, speaker) in [
        (SPEAKER_FRONT_LEFT, AudioSpeakerChannel::FrontLeft),
        (SPEAKER_FRONT_RIGHT, AudioSpeakerChannel::FrontRight),
        (SPEAKER_FRONT_CENTER, AudioSpeakerChannel::FrontCenter),
        (SPEAKER_LOW_FREQUENCY, AudioSpeakerChannel::LowFrequency),
        (SPEAKER_BACK_LEFT, AudioSpeakerChannel::BackLeft),
        (SPEAKER_BACK_RIGHT, AudioSpeakerChannel::BackRight),
        (SPEAKER_SIDE_LEFT, AudioSpeakerChannel::SideLeft),
        (SPEAKER_SIDE_RIGHT, AudioSpeakerChannel::SideRight),
    ] {
        if channel_mask & bit != 0 {
            speakers.push(speaker);
        }
    }
    Ok(AudioChannelLayout {
        name: format!("wav_extensible_{channel_mask:08x}"),
        channel_count,
        speakers,
    })
}

fn read_u16(bytes: &[u8], offset: usize) -> SoundAssetResult<u16> {
    Ok(u16::from_le_bytes(read_fixed_bytes::<2>(bytes, offset)?))
}

fn read_u32(bytes: &[u8], offset: usize) -> SoundAssetResult<u32> {
    Ok(u32::from_le_bytes(read_fixed_bytes::<4>(bytes, offset)?))
}

fn read_fixed_bytes<const N: usize>(bytes: &[u8], offset: usize) -> SoundAssetResult<[u8; N]> {
    let end = offset
        .checked_add(N)
        .ok_or(SoundAssetError::HeaderReadOverflow)?;
    let range = bytes
        .get(offset..end)
        .ok_or(SoundAssetError::HeaderReadOverflow)?;
    let mut value = [0; N];
    for (output, input) in value.iter_mut().zip(range.iter().copied()) {
        *output = input;
    }
    Ok(value)
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const LAYOUT_CHECKS_PER_SAMPLE: usize = 8_192;

    #[test]
    #[ignore = "release-only performance contract"]
    fn benchmark_sound_wav_channel_layout_mask_projection() {
        let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_raw.push(measure_layout_projection(
                    legacy_channel_layout_from_wav_mask,
                ));
                optimized_raw.push(measure_layout_projection(channel_layout_from_wav_mask));
            } else {
                optimized_raw.push(measure_layout_projection(channel_layout_from_wav_mask));
                legacy_raw.push(measure_layout_projection(
                    legacy_channel_layout_from_wav_mask,
                ));
            }
        }

        let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
        let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
        let improvement_percent = legacy_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(100)
            / legacy_p95_ns.max(1);
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(50),
            "direct WAV channel-layout selection must improve P95 by at least 50%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
        println!(
            "PERF_RESULT task=plugins07_direct_wav_channel_layout sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank layout_checks_per_sample={LAYOUT_CHECKS_PER_SAMPLE} benchmark_channel_mask=0000063f benchmark_channel_count=8 legacy_layout_allocations_per_decode=18 optimized_layout_allocations_per_decode=2 legacy_transient_allocations_per_decode=16 optimized_transient_allocations_per_decode=0 threshold_percent=50 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
            raw_samples(&legacy_raw),
            raw_samples(&optimized_raw)
        );
    }

    fn measure_layout_projection(
        project: fn(u32, u16) -> SoundAssetResult<AudioChannelLayout>,
    ) -> u128 {
        let started = Instant::now();
        for _ in 0..LAYOUT_CHECKS_PER_SAMPLE {
            black_box(project(black_box(SURROUND_7_1_SPEAKER_MASK), 8).unwrap());
        }
        started.elapsed().as_nanos()
    }

    fn legacy_channel_layout_from_wav_mask(
        channel_mask: u32,
        channel_count: u16,
    ) -> SoundAssetResult<AudioChannelLayout> {
        if channel_mask.count_ones() != channel_count as u32 {
            return Err(SoundAssetError::ChannelMaskCountMismatch {
                channel_mask,
                channel_count,
            });
        }
        let unsupported = channel_mask & !SUPPORTED_WAV_SPEAKER_MASK;
        if unsupported != 0 {
            return Err(SoundAssetError::UnsupportedSpeakerMaskBits {
                channel_mask,
                unsupported,
            });
        }

        let mut speakers = Vec::with_capacity(channel_count as usize);
        for (bit, speaker) in [
            (SPEAKER_FRONT_LEFT, AudioSpeakerChannel::FrontLeft),
            (SPEAKER_FRONT_RIGHT, AudioSpeakerChannel::FrontRight),
            (SPEAKER_FRONT_CENTER, AudioSpeakerChannel::FrontCenter),
            (SPEAKER_LOW_FREQUENCY, AudioSpeakerChannel::LowFrequency),
            (SPEAKER_BACK_LEFT, AudioSpeakerChannel::BackLeft),
            (SPEAKER_BACK_RIGHT, AudioSpeakerChannel::BackRight),
            (SPEAKER_SIDE_LEFT, AudioSpeakerChannel::SideLeft),
            (SPEAKER_SIDE_RIGHT, AudioSpeakerChannel::SideRight),
        ] {
            if channel_mask & bit != 0 {
                speakers.push(speaker);
            }
        }
        Ok(legacy_layout_from_speakers(
            channel_count,
            speakers,
            format!("wav_extensible_{channel_mask:08x}"),
        ))
    }

    fn legacy_layout_from_speakers(
        channel_count: u16,
        speakers: Vec<AudioSpeakerChannel>,
        fallback_name: String,
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
            name: fallback_name,
            channel_count,
            speakers,
        })
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
