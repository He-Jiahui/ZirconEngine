use super::fixture::RuntimeSourceControlFixture;
use zircon_runtime::core::framework::sound::{SoundMixRenderManager, SoundSourceManager};

#[test]
fn source_pause_resume_and_toggle_update_playing_status() {
    let fixture = RuntimeSourceControlFixture::new();

    fixture.sound.pause_source(fixture.source).unwrap();
    assert_eq!(fixture.sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    assert_eq!(
        fixture
            .sound
            .source_status(fixture.source)
            .unwrap()
            .cursor_frame,
        0
    );
    assert!(!fixture.sound.source_status(fixture.source).unwrap().playing);

    fixture.sound.resume_source(fixture.source).unwrap();
    assert!(fixture.sound.source_status(fixture.source).unwrap().playing);

    fixture.sound.toggle_source(fixture.source).unwrap();
    assert!(!fixture.sound.source_status(fixture.source).unwrap().playing);
    fixture.sound.toggle_source(fixture.source).unwrap();
    assert!(fixture.sound.source_status(fixture.source).unwrap().playing);
}
