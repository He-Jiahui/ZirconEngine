use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::MaterialDrawGenerationSelection;

use super::MaterialDrawSelection;

#[test]
fn current_generation_is_implicit_and_fallback_rows_are_sparse() {
    let id = ResourceId::from_stable_label("res://tests/material-selection");
    let mut selection = MaterialDrawSelection::default();

    assert!(selection.overrides.is_empty());
    selection.select(id, MaterialDrawGenerationSelection::PreviousPublished);
    assert_eq!(selection.overrides.len(), 1);
    assert!(selection.has_previous_proxies());
    assert!(!selection.has_error_proxies());
    selection.select(id, MaterialDrawGenerationSelection::ErrorProxy);
    assert!(!selection.has_previous_proxies());
    assert!(selection.has_error_proxies());
    selection.select(id, MaterialDrawGenerationSelection::Published);
    assert!(selection.overrides.is_empty());
    assert!(!selection.has_error_proxies());
}
