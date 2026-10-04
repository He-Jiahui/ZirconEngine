use super::*;

#[test]
fn available_wrap_extent_preserves_narrow_finite_constraints() {
    assert_eq!(available_wrap_extent(0.25), 0.25);
    assert_eq!(available_wrap_extent(-0.25), 0.0);
    assert_eq!(available_wrap_extent(f32::NAN), 0.0);
    assert_eq!(available_wrap_extent(f32::INFINITY), f32::INFINITY);
}

#[test]
fn arabic_tatweel_budget_snapshot_matches_the_fit_algorithm_bounds() {
    let budget = arabic_tatweel_budget_snapshot();

    assert_eq!(
        budget.max_materialized_tatweels_per_line,
        MAX_ARABIC_TATWEELS_PER_LINE
    );
    assert_eq!(
        budget.max_fit_measurements_per_line,
        MAX_ARABIC_TATWEEL_FIT_MEASUREMENTS
    );
}

#[test]
fn arabic_tatweel_fit_uses_a_proportional_bounded_probe() {
    let mut attempted_counts = Vec::new();
    let count = bounded_arabic_tatweel_fit_count(32, 0.0, 100.0, |candidate_count| {
        attempted_counts.push(candidate_count);
        TextShapingOutcome::Ready(Some(candidate_count as f32 * 4.0))
    })
    .into_result()
    .expect("candidate probes remain available");

    assert_eq!(count, Some(25));
    assert_eq!(attempted_counts, vec![32, 25]);
}

#[test]
fn arabic_tatweel_fit_limits_unsuccessful_shape_probes() {
    let mut attempted_counts = Vec::new();
    let count = bounded_arabic_tatweel_fit_count(32, 0.0, 100.0, |candidate_count| {
        attempted_counts.push(candidate_count);
        TextShapingOutcome::Ready(Some(1_000.0))
    })
    .into_result()
    .expect("candidate probes remain available");

    assert_eq!(count, None);
    assert!(attempted_counts.len() <= MAX_ARABIC_TATWEEL_FIT_MEASUREMENTS);
    assert_eq!(attempted_counts.last(), Some(&1));
}

#[test]
fn arabic_tatweel_fit_reserves_its_last_probe_for_one_real_tatweel() {
    let mut attempted_counts = Vec::new();
    let count = bounded_arabic_tatweel_fit_count(32, 0.0, 100.0, |candidate_count| {
        attempted_counts.push(candidate_count);
        TextShapingOutcome::Ready(Some(100.0 + candidate_count as f32 * 0.001))
    })
    .into_result()
    .expect("candidate probes remain available");

    assert_eq!(count, Some(1));
    assert_eq!(attempted_counts, vec![32, 31, 30, 29, 1]);
}

#[test]
fn arabic_tatweel_fit_retries_one_candidate_after_backend_safety_rejection() {
    let mut attempted_counts = Vec::new();
    let count = bounded_arabic_tatweel_fit_count(32, 0.0, 100.0, |candidate_count| {
        attempted_counts.push(candidate_count);
        TextShapingOutcome::Ready((candidate_count == 1).then_some(4.0))
    })
    .into_result()
    .expect("candidate probes remain available");

    assert_eq!(count, Some(1));
    assert_eq!(attempted_counts, vec![32, 1]);
}

#[test]
fn aligned_x_keeps_extreme_line_box_coordinates_finite() {
    let frame = UiFrame::new(f32::MAX, 0.0, f32::MAX, 1.0);

    for align in [UiTextAlign::Center, UiTextAlign::Right, UiTextAlign::End] {
        let x = aligned_x(frame, f32::MAX, align, UiTextDirection::LeftToRight);
        assert_eq!(x, f32::MAX);
        assert!(x.is_finite());
    }
}
