use zircon_runtime::asset::{AssetUri, SoundAsset};
use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectId, SoundEffectKind, SoundListenerDescriptor,
    SoundListenerId, SoundTrackId,
};

pub(super) fn test_clip(uri: &str, mono_samples: &[f32]) -> SoundAsset {
    test_clip_with_rate(uri, 48_000, mono_samples)
}

pub(super) fn test_clip_with_rate(
    uri: &str,
    sample_rate_hz: u32,
    mono_samples: &[f32],
) -> SoundAsset {
    SoundAsset {
        uri: AssetUri::parse(uri).unwrap(),
        sample_rate_hz,
        channel_count: 1,
        samples: mono_samples.to_vec(),
    }
}

pub(super) fn test_listener() -> SoundListenerDescriptor {
    SoundListenerDescriptor {
        id: SoundListenerId::new(1),
        active: true,
        position: [0.0, 0.0, 0.0],
        forward: [0.0, 0.0, 1.0],
        up: [0.0, 1.0, 0.0],
        left_ear_offset: [-0.08, 0.0, 0.0],
        right_ear_offset: [0.08, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0],
        hrtf_profile: None,
        doppler_tracking: true,
        mixer_target: SoundTrackId::master(),
    }
}

pub(super) fn test_effect(kind: SoundEffectKind) -> SoundEffectDescriptor {
    SoundEffectDescriptor::new(SoundEffectId::new(99), "Test Effect", kind)
}

pub(super) fn assert_sample_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.0001,
        "expected {expected}, got {actual}"
    );
}

pub(super) fn assert_samples_near(actual: &[f32], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert_sample_near(*actual, *expected);
    }
}
