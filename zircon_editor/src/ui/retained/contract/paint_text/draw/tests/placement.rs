use super::*;

#[test]
fn grayscale_snaps_the_run_and_subpixel_preserves_its_origin() {
    assert_eq!(
        retained_text_origin_for_smoothing(20.75, HostTextSmoothing::Grayscale),
        21.0
    );
    assert_eq!(
        retained_text_origin_for_smoothing(20.75, HostTextSmoothing::Subpixel),
        20.75
    );
}

#[test]
fn non_finite_origins_use_the_named_fallback() {
    assert_eq!(
        retained_text_origin_for_smoothing(f32::NAN, HostTextSmoothing::Subpixel),
        FALLBACK_TEXT_ORIGIN_PX
    );
}
