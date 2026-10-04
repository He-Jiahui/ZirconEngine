use super::*;

#[test]
fn retained_generation_hit_still_requires_the_published_viewport() {
    let mut generation_counter = 0;
    let generation = ScreenSpaceUiTextFrameProductGeneration::next(&mut generation_counter);
    let mut prepared = PreparedSdfFrameInputs::default();
    prepared.replace(UVec2::new(800, 600), &[], &[], &[], &[], Some(generation));

    assert!(prepared.matches(UVec2::new(800, 600), &[], &[], &[], &[], Some(generation),));
    assert!(!prepared.matches(UVec2::new(801, 600), &[], &[], &[], &[], Some(generation),));
}

#[test]
fn retained_generation_replaces_owned_fallback_snapshots_with_generation_authority() {
    let mut generation_counter = 0;
    let generation = ScreenSpaceUiTextFrameProductGeneration::next(&mut generation_counter);
    let mut prepared = PreparedSdfFrameInputs::default();
    prepared.replace(
        UVec2::new(800, 600),
        &[],
        &[SdfRunCpuPreparation::default()],
        &[],
        &[TextDecorationMetrics::default()],
        None,
    );
    assert_eq!(prepared.cpu_runs.len(), 1);
    assert_eq!(prepared.native_decoration_metrics.len(), 1);

    prepared.replace_iter(
        UVec2::new(800, 600),
        std::iter::empty::<&ScreenSpaceUiTextBatch>(),
        &[],
        std::iter::empty::<&ScreenSpaceUiTextBatch>(),
        &[],
        Some(generation),
    );

    assert!(prepared.texts.is_empty());
    assert!(prepared.cpu_runs.is_empty());
    assert!(prepared.native_decoration_texts.is_empty());
    assert!(prepared.native_decoration_metrics.is_empty());
    assert!(prepared.matches_iter(
        UVec2::new(800, 600),
        std::iter::empty::<&ScreenSpaceUiTextBatch>(),
        &[],
        std::iter::empty::<&ScreenSpaceUiTextBatch>(),
        &[],
        Some(generation),
    ));
}
