use super::z_indices_are_ordered;

#[test]
fn ordered_and_equal_layers_stay_on_the_zero_sort_path() {
    assert!(z_indices_are_ordered([0, 0, 2, 8].into_iter()));
    assert!(z_indices_are_ordered([].into_iter()));
}

#[test]
fn descending_layer_requires_the_fallback_sort() {
    assert!(!z_indices_are_ordered([0, 4, 3, 8].into_iter()));
}
