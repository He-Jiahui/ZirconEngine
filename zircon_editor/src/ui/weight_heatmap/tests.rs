use std::sync::Arc;

use super::{
    static_field_cache_entry_count, WeightHeatmapGeneration, WeightHeatmapGenerationInput,
    WeightHeatmapSource, STATIC_FIELD_CACHE_CAPACITY,
};

fn source(x: f32, y: f32, weight: f32, selected: bool) -> WeightHeatmapSource {
    WeightHeatmapSource::new(x, y, weight, selected)
}

fn generation(sources: Vec<WeightHeatmapSource>) -> WeightHeatmapGeneration {
    WeightHeatmapGeneration::new(WeightHeatmapGenerationInput {
        columns: 32,
        rows: 24,
        low_label: "0.0".to_owned(),
        high_label: "1.0".to_owned(),
        sources,
    })
}

#[test]
fn selection_changes_only_the_dynamic_generation_and_reuses_the_static_field() {
    let idle = generation(vec![source(0.5, 0.5, 1.0, false)]);
    let selected = generation(vec![source(0.5, 0.5, 1.0, true)]);

    assert_eq!(idle.static_generation(), selected.static_generation());
    assert_ne!(idle.dynamic_generation(), selected.dynamic_generation());

    let idle_field = idle.static_field_for_plot_size(640.0, 420.0);
    let selected_field = selected.static_field_for_plot_size(640.0, 420.0);
    assert!(Arc::ptr_eq(&idle_field, &selected_field));
}

#[test]
fn source_geometry_change_invalidates_the_static_field() {
    let before = generation(vec![source(0.1, 0.5, 1.0, false)]);
    let after = generation(vec![source(0.9, 0.5, 1.0, false)]);

    assert_ne!(before.static_generation(), after.static_generation());
    let before_field = before.static_field_for_plot_size(320.0, 180.0);
    let after_field = after.static_field_for_plot_size(320.0, 180.0);
    assert!(!Arc::ptr_eq(&before_field, &after_field));
}

#[test]
fn generation_retains_every_source_without_silent_truncation() {
    let sources = (0..65_537)
        .map(|index| source(index as f32 / 65_537.0, 0.5, 1.0, index == 65_536))
        .collect::<Vec<_>>();
    let generation = generation(sources);
    assert_eq!(generation.sources().len(), 65_537);
    assert!(generation
        .sources()
        .last()
        .is_some_and(|source| source.selected()));
}

#[test]
fn field_resolution_is_bounded_by_requested_and_visible_pixels() {
    let field =
        generation(vec![source(0.5, 0.5, 1.0, false)]).static_field_for_plot_size(4_096.0, 4_096.0);

    assert_eq!((field.columns(), field.rows()), (32, 24));
    assert_eq!(field.intensities().len(), field.cell_count());
}

#[test]
fn source_and_visual_inputs_normalize_without_widening_the_field() {
    let generation = generation(vec![source(
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        false,
    )]);
    let field = generation.static_field_for_plot_size(f32::NAN, f32::INFINITY);

    let source = generation.sources().first().expect("normalized source");
    assert_eq!((source.x(), source.y(), source.weight()), (0.0, 0.0, 0.0));
    assert_eq!((field.columns(), field.rows()), (1, 1));
}

#[test]
fn static_field_cache_has_a_fixed_capacity() {
    for index in 0..STATIC_FIELD_CACHE_CAPACITY * 2 {
        generation(vec![source(index as f32 / 64.0, 0.5, 1.0, false)])
            .static_field_for_plot_size(320.0, 180.0);
    }

    assert!(static_field_cache_entry_count() <= STATIC_FIELD_CACHE_CAPACITY);
}
