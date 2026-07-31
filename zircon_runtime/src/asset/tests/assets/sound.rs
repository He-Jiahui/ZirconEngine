use crate::asset::{AssetUri, SoundAsset, SoundAssetError};
use crate::core::framework::audio::AudioChannelLayout;

const WAVE_FORMAT_EXTENSIBLE: u16 = 0xfffe;
const PCM_SUBFORMAT_GUID: [u8; 16] = [
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0xaa, 0x00, 0x38, 0x9b, 0x71,
];
const SPEAKER_FRONT_LEFT: u32 = 0x0000_0001;
const SPEAKER_FRONT_RIGHT: u32 = 0x0000_0002;
const SPEAKER_FRONT_CENTER: u32 = 0x0000_0004;
const SPEAKER_LOW_FREQUENCY: u32 = 0x0000_0008;
const SPEAKER_SIDE_LEFT: u32 = 0x0000_0200;
const SPEAKER_SIDE_RIGHT: u32 = 0x0000_0400;
const SPEAKER_TOP_CENTER: u32 = 0x0000_0800;

#[test]
fn sound_wav_decode_uses_infallible_reads_after_sample_alignment_validation() {
    let source = include_str!("../../assets/sound.rs");
    assert!(!source.contains(".map(|chunk| Ok(read_i16"));
    assert!(!source.contains(".map(|chunk| Ok(read_i32"));
    assert!(!source.contains(".map(|chunk| Ok(read_f32"));
}

#[test]
fn sound_asset_plain_wav_uses_named_layout_fallback_from_channel_count() {
    let asset = SoundAsset::from_wav_bytes(
        &AssetUri::parse("res://audio/stereo.wav").unwrap(),
        &plain_pcm_wav_bytes(2, &[0, 0, 0, 64]),
    )
    .unwrap();

    assert_eq!(asset.channel_count, 2);
    assert_eq!(asset.channel_layout, AudioChannelLayout::stereo());
    assert_eq!(asset.frame_count(), 1);
}

#[test]
fn sound_asset_wav_extensible_preserves_side_bed_channel_layout() {
    let channel_mask = SPEAKER_FRONT_LEFT
        | SPEAKER_FRONT_RIGHT
        | SPEAKER_FRONT_CENTER
        | SPEAKER_LOW_FREQUENCY
        | SPEAKER_SIDE_LEFT
        | SPEAKER_SIDE_RIGHT;
    let asset = SoundAsset::from_wav_bytes(
        &AssetUri::parse("res://audio/surround-side.wav").unwrap(),
        &extensible_pcm_wav_bytes(6, channel_mask, &[0; 12]),
    )
    .unwrap();

    assert_eq!(asset.channel_count, 6);
    assert_eq!(
        asset.channel_layout,
        AudioChannelLayout::surround_5_1_side()
    );
    assert_eq!(asset.frame_count(), 1);
}

#[test]
fn sound_asset_rejects_wav_extensible_unsupported_speaker_mask_bits() {
    let error = SoundAsset::from_wav_bytes(
        &AssetUri::parse("res://audio/height.wav").unwrap(),
        &extensible_pcm_wav_bytes(
            3,
            SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_TOP_CENTER,
            &[0; 6],
        ),
    )
    .unwrap_err();

    assert_eq!(
        error,
        SoundAssetError::UnsupportedSpeakerMaskBits {
            channel_mask: SPEAKER_FRONT_LEFT | SPEAKER_FRONT_RIGHT | SPEAKER_TOP_CENTER,
            unsupported: SPEAKER_TOP_CENTER
        }
    );
    assert!(error.to_string().contains("unsupported speaker bits"));
}

#[test]
fn sound_asset_wav_parse_reports_typed_error_variants() {
    let uri = AssetUri::parse("res://audio/bad.wav").unwrap();

    assert_eq!(
        SoundAsset::from_wav_bytes(&uri, b"RIFF").unwrap_err(),
        SoundAssetError::WavFileTooSmall
    );

    let unsupported_bits =
        SoundAsset::from_wav_bytes(&uri, &wav_bytes(1, 1, 20, None, &[0, 0])).unwrap_err();
    assert_eq!(
        unsupported_bits,
        SoundAssetError::UnsupportedBitsPerSample {
            bits_per_sample: 20
        }
    );
}

fn plain_pcm_wav_bytes(channel_count: u16, data: &[u8]) -> Vec<u8> {
    wav_bytes(1, channel_count, 16, None, data)
}

fn extensible_pcm_wav_bytes(channel_count: u16, channel_mask: u32, data: &[u8]) -> Vec<u8> {
    wav_bytes(
        WAVE_FORMAT_EXTENSIBLE,
        channel_count,
        16,
        Some(channel_mask),
        data,
    )
}

fn wav_bytes(
    audio_format: u16,
    channel_count: u16,
    bits_per_sample: u16,
    channel_mask: Option<u32>,
    data: &[u8],
) -> Vec<u8> {
    let bytes_per_sample = bits_per_sample / 8;
    let block_align = channel_count * bytes_per_sample;
    let sample_rate_hz = 48_000_u32;
    let byte_rate = sample_rate_hz * block_align as u32;
    let fmt_size = if channel_mask.is_some() {
        40_u32
    } else {
        16_u32
    };
    let riff_size = 4 + (8 + fmt_size) + (8 + data.len() as u32);

    let mut bytes = Vec::with_capacity((riff_size + 8) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&fmt_size.to_le_bytes());
    bytes.extend_from_slice(&audio_format.to_le_bytes());
    bytes.extend_from_slice(&channel_count.to_le_bytes());
    bytes.extend_from_slice(&sample_rate_hz.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    if let Some(mask) = channel_mask {
        bytes.extend_from_slice(&22_u16.to_le_bytes());
        bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
        bytes.extend_from_slice(&mask.to_le_bytes());
        bytes.extend_from_slice(&PCM_SUBFORMAT_GUID);
    }
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(data);
    bytes
}
