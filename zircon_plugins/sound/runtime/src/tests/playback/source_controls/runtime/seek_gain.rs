use super::fixture::RuntimeSourceControlFixture;

#[test]
fn source_seek_and_gain_affect_rendered_samples_and_cursor() {
    let fixture = RuntimeSourceControlFixture::new();

    fixture
        .sound
        .seek_source_seconds(fixture.source, 0.2)
        .unwrap();
    fixture.sound.set_source_gain(fixture.source, 2.0).unwrap();

    assert_eq!(fixture.sound.render_mix(1).unwrap().samples, vec![0.6, 0.6]);
    assert_eq!(
        fixture
            .sound
            .source_status(fixture.source)
            .unwrap()
            .cursor_frame,
        3
    );
}
