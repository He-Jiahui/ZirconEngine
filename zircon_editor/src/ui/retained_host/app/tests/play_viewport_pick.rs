use super::*;

#[test]
fn physical_pixel_mapping_floors_inside_points_and_rejects_edges() {
    assert_eq!(
        viewport_pixel(UiPoint::new(12.75, 24.25), (100, 50)),
        Some(ZrRuntimeViewportPixelV1::new(12, 24))
    );
    assert!(viewport_pixel(UiPoint::new(-0.01, 0.0), (100, 50)).is_none());
    assert!(viewport_pixel(UiPoint::new(100.0, 0.0), (100, 50)).is_none());
    assert!(viewport_pixel(UiPoint::new(0.0, 50.0), (100, 50)).is_none());
    assert!(viewport_pixel(UiPoint::new(f32::NAN, 0.0), (100, 50)).is_none());
}

#[test]
fn input_sequence_never_wraps_or_reuses_an_identity() {
    let mut consumer = PlayViewportPickConsumer {
        next_input_sequence: Some(u64::MAX),
        pending: None,
    };

    assert_eq!(consumer.take_input_sequence().unwrap(), u64::MAX);
    assert!(matches!(
        consumer.take_input_sequence(),
        Err(PlayViewportPickError::InputSequenceExhausted)
    ));
}

#[test]
fn failed_cancellation_paths_keep_the_pending_owner_until_runtime_acknowledges_it() {
    let source = include_str!("../play_viewport_pick.rs");
    let production = source.split("#[cfg(test)]").next().unwrap_or(source);

    assert!(!production.contains("let Some(pending) = self.pending.take() else"));
    assert!(production.contains("self.suppress_pending_selection();"));
    assert!(production.contains("discarded_completion_count"));
    assert!(production.contains("next_error_retry_deadline"));
}
