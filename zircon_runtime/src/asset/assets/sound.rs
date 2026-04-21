use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;

const PCM_FORMAT: u16 = 1;
const IEEE_FLOAT_FORMAT: u16 = 3;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundAsset {
    pub uri: AssetUri,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
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
            samples: decode_samples(
                &format,
                data.ok_or_else(|| "wav file is missing data chunk".to_string())?,
            )?,
        })
    }

    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.channel_count as usize
    }
}

#[derive(Clone, Copy, Debug)]
struct WavFormat {
    audio_format: u16,
    channel_count: u16,
    sample_rate_hz: u32,
    block_align: u16,
    bits_per_sample: u16,
}

fn parse_format_chunk(bytes: &[u8]) -> Result<WavFormat, String> {
    if bytes.len() < 16 {
        return Err("wav fmt chunk is too small".to_string());
    }

    Ok(WavFormat {
        audio_format: read_u16(bytes, 0)?,
        channel_count: read_u16(bytes, 2)?,
        sample_rate_hz: read_u32(bytes, 4)?,
        block_align: read_u16(bytes, 12)?,
        bits_per_sample: read_u16(bytes, 14)?,
    })
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
