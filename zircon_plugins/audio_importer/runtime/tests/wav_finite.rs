use zircon_plugin_audio_importer_runtime::plugin_registration;
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporter, AssetUri,
    ImportedAsset, SoundAsset, SoundAssetError,
};
use zircon_runtime::core::framework::audio::AudioChannelLayout;

const SAMPLE_RATE: u32 = 8_000;
const PCM: u16 = 1;
const IEEE_FLOAT: u16 = 3;
const SOURCE_NAME: &str = "wav156.wav";

#[test]
fn wav156_mono_qnan_is_not_published() {
    let bits = [0, 0x7fc0_0000, 0.25_f32.to_bits()];
    assert_samples(float_wav(1, &bits), 1, &bits, Some(1));
}

#[test]
fn wav156_stereo_qnan_does_not_leak_or_reorder() {
    let bits = [
        (-0.25_f32).to_bits(),
        0.25_f32.to_bits(),
        0.5_f32.to_bits(),
        0xffc0_0000,
        (-0.75_f32).to_bits(),
        0.75_f32.to_bits(),
    ];
    assert_samples(float_wav(2, &bits), 2, &bits, Some(3));
}

#[test]
fn wav156_float_preserves_finite_sample_bits() {
    let bits = [
        0x0000_0000,
        0x8000_0000,
        0x0000_0001,
        0x8000_0001,
        (-0.25_f32).to_bits(),
        0.25_f32.to_bits(),
    ];
    assert_samples(float_wav(1, &bits), 1, &bits, None);
}

#[test]
fn wav156_float_preserves_saturation_for_overrange_and_infinity() {
    let bits = [1.5_f32, -1.5, f32::INFINITY, f32::NEG_INFINITY, 1.0, -1.0].map(f32::to_bits);
    let expected = [1.0_f32, -1.0, 1.0, -1.0, 1.0, -1.0].map(f32::to_bits);
    assert_samples(float_wav(1, &bits), 1, &expected, None);
}

#[test]
fn wav156_pcm16_decodes_normally() {
    let expected = [0.0_f32, 0.5].map(f32::to_bits);
    assert_samples(pcm16_wav(1, &[0, 16_384]), 1, &expected, None);
}

#[test]
fn wav156_pcm16_rejects_partial_stereo_frame() {
    // All chunk sizes and format fields are correct; data has one frame plus one sample.
    let (_, parsed, imported) = decode_both(pcm16_wav(2, &[0, 16_384, -16_384]));
    let parser_error = parsed.expect_err("three PCM16 samples cannot form two stereo frames");
    assert_eq!(parser_error, SoundAssetError::DataFrameAlignment);
    match imported.expect_err("unaligned PCM16 must not admit a Sound root") {
        AssetImportError::Parse(message) => {
            assert_eq!(message, format!("decode wav {SOURCE_NAME}: {parser_error}"));
        }
        other => panic!("expected importer Parse for {parser_error:?}, got {other:?}"),
    }
}

fn decode_both(
    bytes: Vec<u8>,
) -> (
    AssetUri,
    Result<SoundAsset, SoundAssetError>,
    Result<AssetImportOutcome, AssetImportError>,
) {
    let context = AssetImportContext::new(
        SOURCE_NAME.into(),
        AssetUri::parse("res://audio/wav156.wav").unwrap(),
        bytes,
        Default::default(),
    );
    let report = plugin_registration();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    let registry = report.extensions.asset_importers();
    let selected = registry.select(&context.source_path).expect("WAV handler");
    assert_eq!(selected.descriptor().id, "audio_importer.wav");

    let parsed = SoundAsset::from_wav_bytes(&context.uri, &context.source_bytes);
    let imported = AssetImporter::with_registry(registry.clone()).import_context(&context);
    (context.uri, parsed, imported)
}

fn assert_samples(
    bytes: Vec<u8>,
    channels: u16,
    expected_bits: &[u32],
    replaceable_sample: Option<usize>,
) {
    let (uri, parsed, imported) = decode_both(bytes);
    // These cases exercise the successful decode contract, not a new typed rejection policy.
    let parsed = parsed.expect("valid, frame-aligned WAV must decode");
    let outcome = imported.expect("registered WAV importer must accept the same WAV");
    assert_eq!(outcome.entries.len(), 1);
    let root = outcome.root_entry().expect("root Sound entry");
    assert_eq!(root.locator, uri);
    let ImportedAsset::Sound(imported) = &root.asset else {
        panic!("expected a Sound root, got {:?}", root.asset);
    };

    let to_bits = |sound: &SoundAsset| -> Vec<u32> {
        sound.samples.iter().copied().map(f32::to_bits).collect()
    };
    let parser_bits = to_bits(&parsed);
    let imported_bits = to_bits(imported);
    eprintln!("SoundAsset::from_wav_bytes sample bits: {parser_bits:08x?}");
    eprintln!("audio_importer.wav Sound root sample bits: {imported_bits:08x?}");

    let layout = match channels {
        1 => AudioChannelLayout::mono(),
        2 => AudioChannelLayout::stereo(),
        _ => unreachable!("fixtures use only mono or stereo"),
    };
    let sample_count = expected_bits.len();
    let frames = sample_count / usize::from(channels);
    for (route, sound) in [("parser", &parsed), ("importer", imported)] {
        assert_eq!(sound.uri, uri, "{route} URI");
        assert_eq!(sound.sample_rate_hz, SAMPLE_RATE, "{route} sample rate");
        assert_eq!(sound.channel_count, channels, "{route} channel count");
        assert_eq!(sound.channel_layout, layout, "{route} channel layout");
        assert_eq!(sound.frame_count(), frames, "{route} frame count");
        assert_eq!(sound.samples.len(), sample_count, "{route} sample count");
    }

    let matches = |bits: &[u32]| {
        bits.iter()
            .zip(expected_bits)
            .enumerate()
            .all(|(index, (&actual, &expected))| {
                if replaceable_sample == Some(index) {
                    // NaN and infinities cannot belong to this finite normalized range.
                    (-1.0_f32..=1.0).contains(&f32::from_bits(actual))
                } else {
                    actual == expected
                }
            })
    };
    let parser_matches = matches(&parser_bits);
    let importer_matches = matches(&imported_bits);
    assert!(
        parser_matches && importer_matches,
        "unexpected sample bits; expected {expected_bits:08x?}, with a finite normalized \
         replacement only where specified ({replaceable_sample:?}); \
         parser={parser_bits:08x?} (match={parser_matches}), \
         importer={imported_bits:08x?} (match={importer_matches})"
    );
}

fn float_wav(channels: u16, bits: &[u32]) -> Vec<u8> {
    let data: Vec<u8> = bits.iter().flat_map(|bits| bits.to_le_bytes()).collect();
    wav(IEEE_FLOAT, channels, 32, &data)
}

fn pcm16_wav(channels: u16, samples: &[i16]) -> Vec<u8> {
    let data: Vec<_> = samples.iter().flat_map(|s| s.to_le_bytes()).collect();
    wav(PCM, channels, 16, &data)
}

fn wav(tag: u16, channels: u16, bits_per_sample: u16, data: &[u8]) -> Vec<u8> {
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = SAMPLE_RATE * u32::from(block_align);
    let mut fmt = Vec::new();
    fmt.extend_from_slice(&tag.to_le_bytes());
    fmt.extend_from_slice(&channels.to_le_bytes());
    fmt.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    fmt.extend_from_slice(&byte_rate.to_le_bytes());
    fmt.extend_from_slice(&block_align.to_le_bytes());
    fmt.extend_from_slice(&bits_per_sample.to_le_bytes());
    if tag == IEEE_FLOAT {
        fmt.extend_from_slice(&0_u16.to_le_bytes()); // WAVEFORMATEX cbSize
    }

    let mut bytes = b"RIFF\0\0\0\0WAVE".to_vec();
    append_chunk(&mut bytes, b"fmt ", &fmt);
    if tag == IEEE_FLOAT {
        let frame_bytes = usize::from(block_align);
        assert_eq!(data.len() % frame_bytes, 0);
        let frames = (data.len() / frame_bytes) as u32;
        append_chunk(&mut bytes, b"fact", &frames.to_le_bytes());
    }
    append_chunk(&mut bytes, b"data", data);
    let riff_size = (bytes.len() - 8) as u32;
    bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());
    bytes
}

fn append_chunk(bytes: &mut Vec<u8>, tag: &[u8; 4], payload: &[u8]) {
    bytes.extend_from_slice(tag);
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(payload);
    if payload.len() % 2 != 0 {
        bytes.push(0); // Padding is outside the chunk's declared payload size.
    }
}
