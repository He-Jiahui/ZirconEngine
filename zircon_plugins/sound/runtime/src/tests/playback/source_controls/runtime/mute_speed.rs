use super::fixture::RuntimeSourceControlFixture;
use zircon_runtime::core::framework::sound::{SoundMixRenderManager, SoundSourceManager};

#[test]
fn source_mute_toggle_and_speed_update_runtime_status() {
    let fixture = RuntimeSourceControlFixture::new();

    fixture.sound.mute_source(fixture.source).unwrap();
    assert_eq!(fixture.sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    assert!(fixture.sound.source_status(fixture.source).unwrap().muted);

    fixture.sound.toggle_mute_source(fixture.source).unwrap();
    fixture.sound.set_source_speed(fixture.source, 0.5).unwrap();

    let status = fixture.sound.source_status(fixture.source).unwrap();
    assert_eq!(status.speed, 0.5);
    assert!(!status.muted);
}
