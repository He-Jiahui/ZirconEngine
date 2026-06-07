use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;
use crate::core::framework::sound::{SoundChannelLayout, SoundSpeakerChannel};

const PCM_FORMAT: u16 = 1;
const IEEE_FLOAT_FORMAT: u16 = 3;
const WAVE_FORMAT_EXTENSIBLE: u16 = 0xfffe;
const PCM_SUBFORMAT_GUID: [u8; 16] = [
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b,
    0x71,
];
const IEEE_FLOAT_SUBFORMAT_GUID: [u8; 16] = [
    0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b,
    0x71,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundAsset {
    pub uri: AssetUri,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    /// Speaker layout for each interleaved frame in `samples`.
    pub channel_layout: SoundChannelLayout,
    pub samples: Vec<f32>,
}

impl SoundAsset {
    pub fn from_wav_bytes(uri: &AssetUri, bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 12 {
            return Err("wav file is too small".to_string());
        }
        if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return Err("wav file is missing RIFF/WAVE header".to_string());
        }

        let mut cursor = 12;
        let mut format = None;
        let mut data = None;
        while cursor + 8 <= bytes.len() {
            let chunk_id = &bytes[cursor..cursor + 4];
            let chunk_size =
                u32::from_le_bytes(bytes[cursor + 4..cursor + 8].try_into().unwrap()) as usize;
            let chunk_start = cursor + 8;
            let chunk_end = chunk_start + chunk_size;
            if chunk_end > bytes.len() {
                return Err("wav chunk extends beyond end of file".to_string());
            }

            match chunk_id {
                b"fmt " => format = Some(parse_format_chunk(&bytes[chunk_start..chunk_end])?),
                b"data" => data = Some(&bytes[chunk_start..chunk_end]),
                _ => {}
            }

            cursor = chunk_end + (chunk_size % 2);
        }

        let format = format.ok_or_else(|| "wav file is missing fmt chunk".to_string())?;
        if format.channel_count == 0 {
            return Err("wav fmt chunk declared zero channels".to_string());
        }
        if format.sample_rate_hz == 0 {
            return Err("wav fmt chunk declared zero sample rate".to_string());
        }

        Ok(Self {
            uri: uri.clone(),
            sample_rate_hz: format.sample_rate_hz,
            channel_count: format.channel_count,
            channel_layout: format.channel_layout()?,
            samples: decode_samples(
                &format,
                data.ok_or_else(|| "wav file is missing data chunk".to_string())?,
            )?,
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
    fn channel_layout(&self) -> Result<SoundChannelLayout, String> {
        match self.channel_mask {
            Some(mask) => channel_layout_from_wav_mask(mask, self.channel_count),
            None => Ok(SoundChannelLayout::for_channel_count(self.channel_count)),
        }
    }
}

fn parse_format_chunk(bytes: &[u8]) -> Result<WavFormat, String> {
    if bytes.len() < 16 {
        return Err("wav fmt chunk is too small".to_string());
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

fn parse_extensible_format_chunk(bytes: &[u8], format: &mut WavFormat) -> Result<(), String> {
    if bytes.len() < 40 {
        return Err("wav extensible fmt chunk is too small".to_string());
    }
    let extension_size = read_u16(bytes, 16)?;
    if extension_size < 22 {
        return Err("wav extensible fmt chunk extension is too small".to_string());
    }
    let valid_bits_per_sample = read_u16(bytes, 18)?;
    if valid_bits_per_sample != 0 && valid_bits_per_sample != format.bits_per_sample {
        return Err(format!(
            "unsupported wav extensible valid bits per sample {valid_bits_per_sample} for container bits {}",
            format.bits_per_sample
        ));
    }
    let subformat = bytes
        .get(24..40)
        .ok_or_else(|| "wav extensible subformat read overflow".to_string())?;
    format.audio_format = if subformat == PCM_SUBFORMAT_GUID {
        PCM_FORMAT
    } else if subformat == IEEE_FLOAT_SUBFORMAT_GUID {
        IEEE_FLOAT_FORMAT
    } else {
        return Err("unsupported wav extensible subformat".to_string());
    };
    let channel_mask = read_u32(bytes, 20)?;
    format.channel_mask = (channel_mask != 0).then_some(channel_mask);
    Ok(())
}

fn decode_samples(format: &WavFormat, data: &[u8]) -> Result<Vec<f32>, String> {
    let bytes_per_sample = match format.bits_per_sample {
        8 => 1,
        16 => 2,
        24 => 3,
        32 => 4,
        other => return Err(format!("unsupported wav bits per sample: {other}")),
    };
    let expected_block_align = format.channel_count as usize * bytes_per_sample;
    if format.block_align as usize != expected_block_align {
        return Err(format!(
            "wav block align {} did not match channel_count {} * bytes_per_sample {}",
            format.block_align, format.channel_count, bytes_per_sample
        ));
    }
    if data.len() % format.block_align as usize != 0 {
        return Err("wav data chunk did not align to whole audio frames".to_string());
    }
    if data.len() % bytes_per_sample != 0 {
        return Err("wav data chunk did not align to sample width".to_string());
    }

    match (format.audio_format, format.bits_per_sample) {
        (PCM_FORMAT, 8) => Ok(data
            .iter()
            .map(|sample| (*sample as f32 - 128.0) / 128.0)
            .collect()),
        (PCM_FORMAT, 16) => Ok(data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes(chunk.try_into().unwrap()) as f32 / 32768.0)
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
            .map(|chunk| i32::from_le_bytes(chunk.try_into().unwrap()) as f32 / 2_147_483_648.0)
            .collect()),
        (IEEE_FLOAT_FORMAT, 32) => Ok(data
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()).clamp(-1.0, 1.0))
            .collect()),
        (audio_format, bits_per_sample) => Err(format!(
            "unsupported wav format {audio_format} / {bits_per_sample}-bit"
        )),
    }
}

fn channel_layout_from_wav_mask(
    channel_mask: u32,
    channel_count: u16,
) -> Result<SoundChannelLayout, String> {
    if channel_mask.count_ones() != channel_count as u32 {
        return Err(format!(
            "wav extensible channel mask {channel_mask:#010x} did not match channel count {channel_count}"
        ));
    }
    let unsupported = channel_mask & !SUPPORTED_WAV_SPEAKER_MASK;
    if unsupported != 0 {
        return Err(format!(
            "wav extensible channel mask {channel_mask:#010x} uses unsupported speaker bits {unsupported:#010x}"
        ));
    }

    let mut speakers = Vec::with_capacity(channel_count as usize);
    for (bit, speaker) in [
        (SPEAKER_FRONT_LEFT, SoundSpeakerChannel::FrontLeft),
        (SPEAKER_FRONT_RIGHT, SoundSpeakerChannel::FrontRight),
        (SPEAKER_FRONT_CENTER, SoundSpeakerChannel::FrontCenter),
        (SPEAKER_LOW_FREQUENCY, SoundSpeakerChannel::LowFrequency),
        (SPEAKER_BACK_LEFT, SoundSpeakerChannel::BackLeft),
        (SPEAKER_BACK_RIGHT, SoundSpeakerChannel::BackRight),
        (SPEAKER_SIDE_LEFT, SoundSpeakerChannel::SideLeft),
        (SPEAKER_SIDE_RIGHT, SoundSpeakerChannel::SideRight),
    ] {
        if channel_mask & bit != 0 {
            speakers.push(speaker);
        }
    }
    Ok(layout_from_speakers(
        channel_count,
        speakers,
        format!("wav_extensible_{channel_mask:08x}"),
    ))
}

fn layout_from_speakers(
    channel_count: u16,
    speakers: Vec<SoundSpeakerChannel>,
    fallback_name: String,
) -> SoundChannelLayout {
    [
        SoundChannelLayout::mono(),
        SoundChannelLayout::stereo(),
        SoundChannelLayout::quad(),
        SoundChannelLayout::surround_5_0(),
        SoundChannelLayout::surround_5_1(),
        SoundChannelLayout::surround_5_1_side(),
        SoundChannelLayout::surround_7_0(),
        SoundChannelLayout::surround_7_1(),
    ]
    .into_iter()
    .find(|layout| layout.channel_count == channel_count && layout.speakers == speakers)
    .unwrap_or(SoundChannelLayout {
        name: fallback_name,
        channel_count,
        speakers,
    })
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let range = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| "wav header read overflow".to_string())?;
    Ok(u16::from_le_bytes(range.try_into().unwrap()))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let range = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| "wav header read overflow".to_string())?;
    Ok(u32::from_le_bytes(range.try_into().unwrap()))
}
