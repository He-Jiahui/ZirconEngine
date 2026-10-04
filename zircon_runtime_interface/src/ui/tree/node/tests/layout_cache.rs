use crate::ui::layout::UiFrame;

use super::UiLayoutCache;

#[test]
fn text_layout_revision_advances_from_the_default_cache() {
    let mut cache = UiLayoutCache::default();

    cache.advance_text_layout_revision();

    assert_eq!(cache.text_layout_revision, 1);
    assert_eq!(cache.retained_text_layout_revision(), Some(1));
}

#[test]
fn exhausted_text_layout_revision_disables_retained_identity_without_wrapping() {
    let mut cache = UiLayoutCache {
        text_layout_revision: u64::MAX - 1,
        ..UiLayoutCache::default()
    };

    cache.advance_text_layout_revision();
    assert_eq!(cache.text_layout_revision, u64::MAX);
    assert_eq!(cache.retained_text_layout_revision(), None);

    cache.advance_text_layout_revision();
    assert_eq!(cache.text_layout_revision, u64::MAX);
    assert_eq!(cache.retained_text_layout_revision(), None);
}

#[test]
fn measurement_validity_is_independent_from_zero_geometry() {
    let mut cache = UiLayoutCache::default();

    cache.complete_measure();
    assert!(cache.measure_valid);

    cache.frame = UiFrame::default();
    assert!(cache.measure_valid);

    cache.invalidate_measure();
    assert!(!cache.measure_valid);
}
