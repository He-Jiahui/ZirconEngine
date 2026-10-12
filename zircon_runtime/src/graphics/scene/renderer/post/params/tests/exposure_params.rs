use super::*;

#[test]
fn exposure_params_preserve_the_authoritative_frame_delta() {
    let params = ExposureParams::new(
        UVec2::new(640, 360),
        RenderExposureSettings::default(),
        0.125,
    );

    assert_eq!(params.speeds_and_compensation[3], 0.125);
}
