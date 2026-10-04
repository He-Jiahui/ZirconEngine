use super::*;

#[test]
fn render_mip_streaming_metadata_defaults_enabled_and_exempts_ineligible_assets() {
    let metadata = TextureMetadata::default();
    assert!(metadata.streaming_enabled);
    assert!(metadata.allows_mip_streaming(1024, 1024, 8));

    let ui = TextureMetadata {
        usage_hint: TextureUsageHint::Ui,
        ..TextureMetadata::default()
    };
    assert!(!ui.allows_mip_streaming(1024, 1024, 8));

    assert!(!metadata.allows_mip_streaming(128, 128, 8));
    assert!(!metadata.allows_mip_streaming(1024, 1024, 1));

    let disabled = TextureMetadata {
        streaming_enabled: false,
        ..TextureMetadata::default()
    };
    assert!(!disabled.allows_mip_streaming(1024, 1024, 8));

    let generated = TextureMetadata {
        mip_policy: TextureMipPolicy::GenerateRuntime,
        ..TextureMetadata::default()
    };
    assert!(!generated.allows_mip_streaming(1024, 1024, 8));

    let svt = TextureMetadata {
        svt: Some(SvtSettings::default()),
        ..TextureMetadata::default()
    };
    assert!(!svt.allows_mip_streaming(4096, 4096, 12));
}
