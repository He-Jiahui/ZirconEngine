use super::super::super::super::SoundSourceId;
use super::fixture::RuntimeSourceControlFixture;
use zircon_runtime::core::framework::sound::SoundSourceManager;

#[test]
fn source_runtime_controls_reject_invalid_inputs() {
    let fixture = RuntimeSourceControlFixture::new();

    assert!(fixture
        .sound
        .seek_source_seconds(fixture.source, -0.1)
        .is_err());
    assert!(fixture
        .sound
        .set_source_gain(fixture.source, f32::NAN)
        .is_err());
    assert!(fixture.sound.set_source_speed(fixture.source, 0.0).is_err());
    assert!(fixture
        .sound
        .unmute_source(SoundSourceId::new(999_999))
        .is_err());
}
